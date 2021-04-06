
use stm32f1xx_hal as hal;

use hal::{prelude::*, rcc, flash};
use crate::system;

pub fn init(cfgr: rcc::CFGR, acr: &mut flash::ACR) -> rcc::Clocks {
    cfgr
        .use_hse(8.mhz())
        .sysclk(system::config::SYSTEM_CLOCK.hz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(acr)
}
