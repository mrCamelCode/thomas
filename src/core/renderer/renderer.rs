use std::error::Error;

use crate::core::{BehaviourList, Entity};

pub trait Renderer {
    fn init(&self) {}

    fn render(&self, entities: Vec<(&Entity, &BehaviourList)>) -> Result<(), Box<dyn Error>>;

    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
