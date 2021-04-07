#![no_std]
#![no_main]


use panic_halt as _;

use core::fmt::Write;

use cortex_m_rt::{entry};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usb_device::{device::{UsbDeviceBuilder, UsbVidPid}};
use stm32f1xx_hal::{prelude::*, stm32, usb};
use app::system::systick;
use app::system::clocks;

use heapless;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    systick::init(cp.SYST);

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let cfgr = rcc.cfgr;
    
    let clocks = clocks::init(cfgr, &mut flash.acr);

    assert!(clocks.usbclk_valid());

    let gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let usb_peripheral = usb::Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };

    let usb_bus = usb::UsbBus::new(usb_peripheral);
    let mut usb_serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(
        &usb_bus,
        UsbVidPid(0x16c0, 0x27dd),
    )
    .manufacturer("Fake company")
    .product("Serial port")
    .serial_number("TEST")
    .device_class(USB_CLASS_CDC)
    .build();

    const BUFFER_SIZE: usize = 64;
    let mut buf = [0u8; BUFFER_SIZE];

    let mut start = systick::current_tick();
    let mut output_string: heapless::String<heapless::consts::U512> = heapless::String::new();
    writeln!(output_string, "Second pass").unwrap();

    loop {
        if (systick::current_tick() - start) > 1000 {
            led.toggle().ok();
            start = systick::current_tick();
        }

        if !usb_dev.poll(&mut [&mut usb_serial]) {
            continue;
        }

        match usb_serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                //for data in buf.iter() {
                for (_, _) in [0..count].iter().enumerate() {
                    /* 
                    match wake_handler.process(buf[index]) {
                        Ok(packet) => {
                            writeln!(output_string, "Result packet:").unwrap();
                            writeln!(output_string, "Address: {}", packet.address).unwrap();
                            writeln!(output_string, "Command: {}", packet.command).unwrap();
                            writeln!(output_string, "Data length: {}", packet.length).unwrap();
                            usb_serial.write(output_string.as_bytes()).ok();
                            output_string.clear();
                        },
                        _ => {
                            
                        }
                    }
                    */
                }
            }
            _ => {}
        }
    }
}
