use crate::{Component, Entity, TerminalCollider};

#[derive(Component)]
pub struct TerminalCollision {
  pub entities: [Entity; 2],
  pub colliders: [TerminalCollider; 2],
}