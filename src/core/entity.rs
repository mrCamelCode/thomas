use std::{
    ops::Deref,
    sync::atomic::{AtomicU64},
};

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Entity(pub(crate) u64);
impl Entity {
    pub fn new() -> Self {
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
