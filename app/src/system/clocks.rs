
use stm32f1xx_hal as hal;

//use hal::{stm32, prelude::*, rcc, flash};
use hal::{stm32, prelude::*, rcc};
use crate::system;

const HSE_FREQ: u32 = 8;
const PCLK1_FREQ: u32 = system::config::SYSTEM_CLOCK / 2;
const PCLK2_FREQ: u32 = system::config::SYSTEM_CLOCK;

fn peripherals() -> hal::stm32::Peripherals {
    unsafe {
        stm32::Peripherals::steal()
    }
}

pub fn init() -> rcc::Clocks {
    let dp = peripherals();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    rcc.cfgr
        .use_hse(HSE_FREQ.mhz())
        .sysclk(system::config::SYSTEM_CLOCK.hz())
        .pclk1(PCLK1_FREQ.hz())
        .pclk2(PCLK2_FREQ.hz())
        .freeze(&mut flash.acr)
}
