use crate::Component;
use crate::Layer;

#[derive(Component, Debug)]
pub struct TerminalRenderer {
    pub display: char,
    pub layer: Layer,
}
