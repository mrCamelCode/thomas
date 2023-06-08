use crate::{Component, IntCoords2d};

/// Positional data for a world in the terminal where it's 2D and strictly gridded.
#[derive(Component, Debug)]
pub struct TerminalTransform {
  pub coords: IntCoords2d,
}