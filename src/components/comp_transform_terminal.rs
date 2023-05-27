use crate::{Component, IntCoords2d};

#[derive(Component, Debug)]
pub struct TransformTerminal {
  pub coords: IntCoords2d,
}