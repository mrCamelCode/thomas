use std::any::Any;

/// A `Component` is essentially a data bucket and is one of the core aspects of ECS. A `Component` houses _no_ logic and
/// is simply meant to be a repository for related data. In Thomas, you should never be implementing the `Component` trait
/// directly, but rather deriving it to create your own custom components:
/// ```
/// use thomas::Component;
/// 
/// #[derive(Component)]
/// pub struct Player {
///     pub is_main_player: bool,
/// }
/// ```
/// In general, `Component`s should largely be open so that they can be mutated by `System`s. However, your design is
/// ultimately up to you. You may find it useful to hide some details of a complex component to force controlled mutation
/// facilitated by a method on your custom component.
pub trait Component {
    fn name() -> &'static str
    where
        Self: Sized;
    fn is_component_type(comp: &dyn Component) -> bool
    where
        Self: Sized;
    fn cast(comp: &dyn Component) -> Option<&Self>
    where
        Self: Sized;
    fn cast_mut(comp: &mut dyn Component) -> Option<&mut Self>
    where
        Self: Sized;

    fn component_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
