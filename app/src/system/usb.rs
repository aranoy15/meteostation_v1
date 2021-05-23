
use stm32f1xx_hal as hal;
use hal::{stm32, prelude::*, usb::{UsbBus}};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usb_device::bus::UsbBusAllocator;
use embedded_hal::digital::v2::OutputPin;
use crate::system::delay::Delay;

fn peripherals() -> stm32::Peripherals {
    unsafe {
        stm32::Peripherals::steal()
    }
}

type UsbBusType = UsbBus<hal::usb::Peripheral>;
type UsbBusAllocatorType = UsbBusAllocator<UsbBusType>;
type SerialUsbType<'a> = SerialPort<'a, UsbBusType>;
type UsbDeviceType<'a> = UsbDevice<'a, UsbBusType>;

static mut USB_BUS: Option<UsbBusAllocatorType> = None;

pub fn usb() -> (SerialUsbType<'static>, UsbDeviceType<'static>) {
    let dp = peripherals();
    let mut rcc = dp.RCC.constrain();

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().unwrap();
    Delay::new().delay_ms(100_u16);

    let usb_per: hal::usb::Peripheral = hal::usb::Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh)
    };

    //static usb_bus: UsbBusAllocatorType = UsbBus::new(usb_per);
    unsafe {
        USB_BUS = Some(UsbBus::new(usb_per));

        usb_initialize(USB_BUS.as_ref().unwrap())
    }
}

fn usb_initialize<'a>(usb_bus: &UsbBusAllocatorType) -> (SerialUsbType, UsbDeviceType) {
    (
        SerialPort::new(&usb_bus),
        UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27de))
            .manufacturer("Fake company")
            .product("Serial port")
            .device_class(USB_CLASS_CDC)
            .build()
    )
}
