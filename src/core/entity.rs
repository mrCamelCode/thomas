use crate::core::behaviours::Behaviour;
use crate::core::behaviours::Transform;

use super::data::Vector2;
use super::Scene;

/// Representation of an entity in the game world. All objects that exist in the game world are Entities.
pub struct Entity {
    id: String,
    name: String,
    transform: Transform,
    behaviours: Vec<Box<dyn Behaviour>>,
    is_destroyed: bool,
}

impl Entity {
    pub fn create(name: &str, scene: &mut Scene) -> Entity {
        let entity = Entity {
            // TODO: Generate a new ID for every entity.
            id: "123".to_string(),
            name: name.to_string(),
            transform: Transform::new(Vector2::zero()),
            behaviours: vec![],
            is_destroyed: false,
        };

        scene.add_entity(&entity);

        entity
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn behaviours(&self) -> &Vec<Box<dyn Behaviour>> {
        &self.behaviours
    }
}
