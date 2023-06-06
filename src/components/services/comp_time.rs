use std::time::Instant;

use crate::Component;

#[derive(Component)]
pub struct Time {
    last_frame_time: Instant,
}
impl Time {
    pub fn new() -> Self {
        Time {
            last_frame_time: Instant::now(),
        }
    }

    /// The time in milliseconds that's passed since the last update.
    pub fn delta_time(&self) -> u128 {
        self.last_frame_time.elapsed().as_millis()
    }

    pub(crate) fn update(&mut self) {
        self.last_frame_time = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod delta_time {
        use super::*;

        use std::{thread, time::Duration};

        #[test]
        fn has_difference_after_elapsed_time() {
            let mut time = Time::new();

            time.update();

            thread::sleep(Duration::from_millis(5));

            assert!(time.delta_time() >= 5);
        }
    }
}
