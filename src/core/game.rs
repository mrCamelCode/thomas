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
    command_queue: GameCommandQueue,
    game_services: GameServices,
}
impl Game {
    pub fn new() -> Game {
        Game {
            entity_behaviour_map: EntityBehaviourMap::new(),
            command_queue: GameCommandQueue::new(),
            game_services: GameServices::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity, behaviours: BehaviourList) {}

    pub fn start(&mut self, renderer: &dyn Renderer) {
        loop {
            self.game_services.input.update();
            self.game_services.time.update();

            self.entity_behaviour_map
                .update(&self.game_services, &mut self.command_queue);

            renderer.render(
                self.entity_behaviour_map
                    .entries()
                    .collect::<Vec<(&Entity, &BehaviourList)>>(),
            );

            self.command_queue.handle();
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

    pub(crate) fn handle(&self) {
        todo!("Maybe handle by giving back a consuming iterator?");
        // Reverse because pushes put things at the end of the vector and Queues should work first-in, first-out.
        // self.queue.into_iter().rev()
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
