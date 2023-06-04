use crate::{Component, Layer};

#[derive(Component, Copy, Clone, Debug)]
pub struct TerminalCollider {
  pub layer: Layer,
  pub is_active: bool,
}