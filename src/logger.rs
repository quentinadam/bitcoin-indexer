use crate::Chronometer;
use std::fmt::Display;

#[derive(Clone, Copy)]
pub struct Logger {
    chronometer: Chronometer,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            chronometer: Chronometer::new(),
        }
    }

    pub fn log(&self, value: impl Display) {
        println!("{}", format!("{} ({} elapsed)", value, self.chronometer.elapsed()));
    }
}

#[derive(Clone, Copy)]
pub struct PartialLogger<'a> {
    index: usize,
    interval: usize,
    logger: &'a Logger,
}

impl<'a> PartialLogger<'a> {
    pub fn new(interval: usize, logger: &'a Logger) -> Self {
        Self {
            index: 0,
            interval,
            logger,
        }
    }

    pub fn log<D: Display>(&mut self, f: impl FnOnce(usize) -> D) {
        if self.index % self.interval == 0 {
            self.logger.log(f(self.index));
        }
        self.index += 1;
    }

    pub fn interval(&self) -> usize {
        self.interval
    }
}
