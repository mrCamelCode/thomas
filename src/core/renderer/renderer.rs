use crate::core::{BehaviourList, Entity};

pub trait Renderer {
    fn render(&self, entities: Vec<(&Entity, &BehaviourList)>);
}
