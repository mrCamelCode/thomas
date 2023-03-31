use crate::core::data::Transform;
use std::sync::atomic::{AtomicUsize, Ordering};

fn get_id() -> usize {
    static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Representation of an entity in the game world. All objects that exist in the game world are Entities.
pub struct Entity {
    id: String,
    name: String,
    transform: Transform,
    is_destroyed: bool,
}

impl Entity {
    pub fn new(name: &str, transform: Transform) -> Self {
        Entity {
            id: get_id().to_string(),
            name: name.to_string(),
            transform,
            is_destroyed: false,
        }
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }
    pub fn transform_mut(&mut self) -> &mut Transform {
        self.transform.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod id_generation {
        use super::*;

        #[test]
        fn unique_id_made_for_each_entity() {
            let [e1, e2, e3] = [
                Entity::new("e1", Transform::default()),
                Entity::new("e2", Transform::default()),
                Entity::new("e3", Transform::default()),
            ];

            assert_ne!(e1.id, e2.id);
            assert_ne!(e1.id, e3.id);
            assert_ne!(e2.id, e3.id);
        }
    }
}
