use crate::Component;
use crate::Layer;

#[derive(Component, Debug)]
pub struct TerminalRenderable {
    pub display: char,
    pub layer: Layer,
}
