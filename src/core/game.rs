use std::{cell::RefCell, collections::HashMap, rc::Rc};

use device_query::Keycode;

use crate::{
    Component, Entity, EntityManager, Input, Query, ServicesSystemsGenerator, System,
    SystemsGenerator, TerminalRendererOptions, TerminalRendererState,
    TerminalRendererSystemsGenerator, Timer,
};

pub type GameCommandsArg = Rc<RefCell<GameCommandQueue>>;

/// The init event. Runs once before the main game loop starts.
pub const EVENT_INIT: &str = "init";
/// The before-update event. Runs once per frame before the update event.
pub const EVENT_BEFORE_UPDATE: &str = "before-update";
/// The update event. Runs once per frame.
pub const EVENT_UPDATE: &str = "update";
/// The after-update event. Runs once per frame after the update event.
pub const EVENT_AFTER_UPDATE: &str = "after-update";
/// The cleanup event. Runs once after the main game loop ends.
pub const EVENT_CLEANUP: &str = "cleanup";

#[derive(PartialEq, Eq)]
pub enum Renderer {
    Terminal(TerminalRendererOptions),
}

pub struct GameOptions {
    /// Whether the user can press the Escape key to quit the game. Note that in a terminal game, the user
    /// can always press Ctrl+C to quit the game.
    pub press_escape_to_quit: bool,
    /// The maximum number of times the main game loop should run in one second. A value of 0 indicates an uncapped
    /// frame rate.
    pub max_frame_rate: u16,
}

/// The core structure of any game made with Thomas. The `Game` instance facilitates communication with Thomas' internal
/// mechanisms to automate the nitty gritty details of running the game.
///
/// All Thomas games will begin with using `Game`:
/// ```
/// use thomas::{Game, Renderer, TerminalRendererOptions, Dimensions2d, Text, System, GameCommand, UiAnchor, Alignment, IntCoords2d, GameOptions};
///
/// Game::new(GameOptions {
///     press_escape_to_quit: false,
///     max_frame_rate: 60,
/// })
/// .add_init_system(System::new(vec![], |_, commands| {
///     commands.borrow_mut().issue(GameCommand::AddEntity(vec![
///         Box::new(Text {
///             anchor: UiAnchor::TopLeft,
///             justification: Alignment::Left,
///             offset: IntCoords2d::zero(),
///             value: String::from("Hello Thomas!"),
///         }),
///     ]));
/// }));
/// // .start(Renderer::Terminal(TerminalRendererOptions {
/// //     screen_resolution: Dimensions2d::new(10, 30),
/// //     include_screen_outline: true,
/// //     include_default_camera: true,
/// // }));
///     
/// ```
pub struct Game {
    entity_manager: EntityManager,
    events_to_systems: HashMap<&'static str, Vec<System>>,
    is_playing: bool,
    options: GameOptions,
    frame_timer: Timer,
}
impl Game {
    pub fn new(options: GameOptions) -> Self {
        Self {
            entity_manager: EntityManager::new(),
            events_to_systems: HashMap::new(),
            is_playing: false,
            options,
            frame_timer: Timer::new(),
        }
    }

    /// Adds a system to the init event. The init event runs exactly **one** time during the life of the game. It runs
    /// before the main game loop starts. The init event is a good place to put any systems that will be used to
    /// initialize your game world.
    pub fn add_init_system(self, system: System) -> Self {
        self.add_system(EVENT_INIT, system)
    }

    /// Adds a system to the update event. The update event runs once every frame. This is where the bulk of your systems
    /// will be.
    pub fn add_update_system(self, system: System) -> Self {
        self.add_system(EVENT_UPDATE, system)
    }

    /// Adds a system to the cleanup event. The cleanup event runs exactly **one** time during the life of the game. It
    /// runs after the main game loop has ended. It's a good place to do anything you want to do when the game
    /// _successfully and properly_ exits. This could include cleaning up any system side effects, or perhaps saving
    /// the player's progress.
    pub fn add_cleanup_system(self, system: System) -> Self {
        self.add_system(EVENT_CLEANUP, system)
    }

    /// Adds a system to the specified event. While it's likely you'll mostly use the init, update, and cleanup events,
    /// this method can be useful if you have a reason to use the other events Thomas provides. All event names are
    /// constants and start with `EVENT_`.
    pub fn add_system(mut self, event_name: &'static str, system: System) -> Self {
        if !self.events_to_systems.contains_key(event_name) {
            self.events_to_systems.insert(event_name, vec![]);
        }

        if let Some(system_list) = self.events_to_systems.get_mut(event_name) {
            system_list.push(system);
        }

        self
    }

    /// Adds all systems specified by the `SystemsGenerator`. `SystemsGenerator`s are a great way to split collections of
    /// systems into units for organization. Thomas also includes some SystemsGenerators for you for engine features
    /// you have to opt into. An example is the `TerminalCollisionsSystemsGenerators`, which enables collision detection
    /// in the terminal when added.
    pub fn add_systems_from_generator(mut self, systems_generator: impl SystemsGenerator) -> Self {
        for (event_name, system) in systems_generator.generate() {
            self = self.add_system(event_name, system);
        }

        self
    }

    /// Starts the game. This is the last thing you should be calling on your game instance, as it starts the main game
    /// loop. The thread will spin in this method until the `GameCommand::Quit` command is issued.
    pub fn start(mut self, renderer: Renderer) {
        let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

        self = self.setup_renderer(renderer);
        self = self.setup_builtin_systems();

        self.sort_systems_by_priority();

        self.is_playing = true;

        self.trigger_event(EVENT_INIT, Rc::clone(&commands));

        while self.is_playing {
            self.frame_timer.restart();

            self.trigger_event(EVENT_BEFORE_UPDATE, Rc::clone(&commands));

            self.trigger_event(EVENT_UPDATE, Rc::clone(&commands));

            self.trigger_event(EVENT_AFTER_UPDATE, Rc::clone(&commands));

            self.wait_for_frame();
        }

        self.trigger_event(EVENT_CLEANUP, Rc::clone(&commands));
    }

    fn wait_for_frame(&self) {
        let minimum_frame_time = if self.options.max_frame_rate > 0 {
            1000 / self.options.max_frame_rate
        } else {
            0
        };

        while self.frame_timer.elapsed_millis() < minimum_frame_time as u128 {}
    }

    fn sort_systems_by_priority(&mut self) {
        for (_, system_list) in &mut self.events_to_systems {
            system_list.sort_by(|a, b| a.priority().cmp(&b.priority()))
        }
    }

    fn trigger_event(&mut self, event_name: &'static str, commands: GameCommandsArg) {
        if let Some(system_list) = self.events_to_systems.get(event_name) {
            for system in system_list {
                let queries_results = system
                    .queries()
                    .iter()
                    .map(|query| self.entity_manager.query(query))
                    .collect();

                system.operator()(queries_results, Rc::clone(&commands));
            }

            self.process_command_queue(commands);
        }
    }

    fn setup_renderer(mut self, renderer: Renderer) -> Self {
        match renderer {
            Renderer::Terminal(options) => {
                self.entity_manager
                    .add_entity(vec![Box::new(TerminalRendererState::new(options))]);

                return self.add_systems_from_generator(TerminalRendererSystemsGenerator::new());
            }
        }
    }

    fn setup_builtin_systems(mut self) -> Self {
        if self.options.press_escape_to_quit {
            self = self.add_update_system(System::new(
                vec![Query::new().has::<Input>()],
                |results, commands| {
                    if let [input_results, ..] = &results[..] {
                        let input = input_results.get_only::<Input>();

                        if input.is_key_down(&Keycode::Escape) {
                            commands.borrow_mut().issue(GameCommand::Quit);
                        }
                    }
                },
            ));
        }

        self.add_update_system(System::new(
            vec![Query::new().has::<Input>()],
            |results, commands| {
                if let [input_results, ..] = &results[..] {
                    let input = input_results.get_only::<Input>();

                    if input.is_chord_pressed_exclusively(&[&Keycode::LControl, &Keycode::C]) {
                        commands.borrow_mut().issue(GameCommand::Quit);
                    }
                }
            },
        ))
        .add_systems_from_generator(ServicesSystemsGenerator::new())
    }

    fn process_command_queue(&mut self, commands: GameCommandsArg) {
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
            }
        }
    }
}

pub enum GameCommand {
    Quit,
    AddEntity(Vec<Box<dyn Component>>),
    AddComponentsToEntity(Entity, Vec<Box<dyn Component>>),
    RemoveComponentFromEntity(Entity, &'static str),
    DestroyEntity(Entity),
}

pub struct GameCommandQueue {
    queue: Vec<GameCommand>,
}
impl GameCommandQueue {
    pub(crate) fn new() -> Self {
        Self { queue: vec![] }
    }

    /// Issues a command to the queue. Nothing changes in the game until the queue is processed. Because of this,
    /// you can think of the side effects of a command as being asynchronous--they won't happen immediately after
    /// you issue them.
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

            let mut game = Game::new(GameOptions {
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

            game.trigger_event(EVENT_1, Rc::new(RefCell::new(GameCommandQueue::new())));

            assert_eq!(COUNTER_1.fetch_add(0, Ordering::Relaxed), 2);
            assert_eq!(COUNTER_2.fetch_add(0, Ordering::Relaxed), 5);
        }

        #[test]
        fn systems_have_access_to_commands() {
            let mut game = Game::new(GameOptions {
                press_escape_to_quit: false,
                max_frame_rate: 5,
            })
            .add_system(
                EVENT_1,
                System::new(vec![], |_, commands| {
                    commands.borrow_mut().issue(GameCommand::Quit);

                    assert_eq!(commands.borrow().queue.len(), 1);
                }),
            );

            game.trigger_event(EVENT_1, Rc::new(RefCell::new(GameCommandQueue::new())));
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
