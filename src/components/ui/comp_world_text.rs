use crate::{Alignment, Component, IntCoords2d, Rgb};

/// Text UI that's rendered in the world space rather than a camera's screen space.
#[derive(Component)]
pub struct WorldText {
    pub value: String,
    pub coords: IntCoords2d,
    pub justification: Alignment,
    pub offset: IntCoords2d,
    pub foreground_color: Option<Rgb>,
    pub background_color: Option<Rgb>,
}
