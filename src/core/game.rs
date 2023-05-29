use std::{cell::RefCell, collections::HashMap, rc::Rc, thread, time::Duration};

use device_query::Keycode;

use crate::{
    Component, CustomExtraArgs, Entity, EntityManager, Input, Query, System, SystemExtraArgs,
    TerminalRendererOptions, TerminalRendererState, TerminalRendererSystems, Time, Timer,
};

const EVENT_INIT: &str = "init";
const EVENT_UPDATE: &str = "update";
const EVENT_CLEANUP: &str = "cleanup";

#[derive(PartialEq, Eq)]
pub enum Renderer {
    Terminal(TerminalRendererOptions),
}

pub struct GameOptions {
    pub press_escape_to_quit: bool,
    pub max_frame_rate: u16,
}

pub struct Game {
    entity_manager: EntityManager,
    events_to_systems: HashMap<&'static str, Vec<System>>,
    is_playing: bool,
    time: Rc<RefCell<Time>>,
    input: Rc<RefCell<Input>>,
    options: GameOptions,
    frame_timer: Timer,
}
impl Game {
    pub fn new(options: GameOptions) -> Self {
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
            options,
            frame_timer: Timer::new(),
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
        self = self.setup_builtin_systems();

        self.is_playing = true;

        self.trigger_event(EVENT_INIT, &self.make_extra_args(&commands, vec![]));

        while self.is_playing {
            self.frame_timer.restart();

            self.update_services();

            self.trigger_event(EVENT_UPDATE, &self.make_extra_args(&commands, vec![]));

            self.process_command_queue(Rc::clone(&commands));

            self.wait_for_frame();
        }

        self.trigger_event(EVENT_CLEANUP, &self.make_extra_args(&commands, vec![]));
    }

    fn wait_for_frame(&self) {
        let minimum_frame_time = if self.options.max_frame_rate > 0 {
            1000 / self.options.max_frame_rate
        } else {
            0
        };

        let elapsed_millis = self.frame_timer.elapsed_millis();
        if elapsed_millis < minimum_frame_time as u128 {
            thread::sleep(Duration::from_millis(
                (minimum_frame_time as u128 - elapsed_millis) as u64,
            ));
        }
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

    fn setup_builtin_systems(mut self) -> Self {
        if self.options.press_escape_to_quit {
            self = self.add_update_system(System::new(Query::new(), |_, util| {
                if util.input().is_key_down(&Keycode::Escape) {
                    util.commands().issue(GameCommand::Quit);
                }
            }));
        }

        self.add_update_system(System::new(Query::new(), |_, util| {
            if util.input().is_key_down(&Keycode::LControl) && util.input().is_key_down(&Keycode::C)
            {
                util.commands().issue(GameCommand::Quit);
            }
        }))
    }

    fn process_command_queue(&mut self, commands: Rc<RefCell<GameCommandQueue>>) {
        let old_commands = commands.replace(GameCommandQueue::new());

        for command in old_commands {
            match command {
                GameCommand::Quit => {
                    self.is_playing = false;
                }
                GameCommand::AddEntity(components) => {
                    self.entity_manager.add_entity(Entity::new(), components);
                }
                GameCommand::AddComponentsToEntity(entity, components) => {
                    for component in components {
                        self.entity_manager
                            .add_component_to_entity(&entity, component);
                    }
                }
                GameCommand::DestroyEntity(entity) => {
                    self.entity_manager.remove_entity(&entity);
                }
                GameCommand::RemoveComponentFromEntity(entity, component_name) => self
                    .entity_manager
                    .remove_component_from_entity(&entity, component_name),
                GameCommand::TriggerEvent(event_name, custom_args) => {
                    self.trigger_event(event_name, &self.make_extra_args(&commands, custom_args));
                }
            }
        }
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

pub enum GameCommand {
    Quit,
    AddEntity(Vec<Box<dyn Component>>),
    AddComponentsToEntity(Entity, Vec<Box<dyn Component>>),
    RemoveComponentFromEntity(Entity, &'static str),
    DestroyEntity(Entity),
    TriggerEvent(&'static str, CustomExtraArgs),
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
