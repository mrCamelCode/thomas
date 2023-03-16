use crate::core::data::Coords;

/// Basic behaviour that stores positional information for an Entity.
pub struct Transform {
  coords: Coords,
}

impl Transform {
  pub fn new(position: Coords) -> Self {
    Transform { coords: position }
  }

  /// Provides the default Transform, which has all values zeroed out.
  pub fn default() -> Self {
    Transform { coords: Coords::zero() }
  }

  pub fn coords(&self) -> &Coords {
    &self.coords
  }

  pub fn move_by(&mut self, amount: &Coords) {
    self.coords.add(amount);
  }

  pub fn move_to(&mut self, new_position: &Coords) {
    self.coords = new_position.clone();
  }
}
