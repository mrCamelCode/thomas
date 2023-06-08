use std::{ops::Deref, sync::atomic::AtomicU64};

/// An `Entity` represents a thing in your game world and is one of the core aspects of ECS. Functionally, 
/// you can think of an `Entity` as its ID. Entities are associated with `Component`s to define what data that `Entity`
/// has. Though it's likely you'll use `Entity` references provided to you, you should never be creating an `Entity` yourself.
/// 
/// `Entity` ID generation happens automatically for you. When an `Entity` is removed from the world, its ID is recycled.
/// For the purposes of a user of Thomas, you can largely ignore an `Entity`'s exact ID. In fact, you shouldn't be trying
/// to hold onto them in an effort to single out a particular `Entity` in the game world. If you need to always be able
/// to single out a particular `Entity` for use in one of your systems, consider using a custom `Component` attached to that
/// `Entity` that's unique to that `Entity`, or use the `Identity` component to give that `Entity` identifiers that are meaningful 
/// for your game.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Entity(pub(crate) u64);
impl Entity {
    pub(crate) fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self(id)
    }
}
impl Deref for Entity {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn new_entities_have_different_ids() {
            let e1 = Entity::new();
            let e2 = Entity::new();

            assert_ne!(e1.0, e2.0);
        }
    }
}
