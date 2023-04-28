use crate::Component;

pub struct Query {}
impl Query {
    pub fn new() -> Self {
        Self {}
    }

    pub fn for_entities(&mut self) {}

    pub fn has<T: Component>(&mut self) {}

    pub fn has_where<T: Component>(&mut self, predicate: impl Fn(T) -> bool) {}

    pub fn and(&mut self) {}
}
