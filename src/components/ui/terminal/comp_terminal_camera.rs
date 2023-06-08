use crate::{Component, Dimensions2d};

#[derive(Component)]
pub struct TerminalCamera {
    /// The amount of the world that will be visible to the camera. Anything exceeding
    /// the screen's maximum size in a direction cannot be rendered.
    pub field_of_view: Dimensions2d,
    /// Whether this is the main camera. There should only ever be one camera marked as main in the world at any
    /// given time.
    pub is_main: bool,
}
