use super::{Behaviour, CustomBehaviour};
use thomas_derive::Behaviour;

#[derive(Behaviour)]
pub struct TerminalRenderable {
    pub display: char,
    pub layer: u8,
}
impl TerminalRenderable {
    pub fn new(display: char, layer: u8) -> Self {
        Self { display, layer }
    }
}
impl CustomBehaviour for TerminalRenderable {}
