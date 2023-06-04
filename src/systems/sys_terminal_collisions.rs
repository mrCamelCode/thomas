use std::{cell::Ref, collections::HashMap};

use crate::{
    Entity, GameCommand, IntCoords2d, Query, QueryResultList, System, SystemExtraArgs,
    SystemsGenerator, TerminalCollider, TerminalCollision, TerminalTransform, EVENT_AFTER_UPDATE,
    EVENT_BEFORE_UPDATE,
};

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

fn detect_collisions(results: Vec<QueryResultList>, util: &SystemExtraArgs) {
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
                        util.commands().issue(GameCommand::AddEntity(vec![Box::new(
                            TerminalCollision {
                                bodies: [(**other_entity, **other_collider), (*entity, *collider)],
                            },
                        )]));
                    }

                    entity_list.push((entity, collider));
                }
            } else {
                used_coords.insert(hash_string, vec![(entity, collider)]);
            }
        }
    }
}

fn cleanup_collisions(results: Vec<QueryResultList>, util: &SystemExtraArgs) {
    if let [collision_query, ..] = &results[..] {
        for collision_result in collision_query {
            util.commands()
                .issue(GameCommand::DestroyEntity(*collision_result.entity()));
        }
    }
}

fn get_coords_hash_string(coords: &IntCoords2d) -> String {
    format!("{},{}", coords.x(), coords.y())
}
