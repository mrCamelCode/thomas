use crate::core::Scene;

pub trait Renderer {
  fn render(&self, scene: &Scene);
}