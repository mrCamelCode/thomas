use super::{behaviours, Entity, GameUtil};

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

    // TODO: Call this when an entity that is_destroyed is encountered during update_entities
    fn remove_entity(&mut self, entity: Entity) {
        if let Some(found_index) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(found_index);
        };
    }

    pub(crate) fn update_entities(&self, util: &GameUtil) {
        self.entities.iter().for_each(|entity| {
            entity.behaviours().iter().for_each(|behaviour| {
                behaviour.update(util);
            })
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
