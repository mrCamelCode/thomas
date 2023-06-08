use std::collections::VecDeque;

use crate::{Component, Timer};

/// Represents stats tracked by the engine to report on its performance.
#[derive(Component)]
pub struct EngineStats {
  pub fps: u64,
  pub(crate) frame_timer: Timer,
  pub(crate) frame_counter: u64,
  pub(crate) frame_counts: VecDeque<u64>,
}