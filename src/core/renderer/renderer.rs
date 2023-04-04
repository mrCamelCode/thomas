use crate::core::{BehaviourList, Entity};

pub trait Renderer {
    fn init(&self) {}

    fn render(&self, entities: Vec<(&Entity, &BehaviourList)>);

    fn cleanup(&self) {}
}
