use std::rc::Weak;

use super::{Entity, GameUtil, SceneManager};

/// Houses a slice of a game world. Represents a single renderable section of the world that contains Entities.
pub struct Scene {
    name: String,
    entities: Vec<Box<Entity>>,
}

impl Scene {
    pub fn new(name: &str) -> Scene {
        Scene {
            name: name.to_string(),
            entities: vec![],
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Entity> {
        if let Some(entity) = self.entities.iter().find(|entity| entity.name() == name) {
            return Some(entity);
        }

        None
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(Box::new(entity));
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        if let Some(found_index) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(found_index);
        };
    }

    pub(crate) fn update_entities(&mut self, util: &GameUtil) {
        self.entities.retain(|entity| !entity.is_destroyed());

        self.entities.iter_mut().for_each(|entity| {
            entity.update(util);
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn as_mut(&mut self) -> &mut Self {
        self
    }
}
