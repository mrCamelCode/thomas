use crate::{Component, Entity, TerminalCollider};

/// Represents a collision between two `TerminalCollider`s. Also provides the entities that collided.
#[derive(Component)]
pub struct TerminalCollision {
    pub bodies: [(Entity, TerminalCollider); 2],
}
