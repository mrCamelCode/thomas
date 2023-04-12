use std::time::Instant;

pub struct Time {
    last_frame_time: Instant,
    delta_time: u128,
}
impl Time {
    pub fn new() -> Self {
        Time {
            last_frame_time: Instant::now(),
            delta_time: 0,
        }
    }

    /// The time in milliseconds that's passed since the last update.
    pub fn delta_time(&self) -> u128 {
        self.delta_time
    }

    pub(crate) fn update(&mut self) {
        self.delta_time = self.last_frame_time.elapsed().as_millis();

        self.last_frame_time = Instant::now();
    }
}

#[derive(Clone)]
pub struct Timer {
    start_time: Instant,
    is_running: bool,
}
impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            is_running: false,
        }
    }

    /// Starts the timer. This must be done before the timer will start giving you measured
    /// time on calls to elapsed methods. Has no effect on a timer that's already running.
    pub fn start(&mut self) {
        self.is_running = true;
    }

    /// Stops the timer. Any future calls to elapsed methods will effectively give 0.
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// Resets the timer such that its elapsed time at the moment of this call would be 0.
    /// The timer continues to run after this call.
    pub fn restart(&mut self) {
        self.start_time = Instant::now();
        self.is_running = true;
    }

    pub fn elapsed_seconds(&self) -> u64 {
        if self.is_running {
            self.start_time.elapsed().as_secs()
        } else {
            0
        }
    }

    pub fn elapsed_millis(&self) -> u128 {
        if self.is_running {
            self.start_time.elapsed().as_millis()
        } else {
            0
        }
    }

    pub fn is_running(&self) -> bool {
      self.is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod delta_time {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn has_no_difference_before_update() {
            let time = Time::new();

            thread::sleep(Duration::from_millis(5));

            assert_eq!(time.delta_time(), 0);
        }

        #[test]
        fn has_difference_after_elapsed_time() {
            let mut time = Time::new();

            thread::sleep(Duration::from_millis(5));

            time.update();

            assert!(time.delta_time() >= 5);
        }
    }
}
