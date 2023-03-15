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

  /// The time in milliseconds that's passed since the last frame.
  pub fn delta_time(&self) -> u128 {
    self.delta_time
  }

  pub(crate) fn update(&mut self) {
    self.delta_time = self.last_frame_time.elapsed().as_millis();

    self.last_frame_time = Instant::now();
  }
}