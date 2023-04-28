use std::collections::{HashMap};

use crate::{Entity, Component};

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

  pub fn add_entity(entity: Entity, components: Vec<Box<dyn Component>>) -> u64 {
    0
  }

  pub fn remove_entity(entity: u64) {}

  // pub fn query(query: Query) -> impl Iterator<Item = Vec<&Box<dyn Component>>> {}
}