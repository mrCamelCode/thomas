use crate::core::behaviours::Behaviour;
use crate::core::behaviours::Transform;

use super::Scene;
use super::data::Vector2;

/// Representation of an entity in the game world. All objects
/// that exist in the game world are Entities.
pub struct Entity<'a> {
    id: String,
    name: String,
    scene: &'a Scene<'a>,
    transform: Transform,
    behaviours: Vec<Box<dyn Behaviour>>,
}

impl<'a> Entity<'a> {
    pub fn new(name: &str, scene: &Scene) -> Entity<'a> {
        Entity {
            // TODO: Generate a new ID for every entity.
            id: "123".to_string(),
            name: name.to_string(),
            // TODO: Fix lifetime specifier complaint.
            scene,
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

}
