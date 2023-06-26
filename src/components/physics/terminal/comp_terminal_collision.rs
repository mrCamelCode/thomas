use crate::{Component, Entity, Layer, TerminalCollider};

pub type TerminalCollisionBody = (Entity, TerminalCollider);

/// Represents a collision between two `TerminalCollider`s. Also provides the entities that collided.
#[derive(Component)]
pub struct TerminalCollision {
    pub bodies: [TerminalCollisionBody; 2],
}
impl TerminalCollision {
    pub fn is_collision_between(&self, collision_layer1: Layer, collision_layer2: Layer) -> bool {
        let first_body_option = self
            .bodies
            .iter()
            .find(|(_, collider)| collider.layer == collision_layer1);

        first_body_option.is_some()
            && self.bodies.iter().any(|(entity, collider)| {
                collider.layer == collision_layer2 && *entity != first_body_option.unwrap().0
            })
    }

    /// Returns the first body that's on the specified layer. Note that this will give the _first_
    /// match. You may find this method less useful when processing a collision between two things on the same collision
    /// layer.
    pub fn get_body_on_layer(&self, collision_layer: Layer) -> Option<&TerminalCollisionBody> {
        self.bodies
            .iter()
            .find(|(_, collider)| collider.layer == collision_layer)
    }

    /// Returns the entity of the first collision body that's on the specified layer. Note that this will give the _first_
    /// match. You may find this method less useful when processing a collision between two things on the same collision
    /// layer.
    pub fn get_entity_on_layer(&self, collision_layer: Layer) -> Option<Entity> {
        if let Some((entity, _)) = self.get_body_on_layer(collision_layer) {
            Some(*entity)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_is_collision_between {
        use super::*;

        #[test]
        fn is_true_when_both_layers_are_present() {
            let collision = TerminalCollision {
                bodies: [
                    (
                        Entity(0),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(0),
                        },
                    ),
                    (
                        Entity(1),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(1),
                        },
                    ),
                ],
            };

            assert!(collision.is_collision_between(Layer(0), Layer(1)));
        }

        #[test]
        fn is_true_when_checking_for_collision_on_same_layer_and_both_bodies_have_the_correct_layer(
        ) {
            let collision = TerminalCollision {
                bodies: [
                    (
                        Entity(0),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(0),
                        },
                    ),
                    (
                        Entity(1),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(0),
                        },
                    ),
                ],
            };

            assert!(collision.is_collision_between(Layer(0), Layer(0)));
        }

        #[test]
        fn is_false_when_checking_for_collision_on_same_layer_and_only_one_body_has_the_correct_layer(
        ) {
            let collision = TerminalCollision {
                bodies: [
                    (
                        Entity(0),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(0),
                        },
                    ),
                    (
                        Entity(1),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(1),
                        },
                    ),
                ],
            };

            assert!(!collision.is_collision_between(Layer(0), Layer(0)));
        }

        #[test]
        fn is_false_when_only_one_layer_is_present() {
            let collision = TerminalCollision {
                bodies: [
                    (
                        Entity(0),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(0),
                        },
                    ),
                    (
                        Entity(1),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(2),
                        },
                    ),
                ],
            };

            assert!(!collision.is_collision_between(Layer(0), Layer(1)));
        }

        #[test]
        fn is_false_when_both_layers_are_absent() {
            let collision = TerminalCollision {
                bodies: [
                    (
                        Entity(0),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(3),
                        },
                    ),
                    (
                        Entity(1),
                        TerminalCollider {
                            is_active: true,
                            layer: Layer(2),
                        },
                    ),
                ],
            };

            assert!(!collision.is_collision_between(Layer(0), Layer(1)));
        }
    }
}
