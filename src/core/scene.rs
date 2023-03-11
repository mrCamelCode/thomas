use super::Entity;

/// Houses a slice of a game world. Represents a single renderable
/// section of the world that contains Entities.
pub struct Scene<'a> {
    name: String,
    entities: Vec<Entity<'a>>,
}

impl<'a> Scene<'a>{
    pub fn new(name: &str) -> Scene {
        Scene {
            name: name.to_string(),
            entities: vec![],
        }
    }

    pub fn add_entity(&self) {

    }

    pub(crate) fn remove_entity(&self, entity: Entity) {

    }
}
