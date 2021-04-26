
use stm32f1xx_hal as hal;

use hal::{usb, stm32::{interrupt, Interrupt}};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usb_device::{class_prelude::UsbBusAllocator, device::{UsbDevice, UsbDeviceBuilder, UsbVidPid}};

use heapless::mpmc::Q64;

pub type DefaultString = heapless::String<heapless::consts::U64>;

pub const VID: u16 = 0x16C0;
pub const PID: u16 = 0x27DD;
pub const MANUFACTURER: &'static str = "Personal";
pub const PRODUCT: &'static str = "Serial port";
pub const SERIAL_NUMBER: &'static str = "103298383";

static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<'static, usb::UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<'static, usb::UsbBusType>> = None;

static INTPUT_BUFFER: Q64<u8> = Q64::new();

pub struct UsbLog {}

impl core::fmt::Write for UsbLog {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        match crate::peripherals::usb::write(s.as_bytes()) { _ => {} }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), core::fmt::Error> {
        match crate::peripherals::usb::write(&mut [c as u8]) { _ => {} }
        Ok(())
    }
}

#[macro_export]
macro_rules! usb_write {
    ($($arg:tt)*) => {
       {
           let mut temp_log = app::peripherals::usb::UsbLog {};
           match core::writeln!(&mut temp_log, $($arg)*) { _ => {} }
       } 
    };
}

pub fn init(peripheral: usb::Peripheral) {
    unsafe {
        USB_BUS = Some(usb::UsbBus::new(peripheral));
        USB_SERIAL = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(
            UsbDeviceBuilder::new(
                USB_BUS.as_ref().unwrap(),
                UsbVidPid(VID, PID)
            )
            .manufacturer(MANUFACTURER)
            .product(PRODUCT)
            .serial_number(SERIAL_NUMBER)
            .device_class(USB_CLASS_CDC)
            .build()
        );
        
        cortex_m::peripheral::NVIC::unmask(Interrupt::USB_HP_CAN_TX);
        cortex_m::peripheral::NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
    }
}

pub fn read(data: &mut [u8]) -> Result<usize, usize> {
    let mut result: usize = 0;

    while let Some(item) = INTPUT_BUFFER.dequeue() {
        if result >= data.len() {
            return Ok(result);
        }

        data[result] = item;
        result += 1;
    }

    if result > 0 {
        return Ok(result);
    } else {
        return Err(result);
    }
}

pub fn write(data: &[u8]) -> Result<usize, ()> {
    if data.len() > 64 {
        return Err(());
    }

    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    match serial.write(data) {
        _ => {}
    }

    Ok(data.len())
}

fn usb_interrupt() {
    let device = unsafe { USB_DEVICE.as_mut().unwrap() };
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    if !device.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 64];


    match serial.read(&mut buf) {
        Ok(count) => {
            for (i, _) in buf[0..count].iter().enumerate() {
                INTPUT_BUFFER.enqueue(buf[i]).ok();
            }
        },
        _ => {}
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

#[interrupt]
fn USB_LP_CAN_RX0() {
    usb_interrupt();
}
