use crate::core::{Entity, GameCommand, GameCommandQueue, GameServices};
use std::any::Any;
use std::collections::HashMap;

pub(crate) struct EntityBehaviourMap {
    map: HashMap<Entity, EntityBehaviourMapValue>,
}
impl EntityBehaviourMap {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
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
