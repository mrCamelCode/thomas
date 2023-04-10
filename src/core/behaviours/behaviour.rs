use dyn_clone::{clone_trait_object, DynClone};

use crate::core::data::Coords;
use crate::core::{Entity, GameCommandQueue, GameServices};
use std::any::Any;
use std::collections::HashMap;

#[macro_export]
macro_rules! get_behaviour_name {
    ($name:ty) => {
        stringify!($name)
    };
}

pub struct World {
    entity_behaviour_map: HashMap<String, EntityBehaviourMapValue>,
}
impl World {
    pub(crate) fn new() -> Self {
        Self {
            entity_behaviour_map: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, behaviours: BehaviourList) {
        self.entity_behaviour_map.insert(
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
        world: &World,
    ) {
        self.entity_behaviour_map
            .retain(|_, entity_behaviour_map_val| {
                let is_destroyed = entity_behaviour_map_val.entity.is_destroyed();

                let mut utils = BehaviourUtils::new(
                    &mut entity_behaviour_map_val.entity,
                    game_services,
                    game_commands,
                    world,
                );

                if is_destroyed {
                    entity_behaviour_map_val
                        .entity_behaviour_list
                        .destroy(&mut utils);
                } else {
                    entity_behaviour_map_val
                        .entity_behaviour_list
                        .update(&mut utils)
                }

                !entity_behaviour_map_val.entity.is_destroyed()
            });
    }

    pub fn entities(&self) -> impl Iterator<Item = (&Entity, &BehaviourList)> {
        self.entity_behaviour_map
            .iter()
            .map(|(_, behaviour_map_value)| {
                (
                    &behaviour_map_value.entity,
                    &behaviour_map_value.entity_behaviour_list,
                )
            })
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = (&mut Entity, &mut BehaviourList)> {
        self.entity_behaviour_map
            .iter_mut()
            .map(|(_, behaviour_map_value)| {
                (
                    &mut behaviour_map_value.entity,
                    &mut behaviour_map_value.entity_behaviour_list,
                )
            })
    }

    /// Retrieves an Entity and its Behaviours by the Entity's ID. This is an *O(1)* operation.
    /// Destroyed entities are ignored and considered `None`.
    pub fn get_entity(&self, entity_id: &str) -> Option<(&Entity, &BehaviourList)> {
        if let Some(entry) = self.entity_behaviour_map.get(entity_id) {
            return if !entry.entity.is_destroyed() {
                Some((&entry.entity, &entry.entity_behaviour_list))
            } else {
                None
            };
        }

        None
    }

    pub fn get_entity_mut(&mut self, entity_id: &str) -> Option<(&mut Entity, &mut BehaviourList)> {
        if let Some(entry) = self.entity_behaviour_map.get_mut(entity_id) {
            return if !entry.entity.is_destroyed() {
                Some((&mut entry.entity, &mut entry.entity_behaviour_list))
            } else {
                None
            };
        }

        None
    }

    // TODO: Unit test
    pub fn get_overlapping_entities(&self, entity_id: &str) -> Vec<(&Entity, &BehaviourList)> {
        if let Some((entity, _)) = self.get_entity(entity_id) {
            return self
                .entities()
                .filter_map(|(other_entity, other_behaviours)| {
                    // Note: This is currently fairly coupled to the idea that things run in the terminal. They're overlapping if their
                    // rounded position is the same. Should potentially look to decouple that.
                    if entity.id() != other_entity.id()
                        && entity.transform().coords().rounded()
                            == other_entity.transform().coords().rounded()
                    {
                        return Some((other_entity, other_behaviours));
                    }

                    None
                })
                .collect::<Vec<(&Entity, &BehaviourList)>>();
        }

        vec![]
    }
}
impl Clone for World {
    fn clone(&self) -> Self {
        let mut cloned_map = World::new();

        for (entity, behaviours) in self.entities() {
            cloned_map.add(entity.clone(), behaviours.clone());
        }

        cloned_map
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

    pub(crate) fn update(&mut self, utils: &mut BehaviourUtils) {
        self.behaviours.values_mut().for_each(|val| {
            if val.has_been_init {
                val.custom_behaviour.update(utils);
            } else {
                val.custom_behaviour.init(utils);

                val.has_been_init = true;
            }
        });
    }

    pub(crate) fn destroy(&mut self, utils: &mut BehaviourUtils) {
        self.behaviours.values_mut().for_each(|behaviour| {
            behaviour.custom_behaviour.on_destroy(utils);
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
impl Clone for BehaviourList {
    fn clone(&self) -> Self {
        let mut cloned_list = BehaviourList::new();

        for (behaviour_name, behaviour_meta_data) in self.behaviours.iter() {
            cloned_list
                .behaviours
                .insert(behaviour_name.clone(), behaviour_meta_data.clone());
        }

        cloned_list
    }
}

pub trait Behaviour {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

pub trait CustomBehaviour: Behaviour + DynClone {
    /// Invoked on the first frame this behaviour is alive.
    fn init(&mut self, _utils: &mut BehaviourUtils) {}

    /// Invoked on every frame after the first init.
    fn update(&mut self, _utils: &mut BehaviourUtils) {}

    fn on_destroy(&mut self, _utils: &mut BehaviourUtils) {}
}
clone_trait_object!(CustomBehaviour);

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
impl Clone for BehaviourMetaData {
    fn clone(&self) -> Self {
        Self {
            custom_behaviour: self.custom_behaviour.clone(),
            has_been_init: self.has_been_init,
        }
    }
}

#[allow(dead_code)]
pub struct BehaviourUtils<'a> {
    entity: &'a mut Entity,
    services: &'a GameServices,
    commands: &'a mut GameCommandQueue,
    world: &'a World,
}
impl<'a> BehaviourUtils<'a> {
    pub(crate) fn new(
        entity: &'a mut Entity,
        services: &'a GameServices,
        commands: &'a mut GameCommandQueue,
        world: &'a World,
    ) -> Self {
        Self {
            entity,
            services,
            commands,
            world,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_fn::MockFn;
    use thomas_derive::Behaviour;

    #[derive(Behaviour, Clone)]
    struct MockBehaviour {
        mock_update: MockFn,
        mock_init: MockFn,
        mock_on_destroy: MockFn,
    }
    impl MockBehaviour {
        pub fn new() -> Self {
            Self {
                mock_init: MockFn::new(),
                mock_update: MockFn::new(),
                mock_on_destroy: MockFn::new(),
            }
        }
    }
    impl CustomBehaviour for MockBehaviour {
        fn update(&mut self, _utils: &mut BehaviourUtils) {
            self.mock_update.call();
        }

        fn init(&mut self, _utils: &mut BehaviourUtils) {
            self.mock_init.call();
        }

        fn on_destroy(&mut self, _utils: &mut BehaviourUtils) {
            self.mock_on_destroy.call();
        }
    }

    mod get_behaviour_name {
        #[test]
        fn name_is_correct() {
            assert_eq!(get_behaviour_name!(MockBehaviour), "MockBehaviour");
        }
    }

    mod world {
        use super::*;

        mod update {
            use crate::core::data::Transform;

            use super::*;

            fn make_mock() -> World {
                let mut world = World::new();

                world.add(
                    Entity::new_with_id("Test Entity 1", Transform::default(), "te1"),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );
                world.add(
                    Entity::new_with_id("Test Entity 2", Transform::default(), "te2"),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );
                world.add(
                    Entity::new_with_id("Test Entity 3", Transform::default(), "te3"),
                    BehaviourList::from(vec![Box::new(MockBehaviour::new())]),
                );

                world
            }

            fn get_mock_behaviour_from_mock_map(world: &World) -> &MockBehaviour {
                world
                    .entity_behaviour_map
                    .get("te1")
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

                map.update(&services, &mut commands, &World::new());

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

                map.update(&services, &mut commands, &World::new());

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_init.num_calls(), 1);
                }

                map.update(&services, &mut commands, &World::new());

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

                map.update(&services, &mut commands, &World::new());

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

                map.update(&services, &mut commands, &World::new());

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 0);
                }

                map.update(&services, &mut commands, &World::new());

                {
                    let behaviour = get_mock_behaviour_from_mock_map(&map);
                    assert_eq!(behaviour.mock_update.num_calls(), 1);
                }
            }

            #[test]
            fn update_removes_destroyed_entities() {
                let mut world = make_mock();
                let services = make_services_mock();
                let mut commands = make_commands_mock();

                world
                    .entity_behaviour_map
                    .get_mut("te2")
                    .unwrap()
                    .entity
                    .destroy();

                assert_eq!(
                    world
                        .entity_behaviour_map
                        .keys()
                        .collect::<Vec<&String>>()
                        .len(),
                    3
                );

                world.update(&services, &mut commands, &World::new());

                let te1 = world.entity_behaviour_map.get("te1");
                let te2 = world.entity_behaviour_map.get("te2");
                let te3 = world.entity_behaviour_map.get("te3");

                assert!(te2.is_none());

                assert!(te1.is_some());
                assert!(te3.is_some());

                assert_eq!(
                    world
                        .entity_behaviour_map
                        .keys()
                        .collect::<Vec<&String>>()
                        .len(),
                    2
                );
            }
        }

        mod get_entity {
            use crate::core::data::Transform;

            use super::*;

            #[test]
            fn is_none_when_the_entity_does_not_exist() {
                let world = World::new();

                assert!(world.get_entity("test 1").is_none());
            }

            #[test]
            fn is_expected_entity_when_it_does_exist() {
                let mut world = World::new();

                let entity = Entity::new_with_id("e", Transform::default(), "test 1");

                world.add(entity, BehaviourList::new());

                let result = world.get_entity("test 1");

                assert!(result.is_some());
                assert_eq!(result.unwrap().0.id(), "test 1");
            }

            #[test]
            fn is_none_if_the_entity_is_destroyed() {
                let mut world = World::new();

                let mut entity = Entity::new_with_id("e", Transform::default(), "test 1");
                entity.destroy();

                world.add(entity, BehaviourList::new());

                assert!(world.get_entity("test 1").is_none());
            }
        }
    }
}
