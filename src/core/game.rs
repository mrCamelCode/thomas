use device_query::Keycode;

use super::{renderer::Renderer, BehaviourList, Entity, EntityBehaviourMap, Input, Time};

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

pub struct Game {
    entity_behaviour_map: EntityBehaviourMap,
    game_services: GameServices,
}
impl Game {
    pub fn new() -> Game {
        Game {
            entity_behaviour_map: EntityBehaviourMap::new(),
            game_services: GameServices::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity, behaviours: BehaviourList) {
        self.entity_behaviour_map.add(entity, behaviours);
    }

    pub fn start(&mut self, renderer: &dyn Renderer) {
        renderer.init();

        'main_game_loop: loop {
            let mut command_queue = GameCommandQueue::new();

            self.game_services.input.update();
            self.game_services.time.update();

            if self.game_services.input.is_key_pressed(&Keycode::Escape) {
                break;
            }

            self.entity_behaviour_map.update(
                &self.game_services,
                &mut command_queue,
                &self.entity_behaviour_map.clone()
            );

            renderer.render(
                self.entity_behaviour_map
                    .entries()
                    .collect::<Vec<(&Entity, &BehaviourList)>>(),
            );

            for command in command_queue.consume() {
                match command {
                    GameCommand::AddEntity { entity, behaviours } => {
                        self.add_entity(entity, behaviours);
                    }
                    GameCommand::ClearEntities => {
                        self.entity_behaviour_map = EntityBehaviourMap::new();
                    }
                    GameCommand::Quit => break 'main_game_loop,
                }
            }
        }

        renderer.cleanup();
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
