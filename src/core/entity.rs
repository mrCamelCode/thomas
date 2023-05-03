use std::{
    ops::Deref,
    sync::atomic::{AtomicU64, AtomicUsize},
};

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Entity(u64);
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
