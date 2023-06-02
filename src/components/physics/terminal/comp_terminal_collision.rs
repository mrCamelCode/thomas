use crate::{Component, Entity, TerminalCollider};

#[derive(Component)]
pub struct TerminalCollision {
    pub bodies: [(Entity, TerminalCollider); 2],
}
