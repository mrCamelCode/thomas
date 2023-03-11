/// A Behaviour is the basis for any logic you want to attach
/// to an Entity. 
pub trait Behaviour {
  /// Invoked once.
  fn init(&self) {}

  /// Invoked on every frame.
  fn update(&self) {}
}