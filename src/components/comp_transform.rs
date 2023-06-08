use crate::{Component, Coords};

/// Positional data for a 3D world.
#[derive(Component, Debug)]
pub struct Transform {
  pub coords: Coords,
}