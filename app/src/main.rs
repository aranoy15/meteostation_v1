#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use stm32f1xx_hal::gpio::{PushPull, Output};
use stm32f1xx_hal::gpio::gpioc::PC13;

use device_drivers::i2c::lcd::{Lcd, LcdTrait};
use device_drivers::i2c::rtc::traits::{RtcTrait, DateTimeTrait};
use device_drivers::i2c::rtc::datetime::DateTime;

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

    let shared_i2c= shared_bus::BusManagerSimple::new(i2c);

    //Configure lcd
    let mut lcd = Lcd::new(
        shared_i2c.acquire_i2c(),
        0x27,
        app::system::delay::Delay::new()
    );

    let mut rtc = device_drivers::i2c::rtc::ds3231::Rtc::new(
        shared_i2c.acquire_i2c(),
        0x68
    );

    lcd.init().unwrap();
    lcd.clear().unwrap();
    lcd.reset().unwrap();

    lcd.write_char('A').unwrap();
    lcd.write_char('B').unwrap();
    lcd.write_char('C').unwrap();

    let (mut serial, mut usb_dev) = app::system::usb::usb();

    let mut _blink_timer = app::system::timer::Timer::new();
    _blink_timer.start(500_u32);

    loop {
        check_usb_logic(&mut usb_dev, &mut serial, &mut lcd, &mut rtc);

        if _blink_timer.has_wrapped() {
            led.toggle().unwrap();
            _blink_timer.reset();
        }
    }
}

type UsbBusType = UsbBus<Peripheral>;
type UsbDeviceType<'a> = UsbDevice<'a, UsbBusType>;
type UsbSerialType<'a> = usbd_serial::SerialPort<'a, UsbBusType>;

fn check_usb_logic<LcdType, RtcType>(usb_dev: &mut UsbDeviceType,
                   serial: &mut UsbSerialType,
                   lcd: &mut LcdType,
                   rtc: &mut RtcType
)
    where
        LcdType: LcdTrait,
        RtcType: device_drivers::i2c::rtc::traits::RtcTrait<DateTime>
{
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 64];

    const RECEIVE_STR: &str = "Receive by time ";

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            lcd.clear().unwrap();
            lcd.set_cursor(0, 0).unwrap();
            lcd.write_bytes(&buf[0..count]).unwrap();

            let current_datetime = rtc.get().unwrap_or(DateTime::new());

            serial.write(RECEIVE_STR.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}

            let current_ticks = app::system::systick::current_tick() as u32;
            let temp_number = int_to_string(current_ticks);

            serial.write(temp_number.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}

            let year = current_datetime.get_year().unwrap_or(0);
            let month = current_datetime.get_month().unwrap_or(0);
            let day = current_datetime.get_day().unwrap_or(0);
            let hours = current_datetime.get_hours().unwrap_or(0);
            let minutes = current_datetime.get_minutes().unwrap_or(0);
            let seconds = current_datetime.get_seconds().unwrap_or(0);

            let mut test_output: heapless::String<heapless::consts::U256> = heapless::String::new();

            test_output.clear();

            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(year as u32).as_str()).unwrap();
            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(month as u32).as_str()).unwrap();
            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(day as u32).as_str()).unwrap();
            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(hours as u32).as_str()).unwrap();
            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(minutes as u32).as_str()).unwrap();
            test_output.push('\n').unwrap();
            test_output.push_str(int_to_string(seconds as u32).as_str()).unwrap();

            serial.write(test_output.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}
        }
        _ => {}
    }
}

type NumberStringType = heapless::String<heapless::consts::U16>;

fn int_to_string(number: u32) -> NumberStringType {
    let mut int_to_revers = number;
    int_to_revers = reverse_integer(int_to_revers);
    let mut result: NumberStringType = heapless::String::new();

    let is_have_last_zero = (number % 10) == 0;

    while int_to_revers != 0 {
        let current_char = (int_to_revers % 10) as u8;
        let temp_char = (current_char + 0x30_u8) as char;

        result.push(temp_char).unwrap();
        int_to_revers /= 10;
    }

    if is_have_last_zero {
        result.push('0').unwrap();
    }

    result
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



