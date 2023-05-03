use std::any::Any;

pub trait Component {
    fn name() -> &'static str
    where
        Self: Sized;
    fn is_component_type(comp: &Box<dyn Component>) -> bool
    where
        Self: Sized;
    fn coerce(comp: &Box<dyn Component>) -> Option<&Self>
    where
        Self: Sized;
    fn coerce_mut(comp: &mut Box<dyn Component>) -> Option<&mut Self>
    where
        Self: Sized;

    fn component_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
