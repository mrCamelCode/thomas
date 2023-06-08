use std::{cell::Ref, collections::HashMap};

use crate::{
    Entity, GameCommand, GameCommandsArg, IntCoords2d, Query, QueryResultList, System,
    SystemsGenerator, TerminalCollider, TerminalCollision, TerminalTransform, EVENT_AFTER_UPDATE,
    EVENT_BEFORE_UPDATE,
};

/// A generator responsible for setting up and performing collision detection between active `TerminalCollider`s in
/// the world. This must be added to the world for collisions to be generated.
/// 
/// As it's impossible for Thomas to know exactly what you want to do when two bodies collide, you'll need to implement
/// your own collision processing systems. When a collision occurs, an entity with a `TerminalCollision` component is added
/// to the world. Collision processing systems can query for that component in the update event to act on collisions that
/// were generated that frame. In the after-update event, all existing collisions are cleaned up.
pub struct TerminalCollisionsSystemsGenerator {}
impl TerminalCollisionsSystemsGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl SystemsGenerator for TerminalCollisionsSystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)> {
        vec![
            (
                EVENT_BEFORE_UPDATE,
                System::new(
                    vec![Query::new()
                        .has_where::<TerminalCollider>(|collider| collider.is_active)
                        .has::<TerminalTransform>()],
                    detect_collisions,
                ),
            ),
            (
                EVENT_AFTER_UPDATE,
                System::new(
                    vec![Query::new().has::<TerminalCollision>()],
                    cleanup_collisions,
                ),
            ),
        ]
    }
}

fn detect_collisions(results: Vec<QueryResultList>, commands: GameCommandsArg) {
    if let [bodies_query, ..] = &results[..] {
        let mut used_coords: HashMap<String, Vec<(&Entity, Ref<TerminalCollider>)>> =
            HashMap::new();

        for body in bodies_query {
            let collider = body.components().get::<TerminalCollider>();
            let coords = body.components().get::<TerminalTransform>().coords;
            let entity = body.entity();
            let hash_string = get_coords_hash_string(&coords);

            if let Some(entity_list) = used_coords.get_mut(&hash_string) {
                if !entity_list.is_empty() {
                    for (other_entity, other_collider) in &mut *entity_list {
                        commands
                            .borrow_mut()
                            .issue(GameCommand::AddEntity(vec![Box::new(TerminalCollision {
                                bodies: [(**other_entity, **other_collider), (*entity, *collider)],
                            })]));
                    }

                    entity_list.push((entity, collider));
                }
            } else {
                used_coords.insert(hash_string, vec![(entity, collider)]);
            }
        }
    }
}

fn cleanup_collisions(results: Vec<QueryResultList>, commands: GameCommandsArg) {
    if let [collision_query, ..] = &results[..] {
        for collision_result in collision_query {
            commands
                .borrow_mut()
                .issue(GameCommand::DestroyEntity(*collision_result.entity()));
        }
    }
}

fn get_coords_hash_string(coords: &IntCoords2d) -> String {
    format!("{},{}", coords.x(), coords.y())
}
