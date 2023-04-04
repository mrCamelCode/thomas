use crate::core::{Entity, GameCommandQueue, GameServices};
use std::any::Any;
use std::collections::HashMap;

#[macro_export]
macro_rules! get_behaviour_name {
    ($name:ty) => {
        stringify!($name)
    };
}
pub(crate) struct EntityBehaviourMap {
    map: HashMap<String, EntityBehaviourMapValue>,
}
impl EntityBehaviourMap {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, behaviours: BehaviourList) {
        self.map.insert(
            entity.id().to_string(),
            EntityBehaviourMapValue {
                entity,
                entity_behaviour_list: behaviours,
            },
        );
    }

    pub(crate) fn update(
        &mut self,
        game_services: &GameServices,
        game_commands: &mut GameCommandQueue,
    ) {
        self.map.values_mut().for_each(|val| {
            val.entity_behaviour_list
                .update(&mut val.entity, game_services, game_commands)
        });
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Entity, &BehaviourList)> {
        self.map.iter().map(|(_, behaviour_map_value)| {
            (
                &behaviour_map_value.entity,
                &behaviour_map_value.entity_behaviour_list,
            )
        })
    }
}

pub(crate) struct EntityBehaviourMapValue {
    entity: Entity,
    entity_behaviour_list: BehaviourList,
}

pub struct BehaviourList {
    behaviours: HashMap<String, BehaviourMetaData>,
}
impl BehaviourList {
    pub fn new() -> Self {
        BehaviourList {
            behaviours: HashMap::new(),
        }
    }

    pub fn from(behaviours: Vec<Box<dyn CustomBehaviour>>) -> Self {
        let mut behaviours_map = HashMap::new();

        behaviours.into_iter().for_each(|behaviour| {
            behaviours_map.insert(
                behaviour.name().to_string(),
                BehaviourMetaData::new(behaviour),
            );
        });

        BehaviourList {
            behaviours: behaviours_map,
        }
    }

    pub fn get<T>(&self, name: &str) -> Option<&T>
    where
        T: CustomBehaviour + 'static,
    {
        if let Some(behaviour) = self.behaviours.get(name) {
            return behaviour.custom_behaviour.as_any().downcast_ref::<T>();
        }

        None
    }

    pub fn update(
        &mut self,
        entity: &mut Entity,
        game_services: &GameServices,
        game_commands: &mut GameCommandQueue,
    ) {
        self.behaviours.values_mut().for_each(|val| {
            let utils = BehaviourUtils::new(entity, game_services, game_commands);

            if val.has_been_init {
                val.custom_behaviour.update(utils);
            } else {
                val.custom_behaviour.init(utils);

                val.has_been_init = true;
            }
        });
    }

    pub fn add(&mut self, behaviour: Box<dyn CustomBehaviour>) {
        self.behaviours.insert(
            behaviour.name().to_string(),
            BehaviourMetaData::new(behaviour),
        );
    }

    pub fn remove(&mut self, behaviour_name: &str) {
        self.behaviours.remove(behaviour_name);
    }
}

pub trait Behaviour {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

pub trait CustomBehaviour: Behaviour {
    /// Invoked on the first frame this behaviour is alive.
    fn init(&mut self, _utils: BehaviourUtils) {}

    /// Invoked on every frame after the first init.
    fn update(&mut self, _utils: BehaviourUtils) {}
}

struct BehaviourMetaData {
    custom_behaviour: Box<dyn CustomBehaviour>,
    has_been_init: bool,
}
impl BehaviourMetaData {
    fn new(custom_behaviour: Box<dyn CustomBehaviour>) -> Self {
        Self {
            custom_behaviour,
            has_been_init: false,
        }
    }
}

pub struct BehaviourUtils<'a> {
    entity: &'a mut Entity,
    game_services: &'a GameServices,
    game_commands: &'a mut GameCommandQueue,
}
impl<'a> BehaviourUtils<'a> {
    pub(crate) fn new(
        entity: &'a mut Entity,
        game_services: &'a GameServices,
        game_commands: &'a mut GameCommandQueue,
    ) -> Self {
        Self {
            entity,
            game_services,
            game_commands,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_fn::MockFn;
    use thomas_derive::Behaviour;

    #[derive(Behaviour)]
    struct MockBehaviour {
        mock_update: MockFn,
        mock_init: MockFn,
    }
    impl MockBehaviour {
        pub fn new() -> Self {
            Self {
                mock_init: MockFn::new(),
                mock_update: MockFn::new(),
            }
        }
    }
    impl CustomBehaviour for MockBehaviour {
        fn update(&mut self, _utils: BehaviourUtils) {
            self.mock_update.call();
        }

        fn init(&mut self, _utils: BehaviourUtils) {
            self.mock_init.call();
        }
    }

    mod get_behaviour_name {
        #[test]
        fn name_is_correct() {
            assert_eq!(get_behaviour_name!(MockBehaviour), "MockBehaviour");
        }
    }

    mod entity_behaviour_map {
        use super::*;

        mod update {
            use crate::core::data::Transform;

            use super::*;

            fn make_mock() -> EntityBehaviourMap {
                let mut map = EntityBehaviourMap::new();

                map.add(
                    Entity::new("Test Entity 1", Transform::default()),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );
                map.add(
                    Entity::new("Test Entity 2", Transform::default()),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );
                map.add(
                    Entity::new("Test Entity 3", Transform::default()),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );

                map
            }

            fn get_mock_behaviour_from_mock_map(map: &EntityBehaviourMap) -> &MockBehaviour {
                let ids: Vec<String> = map
                    .map
                    .values()
                    .map(|val| val.entity.id().to_string())
                    .collect::<Vec<String>>();

                map.map
                    .get(&ids[0])
                    .unwrap()
                    .entity_behaviour_list
                    .get::<MockBehaviour>(get_behaviour_name!(MockBehaviour))
                    .unwrap()
            }

            fn make_services_mock() -> GameServices {
                GameServices::new()
            }

            fn make_commands_mock() -> GameCommandQueue {
                GameCommandQueue::new()
            }

            #[test]
            fn init_is_called_on_first_update() {
                let mut map = make_mock();
                let services = make_services_mock();
                let mut commands = make_commands_mock();

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 0);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 1);
                }
            }

            #[test]
            fn init_is_not_called_on_subsequent_updates() {
                let mut map = make_mock();
                let services = make_services_mock();
                let mut commands = make_commands_mock();

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 0);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 1);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 1);
                }
            }

            #[test]
            fn update_is_not_called_on_first_update() {
                let mut map = make_mock();
                let services = make_services_mock();
                let mut commands = make_commands_mock();

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 0);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 0);
                }
            }

            #[test]
            fn update_is_called_on_subsequent_updates() {
                let mut map = make_mock();
                let services = make_services_mock();
                let mut commands = make_commands_mock();

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 0);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 0);
                }

                map.update(&services, &mut commands);

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 1);
                }
            }
        }
    }
}
