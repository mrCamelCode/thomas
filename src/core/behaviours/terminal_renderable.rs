use crate::core::{data::Layer, Behaviour, CustomBehaviour};
use thomas_derive::Behaviour;

use super::Renderable;

#[derive(Behaviour, Clone)]
pub struct TerminalRenderable {
    pub display: char,
    pub(crate) layer: Layer,
}
impl TerminalRenderable {
    pub fn new(display: char, layer: Layer) -> Self {
        Self { display, layer }
    }
}
impl Renderable for TerminalRenderable {
    fn layer(&self) -> &Layer {
        &self.layer
    }
}
impl CustomBehaviour for TerminalRenderable {}
