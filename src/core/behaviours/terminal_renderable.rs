use thomas_derive::*;
use crate::core::Behaviour;

#[derive(Behaviour)]
pub struct TerminalRenderable {
  layer: u8,
}