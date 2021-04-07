#![no_std]
#![no_main]


use panic_halt as _;

use cortex_m_rt::{entry};
use stm32f1xx_hal::{prelude::*, stm32, usb};
use app::system::systick;
use app::system::clocks;
use app::peripherals::usb as app_usb;

use core::fmt::Write;


#[entry]
fn main() -> ! {
    if let (Some(cp), Some(dp)) = (
        cortex_m::Peripherals::take(),
        stm32::Peripherals::take()
    ) {

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

    app_usb::init(usb_peripheral);

    let mut start = systick::current_tick();

    loop {
        if (systick::current_tick() - start) > 1000 {
            led.toggle().ok();
            start = systick::current_tick();
            app::usb_write!("Tick");
        }
    }
    }

    panic!();
}
