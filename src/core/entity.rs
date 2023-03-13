use crate::core::behaviours::Behaviour;
use crate::core::behaviours::Transform;

use super::Scene;
use super::data::Vector2;

/// Representation of an entity in the game world. All objects
/// that exist in the game world are Entities.
pub struct Entity<'a> {
    id: String,
    name: String,
    transform: Transform,
    behaviours: Vec<Box<dyn Behaviour>>,
}

impl<'a> Entity<'a> {
    pub fn create(name: &str, scene: &'a mut Scene<'a>) -> Entity<'a> {
        Entity {
            // TODO: Generate a new ID for every entity.
            id: "123".to_string(),
            name: name.to_string(),
            transform: Transform::new(Vector2::zero()),
            behaviours: vec![],
        }
    }

    /// Destroys the provided Entity, removing it from memory and the
    /// Scene it's in.
    pub fn destroy(entity: Entity) {
      entity.scene.remove_entity(entity);
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}
