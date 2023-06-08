use crate::{Component, Layer};

/// Marks that an entity in the world is capable of colliding with other `TerminalCollider`s.
#[derive(Component, Copy, Clone, Debug)]
pub struct TerminalCollider {
  /// The collision layer this collider is on. The layer can be used by a collision processing system to know
  /// what two kinds of things are colliding.
  pub layer: Layer,
  /// Whether the collider is active. If a collider isn't active, it won't generate any collisions with other colliders.
  pub is_active: bool,
}