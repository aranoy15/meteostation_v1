#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use stm32f1xx_hal::gpio::{PushPull, Output};
use stm32f1xx_hal::gpio::gpioc::PC13;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    app::system::systick::init(cp.SYST);

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = app::system::clocks::init(rcc.cfgr, &mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high().unwrap(); // Turn off

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    let mut local_delay = app::system::delay::Delay::new();

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().unwrap();
    local_delay.delay_ms(100_u16);

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
        check_usb_logic(&mut usb_dev, &mut serial, &mut led);
    }
}

type UsbBusType = UsbBus<Peripheral>;
type UsbDeviceType<'a> = UsbDevice<'a, UsbBusType>;
type UsbSerialType<'a> = usbd_serial::SerialPort<'a, UsbBusType>;
type LedType = PC13<Output<PushPull>>;


fn check_usb_logic(usb_dev: &mut UsbDeviceType, serial: &mut UsbSerialType, led: &mut LedType) {
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 64];

    const RECEIVE_STR: &str = "Receive by time ";

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            led.set_low().unwrap(); // Turn on

            serial.write(RECEIVE_STR.as_bytes()).unwrap();
            while !usb_dev.poll(&mut[serial]) {}

            let mut current_ticks = app::system::systick::current_tick() as u32;
            let mut temp_number: heapless::String<heapless::consts::U64> = heapless::String::new();

            if current_ticks == 0 {
                temp_number.push('0').unwrap();
            } else {
                current_ticks = reverse_integer(current_ticks);

                while current_ticks > 0 {
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



