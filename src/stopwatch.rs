
use crate::constants::Secs;
use std::time::{Instant};

#[derive(Debug, Clone)]
pub struct StopWatch {
    zero: Instant
}

impl StopWatch {

    pub fn new() -> StopWatch {
        StopWatch {
            zero: Instant::now()
        }
    }

    pub fn reset(self: &mut StopWatch) -> Secs {
        let now = Instant::now();
        let time = now - self.zero;
        self.zero = now;
        time.as_secs_f64()
    }

    pub fn time(self: &StopWatch) -> Secs {
        (Instant::now() - self.zero).as_secs_f64()
    }
}