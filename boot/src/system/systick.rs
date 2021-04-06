
use cortex_m_rt::{exception};
use cortex_m::{peripheral::{SYST, syst}, interrupt::Mutex};
use core::cell::Cell;

pub type TickType = u32;

static COUNTER: Mutex<Cell<TickType>> = Mutex::new(Cell::new(0));

pub fn init(mut syst: SYST) {
    syst.set_clock_source(syst::SystClkSource::Core);
    syst.set_reload(72_000_u32);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}

pub fn current_tick() -> TickType {
    let mut result: TickType = 0;

    cortex_m::interrupt::free(|cs|{
        result = COUNTER.borrow(cs).get();
    });

    result
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs|{
        let mut value = COUNTER.borrow(cs).get();
        value += 1;
        COUNTER.borrow(cs).set(value);
    });
}
