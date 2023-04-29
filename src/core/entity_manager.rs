use std::collections::{HashMap};

use crate::{Entity, Component, Query};

pub struct EntityManager {
  // TODO: Eventually implement a HashSet and use that instead of a vector.
  entities_to_components: HashMap<Entity, Vec<Box<dyn Component>>>,
  components_to_entities: HashMap<String, Vec<Entity>>,
}
impl EntityManager {
  pub fn new() -> Self {
    Self {
      entities_to_components: HashMap::new(),
      components_to_entities: HashMap::new(),
    }
  }

  pub fn add_entity(&mut self, entity: Entity, components: Vec<Box<dyn Component>>) -> u64 {
    0
  }

  pub fn remove_entity(&mut self, entity: u64) {}

  pub fn query(&self, query: Query) -> impl Iterator<Item = &dyn Component> {
    todo!("implement")
  }

  pub fn query_mut(&mut self, query: Query) -> impl Iterator<Item = &mut dyn Component> {

  }
}