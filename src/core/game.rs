pub struct Game {}

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
