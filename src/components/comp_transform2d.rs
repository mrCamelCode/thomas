use crate::{Component, IntCoords2d};

#[derive(Component, Debug)]
pub struct Transform2d {
  // TODO: Update to regular coords2d.
  coords: IntCoords2d
}