// src/utils/timer.rs
//
// The timer utility

pub struct Timer {
    duration: f32,
    elapsed: f32,
    paused: bool,
}

impl Timer {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            paused: false,
        }
    }

    // Advance the timer. Return true if timer is finished.
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.paused {
            return false;
        }

        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.reset();
            return true;
        }

        false
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn progress(&self) -> f32 {
        self.elapsed / self.duration
    }
}
