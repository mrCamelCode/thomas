use std::error::Error;

use crate::core::{BehaviourList, Entity};

pub trait Renderer {
    fn init(&mut self) {}

    fn render(&mut self, entities: Vec<(&Entity, &BehaviourList)>) -> Result<(), Box<dyn Error>>;

    fn cleanup(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
