use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    CustomExtraArgs, Entity, EntityManager, Input, System, SystemExtraArgs,
    TerminalRendererOptions, TerminalRendererState, TerminalRendererSystems, Time,
};

const EVENT_INIT: &str = "init";
const EVENT_UPDATE: &str = "update";
const EVENT_CLEANUP: &str = "cleanup";

#[derive(PartialEq, Eq)]
pub enum Renderer {
    Terminal(TerminalRendererOptions),
}

pub struct Game {
    entity_manager: EntityManager,
    events_to_systems: HashMap<&'static str, Vec<System>>,
    is_playing: bool,
    time: Rc<RefCell<Time>>,
    input: Rc<RefCell<Input>>,
}
impl Game {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            events_to_systems: HashMap::from([
                (EVENT_INIT, vec![]),
                (EVENT_UPDATE, vec![]),
                (EVENT_CLEANUP, vec![]),
            ]),
            is_playing: false,
            time: Rc::new(RefCell::new(Time::new())),
            input: Rc::new(RefCell::new(Input::new())),
        }
    }

    pub fn add_init_system(self, system: System) -> Self {
        self.add_system(EVENT_INIT, system)
    }

    pub fn add_update_system(self, system: System) -> Self {
        self.add_system(EVENT_UPDATE, system)
    }

    pub fn add_cleanup_system(self, system: System) -> Self {
        self.add_system(EVENT_CLEANUP, system)
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
        let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

        self = self.setup_renderer(renderer);

        self.is_playing = true;

        self.trigger_event(EVENT_INIT, &self.make_extra_args(&commands, vec![]));

        while self.is_playing {
            self.update_services();

            self.trigger_event(EVENT_UPDATE, &self.make_extra_args(&commands, vec![]));

            self.process_command_queue(&mut commands.borrow_mut());
        }

        self.trigger_event(EVENT_CLEANUP, &self.make_extra_args(&commands, vec![]));
    }

    fn update_services(&mut self) {
        self.time.borrow_mut().update();
        self.input.borrow_mut().update();
    }

    fn trigger_event(&self, event_name: &'static str, extra_args: &SystemExtraArgs) {
        if let Some(system_list) = self.events_to_systems.get(event_name) {
            for system in system_list {
                let query_results = self.entity_manager.query(system.query());

                system.operator()(query_results, extra_args);
            }
        }
    }

    fn setup_renderer(mut self, renderer: Renderer) -> Self {
        match renderer {
            Renderer::Terminal(options) => {
                self.entity_manager.add_entity(
                    Entity::new(),
                    vec![Box::new(TerminalRendererState::new(options))],
                );

                let (init_system, update_system, cleanup_system) =
                    TerminalRendererSystems::new(options).extract_systems();

                return self
                    .add_init_system(init_system)
                    .add_update_system(update_system)
                    .add_cleanup_system(cleanup_system);
            }
        }
    }

    fn process_command_queue(&mut self, commands: &mut GameCommandQueue) {
        for command in &commands.queue {}

        commands.queue.clear();
    }

    fn make_extra_args(
        &self,
        commands: &Rc<RefCell<GameCommandQueue>>,
        custom_pairs: CustomExtraArgs,
    ) -> SystemExtraArgs {
        SystemExtraArgs::new(
            Rc::clone(commands),
            Rc::clone(&self.input),
            Rc::clone(&self.time),
            custom_pairs,
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
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
}
impl IntoIterator for GameCommandQueue {
    type Item = GameCommand;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}
impl<'a> IntoIterator for &'a GameCommandQueue {
    type Item = <std::slice::Iter<'a, GameCommand> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, GameCommand>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.queue).into_iter()
    }
}
