use crate::{Component, IntCoords2d};

#[derive(Component, Debug)]
pub struct TerminalTransform {
  pub coords: IntCoords2d,
}