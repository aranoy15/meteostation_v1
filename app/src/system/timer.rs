
use crate::system::systick::{current_tick, TickType};
use embedded_hal::timer::{CountDown, Periodic, Cancel};
use nb;
use void::Void;

pub enum Error {
    Canceled,
}

#[derive(Clone, Copy)]
pub struct Timer {
    start_tick: TickType,
    timeout: TickType,
    is_started: bool
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start_tick: 0,
            timeout: 0,
            is_started: false
        }
    }

    pub fn start(&mut self, timeout: TickType)  {
        self.timeout = timeout;
        self.start_tick = current_tick();
        self.is_started = true;
    }

    pub fn reset(&mut self) {
        self.start_tick = current_tick();
    }

    pub const fn is_started(&self) -> bool {
        self.is_started
    }

    pub fn has_wrapped(&self) -> bool {
        if self.is_started() {
            (current_tick() - self.start_tick) >= self.timeout
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        self.is_started = false;
    }
}

impl CountDown for Timer {
    type Time = TickType;

    fn start<T>(&mut self, count: T)
    where
            T: Into<Self::Time> {
        
        self.start(count.into());
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Cancel for Timer {
    type Error = Error;

    fn cancel(&mut self) -> Result<(), Self::Error> {
         if !self.is_started() {
             return Err(Self::Error::Canceled);
         }

         self.stop();
         Ok(())
    }
}

impl Periodic for Timer {}
