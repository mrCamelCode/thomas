use crate::{Alignment, Component, IntCoords2d, Rgb};

/// Text UI that's rendered in the world space rather than a camera's screen space.
/// 
/// If you want text that's rendered in a camera's screen space, use `Text`.
#[derive(Component)]
pub struct WorldText {
    pub value: String,
    pub justification: Alignment,
    pub offset: IntCoords2d,
    pub foreground_color: Option<Rgb>,
    pub background_color: Option<Rgb>,
}
