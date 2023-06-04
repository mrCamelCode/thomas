use std::{cell::RefCell, collections::HashMap, rc::Rc, thread, time::Duration};

use device_query::Keycode;

use crate::{
    Component, CustomExtraArgs, Entity, EntityManager, Input, System, SystemExtraArgs,
    SystemsGenerator, TerminalRendererOptions, TerminalRendererState,
    TerminalRendererSystemsGenerator, Time, Timer,
};

pub const EVENT_INIT: &str = "init";
pub const EVENT_BEFORE_UPDATE: &str = "before-update";
pub const EVENT_UPDATE: &str = "update";
pub const EVENT_AFTER_UPDATE: &str = "after-update";
pub const EVENT_CLEANUP: &str = "cleanup";

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
            events_to_systems: HashMap::new(),
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

    pub fn add_after_update_system(self, system: System) -> Self {
        self.add_system(EVENT_AFTER_UPDATE, system)
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

    pub fn add_systems_from_generator(mut self, systems_generator: impl SystemsGenerator) -> Self {
        for (event_name, system) in systems_generator.generate() {
            self = self.add_system(event_name, system);
        }

        self
    }

    pub fn start(mut self, renderer: Renderer) {
        let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

        self = self.setup_renderer(renderer);
        self = self.setup_builtin_systems();

        self.sort_systems_by_priority();

        self.is_playing = true;

        let extra_args = self.make_extra_args(&commands, vec![]);

        self.trigger_event(EVENT_INIT, &extra_args);

        while self.is_playing {
            self.frame_timer.restart();

            self.update_services();

            self.trigger_event(EVENT_BEFORE_UPDATE, &extra_args);

            self.process_command_queue(Rc::clone(&commands));
            self.trigger_event(EVENT_UPDATE, &extra_args);
            self.process_command_queue(Rc::clone(&commands));

            self.trigger_event(EVENT_AFTER_UPDATE, &extra_args);

            self.wait_for_frame();
        }

        self.trigger_event(EVENT_CLEANUP, &extra_args);
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

    fn sort_systems_by_priority(&mut self) {
        for (_, system_list) in &mut self.events_to_systems {
            system_list.sort_by(|a, b| a.priority().cmp(&b.priority()))
        }
    }

    fn trigger_event(&self, event_name: &'static str, extra_args: &SystemExtraArgs) {
        if let Some(system_list) = self.events_to_systems.get(event_name) {
            for system in system_list {
                let queries_results = system
                    .queries()
                    .iter()
                    .map(|query| self.entity_manager.query(query))
                    .collect();

                system.operator()(queries_results, extra_args);
            }
        }
    }

    fn setup_renderer(mut self, renderer: Renderer) -> Self {
        match renderer {
            Renderer::Terminal(options) => {
                self.entity_manager
                    .add_entity(vec![Box::new(TerminalRendererState::new(options))]);

                return self
                    .add_systems_from_generator(TerminalRendererSystemsGenerator::new(options));
            }
        }
    }

    fn setup_builtin_systems(mut self) -> Self {
        if self.options.press_escape_to_quit {
            self = self.add_update_system(System::new(vec![], |_, util| {
                if util.input().is_key_down(&Keycode::Escape) {
                    util.commands().issue(GameCommand::Quit);
                }
            }));
        }

        self.add_update_system(System::new(vec![], |_, util| {
            if util
                .input()
                .is_chord_pressed_exclusively(&[&Keycode::LControl, &Keycode::C])
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
                    self.entity_manager.add_entity(components);
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

#[cfg(test)]
mod tests {
    use super::*;

    mod test_sort_systems_by_priority {
        use crate::Priority;

        use super::*;

        #[test]
        fn should_sort_by_priority_with_lower_numbers_at_the_front() {
            const EVENT_CUSTOM: &str = "custom";

            let mut game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_init_system(System::new_with_priority(
                Priority::new(5),
                vec![],
                |_, _| {},
            ))
            .add_init_system(System::new_with_priority(
                Priority::new(1),
                vec![],
                |_, _| {},
            ))
            .add_init_system(System::new_with_priority(
                Priority::new(3),
                vec![],
                |_, _| {},
            ))
            .add_system(
                EVENT_CUSTOM,
                System::new_with_priority(Priority::new(50), vec![], |_, _| {}),
            )
            .add_system(
                EVENT_CUSTOM,
                System::new_with_priority(Priority::new(10), vec![], |_, _| {}),
            )
            .add_system(
                EVENT_CUSTOM,
                System::new_with_priority(Priority::new(30), vec![], |_, _| {}),
            );

            game.sort_systems_by_priority();

            let init_systems = game.events_to_systems.get(EVENT_INIT).unwrap();
            let custom_systems = game.events_to_systems.get(EVENT_CUSTOM).unwrap();

            assert_eq!(**init_systems[0].priority(), 1);
            assert_eq!(**init_systems[1].priority(), 3);
            assert_eq!(**init_systems[2].priority(), 5);

            assert_eq!(**custom_systems[0].priority(), 10);
            assert_eq!(**custom_systems[1].priority(), 30);
            assert_eq!(**custom_systems[2].priority(), 50);
        }
    }

    mod test_add_system {
        use super::*;

        #[test]
        fn can_add_to_builtin_events() {
            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_init_system(System::new(vec![], |_, _| {}))
            .add_update_system(System::new(vec![], |_, _| {}))
            .add_cleanup_system(System::new(vec![], |_, _| {}));

            assert_eq!(game.events_to_systems.get(EVENT_INIT).unwrap().len(), 1);
            assert_eq!(game.events_to_systems.get(EVENT_UPDATE).unwrap().len(), 1);
            assert_eq!(game.events_to_systems.get(EVENT_CLEANUP).unwrap().len(), 1);
        }

        #[test]
        fn can_add_to_custom_event() {
            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_system("my key", System::new(vec![], |_, _| {}));

            assert_eq!(game.events_to_systems.get("my key").unwrap().len(), 1);
        }
    }

    mod test_add_systems_from_generator {
        use super::*;

        #[test]
        fn adds_all_systems_specified_in_generated_map() {
            const EVENT_CUSTOM: &str = "custom";

            struct TestGenerator {}
            impl SystemsGenerator for TestGenerator {
                fn generate(&self) -> Vec<(&'static str, System)> {
                    vec![
                        (EVENT_INIT, System::new(vec![], |_, _| {})),
                        (EVENT_CLEANUP, System::new(vec![], |_, _| {})),
                        (EVENT_CUSTOM, System::new(vec![], |_, _| {})),
                    ]
                }
            }

            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_systems_from_generator(TestGenerator {});

            assert_eq!(game.events_to_systems.len(), 3);

            assert_eq!(game.events_to_systems.get(EVENT_INIT).unwrap().len(), 1);
            assert_eq!(game.events_to_systems.get(EVENT_CLEANUP).unwrap().len(), 1);
            assert_eq!(game.events_to_systems.get(EVENT_CUSTOM).unwrap().len(), 1);
        }
    }

    mod test_trigger_event {
        use std::sync::atomic::{AtomicU8, Ordering};

        use super::*;

        const EVENT_1: &str = "1";

        #[test]
        fn triggering_event_calls_all_systems() {
            static COUNTER_1: AtomicU8 = AtomicU8::new(0);
            static COUNTER_2: AtomicU8 = AtomicU8::new(0);

            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_system(
                EVENT_1,
                System::new(vec![], |_, _| {
                    COUNTER_1.fetch_add(2, Ordering::Relaxed);
                }),
            )
            .add_system(
                EVENT_1,
                System::new(vec![], |_, _| {
                    COUNTER_2.fetch_add(5, Ordering::Relaxed);
                }),
            );

            game.trigger_event(
                EVENT_1,
                &game.make_extra_args(&Rc::new(RefCell::new(GameCommandQueue::new())), vec![]),
            );

            assert_eq!(COUNTER_1.fetch_add(0, Ordering::Relaxed), 2);
            assert_eq!(COUNTER_2.fetch_add(0, Ordering::Relaxed), 5);
        }

        #[test]
        fn systems_have_access_to_builtin_extra_args() {
            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_system(
                EVENT_1,
                System::new(vec![], |_, args| {
                    // These will panic if any fail.
                    args.commands();
                    args.input();
                    args.time();
                }),
            );

            game.trigger_event(
                EVENT_1,
                &game.make_extra_args(&Rc::new(RefCell::new(GameCommandQueue::new())), vec![]),
            );
        }

        #[test]
        fn systems_have_access_to_custom_extra_args() {
            let game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_system(
                EVENT_1,
                System::new(vec![], |_, args| {
                    let custom = args.try_get::<i32>("custom");

                    assert!(custom.is_some());
                    assert_eq!(*custom.unwrap(), 10);
                }),
            );

            game.trigger_event(
                EVENT_1,
                &game.make_extra_args(
                    &Rc::new(RefCell::new(GameCommandQueue::new())),
                    vec![("custom", Box::new(10))],
                ),
            );
        }
    }

    mod test_process_command_queue {
        use super::*;

        #[test]
        fn quit() {
            let mut game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            });
            game.is_playing = true;

            assert_eq!(game.is_playing, true);

            let commands = Rc::new(RefCell::new(GameCommandQueue::new()));
            commands.borrow_mut().issue(GameCommand::Quit);

            game.process_command_queue(Rc::clone(&commands));

            assert_eq!(game.is_playing, false);
        }

        #[test]
        fn queue_is_empty_after_processing() {
            let mut game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            });

            let commands = Rc::new(RefCell::new(GameCommandQueue::new()));
            commands.borrow_mut().issue(GameCommand::Quit);

            assert_eq!(commands.borrow().queue.len(), 1);

            game.process_command_queue(Rc::clone(&commands));

            assert_eq!(commands.borrow().queue.len(), 0);
        }
    }
}
