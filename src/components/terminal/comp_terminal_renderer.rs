use crate::Component;
use crate::Layer;
use crate::Rgb;

/// Data to describe how to render something in the terminal.
#[derive(Component, Debug)]
pub struct TerminalRenderer {
    pub display: char,
    pub layer: Layer,
    pub foreground_color: Option<Rgb>,
    pub background_color: Option<Rgb>,
}
