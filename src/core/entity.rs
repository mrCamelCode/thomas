use crate::core::data::Transform;
use std::{
    hash::{Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
};

fn get_id() -> usize {
    static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Representation of an entity in the game world. All objects that exist in the game world are Entities.
pub struct Entity {
    pub(crate) id: String,
    name: String,
    transform: Transform,
    is_destroyed: bool,
}
impl Entity {
    pub fn new(name: &str, transform: Transform) -> Self {
        Self::new_with_id(name, transform, &get_id().to_string())
    }

    /// Allows you to specify the ID of the new Entity. Note that Entity IDs **MUST BE UNIQUE**. There cannot be
    /// two Entities in the game at the same time with the same ID, otherwise one will overwrite the other.
    pub fn new_with_id(name: &str, transform: Transform, id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            transform,
            is_destroyed: false,
        }
    }

    pub(crate) fn destroy(&mut self) {
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
impl Clone for Entity {
    fn clone(&self) -> Self {
        Self {
            // Assign a unique clone ID to this new clone of the entity to avoid having it overlap
            // with the existing entity. This allows someone to make an instance of an entity that
            // acts like a prefab they can then clone to easily make more copies of that prefab.
            id: format!("{}-{}", self.id.clone(), get_id()),
            name: self.name.clone(),
            transform: self.transform.clone(),
            is_destroyed: self.is_destroyed.clone(),
        }
    }
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Entity {}

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
