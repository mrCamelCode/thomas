use std::time::{ Instant };

pub struct Time {
  last_frame_time: Instant,
  delta_time: u128,
}

impl Time {
  pub fn new() -> Self {
    Time {
      last_frame_time: Instant::now(),
      delta_time: 0
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