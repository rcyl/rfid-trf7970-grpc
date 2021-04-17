use std::time::Instant;
use std::{thread, time};

pub struct StopWatch {
    start: Instant,
    timeout_ms: u64,
}

impl StopWatch {

    pub fn new(timeout_ms: u64) -> StopWatch {
        StopWatch {
            start: Instant::now(),
            timeout_ms: timeout_ms,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        let duration = self.start.elapsed();
        duration.as_millis() as u64
    }

    pub fn timed_out(&self) -> bool {
        self.elapsed_ms() >= self.timeout_ms
    }
}