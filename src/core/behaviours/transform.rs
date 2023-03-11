use crate::core::data::Vector2;

/// Basic behaviour that stores positional information for an Entity.
pub struct Transform {
  position: Vector2,
}

impl Transform {
  pub fn new(position: Vector2) -> Transform {
    Transform { position }
  }
}
