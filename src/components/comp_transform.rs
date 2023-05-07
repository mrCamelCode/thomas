use crate::{Component, Coords};

#[derive(Component)]
pub struct Transform {
  pub coords: Coords,
}