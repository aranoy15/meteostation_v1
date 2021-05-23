#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use stm32f1xx_hal::gpio::{PushPull, Output, OpenDrain, Alternate};
use stm32f1xx_hal::gpio::gpiob::{PB6, PB7};
use stm32f1xx_hal::gpio::gpioc::PC13;

use device_drivers::i2c::lcd::{Lcd, LcdTrait};

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    app::system::systick::init(cp.SYST);

    let mut rcc = dp.RCC.constrain();

    let clocks = app::system::clocks::init();

    assert!(clocks.usbclk_valid());

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high().unwrap(); // Turn off

    //Configure i2c
    let i2c = app::system::i2c::i2c1(clocks.clone());

    //Configure lcd
    let mut lcd: Lcd<app::system::i2c::I2C1Type, app::system::delay::Delay> = Lcd::new(
        i2c,
        0x27,
        app::system::delay::Delay::new()
    );

    lcd.init().unwrap();
    lcd.clear().unwrap();
    lcd.reset().unwrap();

    lcd.write_char('A').unwrap();
    lcd.write_char('B').unwrap();
    lcd.write_char('C').unwrap();

    let (mut serial, mut usb_dev) = app::system::usb::usb();

    loop {
        check_usb_logic(&mut usb_dev, &mut serial, &mut led, &mut lcd);
    }
}

type UsbBusType = UsbBus<Peripheral>;
type UsbDeviceType<'a> = UsbDevice<'a, UsbBusType>;
type UsbSerialType<'a> = usbd_serial::SerialPort<'a, UsbBusType>;
type LedType = PC13<Output<PushPull>>;
type LcdType = Lcd<stm32f1xx_hal::i2c::BlockingI2c<stm32f1xx_hal::stm32::I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>),>, app::system::delay::Delay>;

fn check_usb_logic(usb_dev: &mut UsbDeviceType,
                   serial: &mut UsbSerialType,
                   led: &mut LedType,
                   lcd: &mut LcdType
) {
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 64];

    const RECEIVE_STR: &str = "Receive by time ";

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            led.set_low().unwrap(); // Turn on

            lcd.clear().unwrap();
            lcd.set_cursor(0, 0).unwrap();
            lcd.write_bytes(&buf[0..count]).unwrap();

            serial.write(RECEIVE_STR.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}

            let mut current_ticks = app::system::systick::current_tick() as u32;
            let mut temp_number: heapless::String<heapless::consts::U64> = heapless::String::new();

            if current_ticks == 0 {
                temp_number.push('0').unwrap();
            } else {
                current_ticks = reverse_integer(current_ticks);

                while current_ticks != 0 {
                    let current_char: u8 = (current_ticks % 10) as u8;
                    let temp_char: char = (current_char + 0x30_u8) as char;

                    temp_number.push(temp_char).unwrap();

                    current_ticks /= 10;
                }
            }

            serial.write(temp_number.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}
        }
        _ => {}
    }

    led.set_high().unwrap(); // Turn off
}

fn reverse_integer(mut number: u32) -> u32 {
    let mut rev = 0_u32;

    while number != 0 {
        let pop = number % 10;
        number /= 10;

        rev = rev * 10 + pop;
    }

    rev
}



