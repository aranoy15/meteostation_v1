#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use stm32f1xx_hal::gpio::{PushPull, Output, OpenDrain, Alternate};
use stm32f1xx_hal::gpio::gpiob::{PB6, PB7};
use stm32f1xx_hal::gpio::gpioc::PC13;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    app::system::systick::init(cp.SYST);

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = app::system::clocks::init(rcc.cfgr, &mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high().unwrap(); // Turn off

    //Configure i2c
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let i2c = stm32f1xx_hal::i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        stm32f1xx_hal::i2c::Mode::Standard {
            frequency: 100_000.hz(),
        },
        clocks,
        &mut rcc.apb1,
        100_000,
        1,
        100_000,
        100_000,
    );

    //Configure lcd
    let mut lcd = app::drivers::lcd::Lcd::new(
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

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().unwrap();
    app::system::delay::Delay::new().delay_ms(100_u16);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27de))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    loop {
        check_usb_logic(&mut usb_dev, &mut serial, &mut led, &mut lcd);
    }
}

type UsbBusType = UsbBus<Peripheral>;
type UsbDeviceType<'a> = UsbDevice<'a, UsbBusType>;
type UsbSerialType<'a> = usbd_serial::SerialPort<'a, UsbBusType>;
type LedType = PC13<Output<PushPull>>;
type LcdType = app::drivers::lcd::Lcd<stm32f1xx_hal::i2c::BlockingI2c<stm32f1xx_hal::stm32::I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>),>, app::system::delay::Delay>;


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



