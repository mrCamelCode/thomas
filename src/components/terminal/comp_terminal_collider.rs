use crate::{Component, Layer};

#[derive(Component, Debug)]
pub struct TerminalCollider {
  pub layer: Layer,
  pub is_active: bool,
}