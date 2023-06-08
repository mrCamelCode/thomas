use crate::{Component, Coords2d};

/// Positional data for a 2D world.
#[derive(Component, Debug)]
pub struct Transform2d {
    pub coords: Coords2d,
}
