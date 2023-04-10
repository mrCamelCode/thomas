use crate::core::data::Layer;

pub trait Renderable {
    fn layer(&self) -> &Layer;
}
