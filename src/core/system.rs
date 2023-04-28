use crate::Component;

pub type System = dyn Fn(Vec<&dyn Component>) -> ();
pub type SystemMut = dyn FnMut(Vec<&mut dyn Component>) -> ();
