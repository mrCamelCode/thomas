use crate::{Component, Coords};

#[derive(Component)]
pub struct Transform {
  coords: Coords,
}