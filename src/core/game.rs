use std::{any::Any, thread, time::Duration};

use device_query::Keycode;

use super::{
    message::Message, renderer::Renderer, BehaviourList, Entity, Input, Time, Timer, World,
};

pub struct GameServices {
    input: Input,
    time: Time,
}
impl GameServices {
    pub fn new() -> Self {
        GameServices {
            input: Input::new(),
            time: Time::new(),
        }
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn time(&self) -> &Time {
        &self.time
    }
}

pub struct GameConfig {
    pub press_escape_to_quit: bool,
    /// The maximum times per second the game's main loop is allowed to run. A value of 0 indicates an uncapped framerate.
    pub max_frame_rate: u16,
}

pub struct Game {
    world: World,
    game_services: GameServices,
    should_quit: bool,
    config: GameConfig,
}
impl Game {
    pub fn new(config: GameConfig) -> Game {
        Game {
            world: World::new(),
            game_services: GameServices::new(),
            should_quit: false,
            config,
        }
    }

    pub fn add_entity(&mut self, entity: Entity, behaviours: BehaviourList) {
        self.world.add(entity, behaviours);
    }

    pub fn start(&mut self, renderer: &mut dyn Renderer) {
        renderer.init();

        let mut frame_timer = Timer::new();

        let minimum_frame_time = if self.config.max_frame_rate > 0 {
            1000 / self.config.max_frame_rate
        } else {
            0
        };

        loop {
            frame_timer.restart();

            let mut command_queue = GameCommandQueue::new();

            self.game_services.input.update();
            self.game_services.time.update();

            self.world
                .update(&self.game_services, &mut command_queue, &self.world.clone());

            if let Err(err) = renderer.render(
                self.world
                    .entities()
                    .collect::<Vec<(&Entity, &BehaviourList)>>(),
            ) {
                panic!("Error occurred during render. Source error: {err}");
            }

            self.handle_command_queue(command_queue);

            if self.config.press_escape_to_quit
                && self.game_services.input.is_key_pressed(&Keycode::Escape)
            {
                self.should_quit = true;
            }

            if self.should_quit {
                break;
            }

            let elapsed_millis = frame_timer.elapsed_millis();
            if elapsed_millis < minimum_frame_time as u128 {
                thread::sleep(Duration::from_millis(
                    (minimum_frame_time as u128 - elapsed_millis) as u64,
                ));
            }
        }

        if let Err(err) = renderer.cleanup() {
            panic!("Error occurred during renderer cleanup. The environment may still be in a dirty state. Source error: {err}");
        }
    }

    fn handle_command_queue(&mut self, command_queue: GameCommandQueue) {
        for command in command_queue.consume() {
            match command {
                GameCommand::AddEntity { entity, behaviours } => {
                    self.add_entity(entity, behaviours);
                }
                GameCommand::DestroyEntity(entity_id) => {
                    if let Some((entity, _)) = self.world.get_entity_mut(&entity_id) {
                        entity.destroy();
                    }
                }
                GameCommand::ClearEntities => {
                    self.world = World::new();
                }
                GameCommand::SendMessage { entity_id, message } => {
                    if let Some((_, behaviours)) = self.world.get_entity_mut(&entity_id) {
                        for behaviour in behaviours.iter_mut() {
                            behaviour.on_message(&message);
                        }
                    }
                }
                GameCommand::Quit => self.should_quit = true,
            }
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
    Quit,
    ClearEntities,
    AddEntity {
        entity: Entity,
        behaviours: BehaviourList,
    },
    DestroyEntity(String),
    SendMessage {
        entity_id: String,
        message: Message<Box<dyn Any>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod game_command_queue {
        use crate::core::data::Transform;

        use super::*;

        #[test]
        fn goes_through_commands_in_the_order_they_were_issued() {
            let mut queue = GameCommandQueue::new();

            queue.issue(GameCommand::Quit);
            queue.issue(GameCommand::ClearEntities);
            queue.issue(GameCommand::AddEntity {
                entity: Entity::new("test", Transform::default()),
                behaviours: BehaviourList::new(),
            });

            let mut iter = queue.consume();

            assert!(match iter.next().unwrap() {
                GameCommand::Quit => true,
                _ => false,
            });
            assert!(match iter.next().unwrap() {
                GameCommand::ClearEntities => true,
                _ => false,
            });
            assert!(match iter.next().unwrap() {
                GameCommand::AddEntity { .. } => true,
                _ => false,
            });
            assert!(iter.next().is_none());
        }
    }
}
