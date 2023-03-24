use std::collections::HashMap;

use crate::core::data::Transform;

use super::{behaviours::BehaviourList, GameUtil};

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

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    pub(crate) fn update(&mut self, util: &GameUtil) {
        self.behaviours.iter_mut().for_each(|behaviour| {
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
    use crate::core::behaviours::Behaviour;
    use thomas_derive::Behaviour;

    use crate::core::behaviours::CustomBehaviour;
    use crate::core::input;

    use super::*;

    struct MockFn {
        times_called: u32,
    }
    impl MockFn {
        pub fn new() -> Self {
            MockFn { times_called: 0 }
        }

        pub fn call(&mut self) {
            self.times_called += 1;
        }
    }

    struct MockBehaviour {
        mock_init: MockFn,
        mock_update: MockFn,
    }
    impl MockBehaviour {
        fn new() -> Self {
            MockBehaviour {
                mock_init: MockFn::new(),
                mock_update: MockFn::new(),
            }
        }
    }
    impl Behaviour for MockBehaviour {
        fn name(&self) -> &'static str {
            "MyBehaviour"
        }
    }
    impl CustomBehaviour for MockBehaviour {
        fn init(&mut self) {
            self.mock_init.call();
        }
        fn update(&mut self, _utils: &GameUtil) {
            self.mock_update.call();
        }
    }

    mod update {
        use super::*;

        #[test]
        fn calls_init_when_it_has_not_been_called_yet() {
            let mut entity = Entity::new("mock entity", Transform::default(), BehaviourList::new());
            let behaviour = MockBehaviour::new();

        }

        #[test]
        fn calls_update_when_behaviour_has_been_init() {}
    }
}
