use crate::{Component, Coords};

#[derive(Component, Debug)]
pub struct Transform {
  pub coords: Coords,
}