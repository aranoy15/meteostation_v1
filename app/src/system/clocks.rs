
use stm32f1xx_hal as hal;

use hal::{prelude::*, rcc, flash};
use crate::system;

const HSE_FREQ: u32 = 8;
const PCLK1_FREQ: u32 = system::config::SYSTEM_CLOCK / 2;
const PCLK2_FREQ: u32 = system::config::SYSTEM_CLOCK;

pub fn init(cfgr: rcc::CFGR, acr: &mut flash::ACR) -> rcc::Clocks {
    cfgr
        .use_hse(HSE_FREQ.mhz())
        .sysclk(system::config::SYSTEM_CLOCK.hz())
        .pclk1(PCLK1_FREQ.mhz())
        .pclk2(PCLK2_FREQ.mhz())
        .freeze(acr)
}
