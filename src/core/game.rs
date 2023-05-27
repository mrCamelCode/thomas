use std::collections::HashMap;

use crate::{EntityManager, OperatorFn, Query, System, TerminalRendererOptions};

const EVENT_INIT: &str = "init";
const EVENT_UPDATE: &str = "update";
const EVENT_RENDER: &str = "render";

#[derive(PartialEq, Eq)]
pub enum Renderer {
    Terminal(TerminalRendererOptions),
}

pub struct Game {
    entity_manager: EntityManager,
    events_to_systems: HashMap<&'static str, Vec<System>>,
    is_playing: bool,
}
impl Game {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            events_to_systems: HashMap::from([
                (EVENT_INIT, vec![]),
                (EVENT_UPDATE, vec![]),
                (EVENT_RENDER, vec![]),
            ]),
            is_playing: false,
        }
    }

    pub fn add_init_system(self, system: System) -> Self {
        self.add_system(EVENT_INIT, system)
    }

    pub fn add_update_system(self, system: System) -> Self {
        self.add_system(EVENT_UPDATE, system)
    }

    pub fn add_system(mut self, event_name: &'static str, system: System) -> Self {
        if !self.events_to_systems.contains_key(event_name) {
            self.events_to_systems.insert(event_name, vec![]);
        }

        if let Some(system_list) = self.events_to_systems.get_mut(event_name) {
            system_list.push(system);
        }

        self
    }

    pub fn start(mut self, renderer: Renderer) {
        self = self.setup_renderer(renderer);

        self.is_playing = true;

        self.trigger_event(EVENT_INIT);

        while self.is_playing {
            self.trigger_event(EVENT_UPDATE);
            self.trigger_event(EVENT_RENDER);
        }
    }

    fn trigger_event(&self, event_name: &'static str) {
        if let Some(system_list) = self.events_to_systems.get(event_name) {
            for system in system_list {
                let query_results = self.entity_manager.query(system.query());

                system.operator()(query_results);
            }
        }
    }

    fn setup_renderer(self, renderer: Renderer) -> Self {
        match renderer {
            Renderer::Terminal(options) => {
                return self.add_system(EVENT_RENDER, System::new(Query::new(), |qr| {}));
            }
            _ => self,
        }
    }
}

pub struct GameCommandQueue {
    queue: Vec<GameCommand>,
}
impl GameCommandQueue {
    pub(crate) fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn issue(&mut self, command: GameCommand) {
        self.queue.push(command);
    }

    pub(crate) fn consume(self) -> impl Iterator<Item = GameCommand> {
        self.queue.into_iter()
    }
}

pub enum GameCommand {
    //     Quit,
    //     ClearEntities,
    //     AddEntity {
    //         entity: Entity,
    //         behaviours: BehaviourList,
    //     },
    //     DestroyEntity(String),
    //     SendMessage {
    //         entity_id: String,
    //         message: Message<Box<dyn Any>>,
    //     },
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod game_command_queue {
        use super::*;

        // #[test]
        // fn goes_through_commands_in_the_order_they_were_issued() {
        //     let mut queue = GameCommandQueue::new();

        //     queue.issue(GameCommand::Quit);
        //     queue.issue(GameCommand::ClearEntities);
        //     queue.issue(GameCommand::AddEntity {
        //         entity: Entity::new("test", Transform::default()),
        //         behaviours: BehaviourList::new(),
        //     });

        //     let mut iter = queue.consume();

        //     assert!(match iter.next().unwrap() {
        //         GameCommand::Quit => true,
        //         _ => false,
        //     });
        //     assert!(match iter.next().unwrap() {
        //         GameCommand::ClearEntities => true,
        //         _ => false,
        //     });
        //     assert!(match iter.next().unwrap() {
        //         GameCommand::AddEntity { .. } => true,
        //         _ => false,
        //     });
        //     assert!(iter.next().is_none());
        // }
    }
}
