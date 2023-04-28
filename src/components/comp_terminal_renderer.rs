use crate::Component;
use crate::Layer;

#[derive(Component)]
pub struct TerminalRenderable {
    pub display: char,
    pub layer: Layer,
}
