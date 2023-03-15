use crate::core::data::Transform;

use super::{behaviours::BehaviourList, Game, Scene};

/// Representation of an entity in the game world. All objects that exist in the game world are Entities.
pub struct Entity {
    id: String,
    name: String,
    transform: Transform,
    behaviours: BehaviourList,
    is_destroyed: bool,
}

impl Entity {
    pub fn new(name: &str, transform: Transform, behaviours: BehaviourList) -> Self {
        Entity {
            id: "123".to_string(),
            name: name.to_string(),
            transform,
            behaviours,
            is_destroyed: false,
        }
    }

    pub fn add_to_scene(self, scene: &mut Scene) {
        scene.add_entity(self)
    }

    pub fn add_to_game(self, game: &mut Game) {
        game.active_scene_as_mut().add_entity(self);
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

    pub fn behaviours(&self) -> &BehaviourList {
        &self.behaviours
    }
}
