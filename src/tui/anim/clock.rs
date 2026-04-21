//! Monotonic frame clock. `tick()` once per render loop iteration before draw;
//! `dt` becomes the time since the previous tick. `elapsed()` returns total
//! time since the clock was created — useful for periodic pulses.

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct FrameClock {
    start: Instant,
    last: Instant,
    dt: Duration,
    frame: u64,
}

impl FrameClock {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last: now,
            dt: Duration::ZERO,
            frame: 0,
        }
    }

    /// Advance the clock. Call once per loop iteration before draw.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.dt = now.saturating_duration_since(self.last);
        self.last = now;
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn elapsed(&self) -> Duration {
        self.last.saturating_duration_since(self.start)
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::new()
    }
}
