#![no_std]
#![no_main]


use panic_halt as _;

use cortex_m_rt::{entry};
use stm32f1xx_hal::{prelude::*, stm32, usb, i2c, timer};
use app::system::systick;
use app::system::clocks;
use app::peripherals::usb as app_usb;
use core::fmt::Write;
use nb::block;

#[entry]
fn main() -> ! {
    if let (Some(cp), Some(dp)) = (
        cortex_m::Peripherals::take(),
        stm32::Peripherals::take()
    ) {

    systick::init(cp.SYST);

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let cfgr = rcc.cfgr;

    let clocks = clocks::init(cfgr, &mut flash.acr);

    assert!(clocks.usbclk_valid());

    let gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let i2c = i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        i2c::Mode::Standard {
            frequency: 100_000.hz()
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000
    );

    let mut local_delay = app::system::delay::Delay::new();

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let usb_peripheral = usb::Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };

    app_usb::init(usb_peripheral);

    let mut timer = app::system::timer::Timer::new();
    timer.start(1000_u32);

    loop {
        led.toggle().ok();
        match block!(timer.wait()) {
            Ok(()) => { timer.reset(); },
            Err(_) => {}
        }
        //timer.reset();
    }
    }

    panic!();
}
