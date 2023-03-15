use crate::core::data::Vector2;

/// Basic behaviour that stores positional information for an Entity.
pub struct Transform {
  position: Vector2,
}

impl Transform {
  pub fn new(position: Vector2) -> Self {
    Transform { position }
  }

  /// Provides the default Transform, which has all values zeroed out.
  pub fn default() -> Self {
    Transform { position: Vector2::zero() }
  }

  pub fn position(&self) -> &Vector2 {
    &self.position
  }

  pub fn displace(&mut self, amount: &Vector2) {
    self.position.add(amount);
  }

  pub fn relocate(&mut self, new_position: &Vector2) {
    self.position = new_position.clone();
  }
}
