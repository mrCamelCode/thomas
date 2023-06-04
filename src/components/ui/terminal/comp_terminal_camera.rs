use crate::{Component, Dimensions2d};

#[derive(Component)]
pub struct TerminalCamera {
    /// The amount of the world that will be visible to the camera. Anything exceeding
    /// the screen's maximum size in a direction cannot be rendered.
    pub field_of_view: Dimensions2d,
    pub is_main: bool,
}
