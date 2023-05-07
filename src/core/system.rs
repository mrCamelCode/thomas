use crate::{Component, Query};

// pub type System = dyn Fn(Vec<&dyn Component>) -> ();
// pub type SystemMut = dyn FnMut(Vec<&mut dyn Component>) -> ();

pub struct System<T> {
  query: Query,
  operator: dyn Fn(dyn Iterator<Item = T>) -> (),
}
