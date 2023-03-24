use std::collections::HashMap;

use crate::core::data::Transform;

use super::{behaviours::BehaviourList, Game, GameUtil, Scene};

/// Representation of an entity in the game world. All objects that exist in the game world are Entities.
pub struct Entity {
    id: String,
    name: String,
    transform: Transform,
    behaviours: BehaviourList,
    is_destroyed: bool,
    behaviour_init_map: HashMap<String, bool>,
}

impl Entity {
    pub fn new(name: &str, transform: Transform, behaviours: BehaviourList) -> Self {
        Entity {
            id: "123".to_string(),
            name: name.to_string(),
            transform,
            behaviours,
            is_destroyed: false,
            behaviour_init_map: HashMap::new(),
        }
    }

    pub fn add_to_scene(self, scene: &mut Scene) {
        scene.add_entity(self)
    }

    pub fn add_to_game(self, game: &mut Game) {
        game.util()
            .scene_manager()
            .active_scene_as_mut()
            .add_entity(self);
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    pub(crate) fn update(&mut self, util: &GameUtil) {
        self.behaviours.iter().for_each(|behaviour| {
            match self.behaviour_init_map.get(behaviour.name()) {
                Some(has_been_init) if *has_been_init => behaviour.update(util),
                _ => {
                    behaviour.init();

                    self.behaviour_init_map
                        .insert(behaviour.name().to_string(), true);
                }
            }
        })
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

#[cfg(test)]
mod tests {
    use thomas_derive::Behaviour;
    use crate::core::behaviours::Behaviour;

    use crate::core::behaviours::CustomBehaviour;
    use crate::core::input;

    use super::*;

    struct MockBehaviour {
        init_count: u32,
        update_count: u32, 
    }

    impl Behaviour for MockBehaviour {
        fn name(&self) -> &'static str {
            "MyBehaviour"
        }
    }

    impl CustomBehaviour for MockBehaviour {
        fn init(&self) {
            
        }
    }

    mod update {
        use super::*;

        #[test]
        fn calls_init_when_it_has_not_been_called() {

        }

        #[test]
        fn calls_update_when_behaviour_has_been_init() {

        }

    }
}
