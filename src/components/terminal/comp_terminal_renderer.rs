use crate::Component;
use crate::Layer;

/// Data to describe how to render something in the terminal.
#[derive(Component, Debug)]
pub struct TerminalRenderer {
    pub display: char,
    pub layer: Layer,
}
