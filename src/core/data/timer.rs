use std::time::Instant;

/// A way to track the passage of real time.
#[derive(Clone)]
pub struct Timer {
    start_time: Instant,
    is_running: bool,
}
impl Timer {
    /// Creates a new `Timer` instance that isn't started. A `Timer` must be started before it'll give any readings
    /// on elapsed time.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            is_running: false,
        }
    }

    /// Creates a new `Timer` and starts it.
    pub fn start_new() -> Self {
        Self {
            start_time: Instant::now(),
            is_running: true,
        }
    }

    /// Starts the timer. This must be done before the timer will start giving you measured
    /// time on calls to elapsed methods. Has no effect on a timer that's already running.
    pub fn start(&mut self) {
        self.start_time = Instant::now();
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

    /// Whether the timer is currently running. A Timer must be running to report on elapsed time.
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
