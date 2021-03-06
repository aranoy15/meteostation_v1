

use super::systick::{current_tick, TickType};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use cortex_m::asm;
use crate::system::config::SYSTEM_CLOCK;

#[derive(Clone, Copy)]
pub struct Delay {
    tick: TickType,
}

impl Delay {
    pub fn new() -> Self {
        Delay { tick: 0 }
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.tick = current_tick(); 
        while (current_tick() - self.tick) < ms {}
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        let ticks = us * SYSTEM_CLOCK / 1_000_000;
        asm::delay(ticks);
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32);
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32);
    }
}
