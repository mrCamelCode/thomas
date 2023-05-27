use crate::{Component, Coords2d};

#[derive(Component, Debug)]
pub struct Transform2d {
    pub coords: Coords2d,
}
