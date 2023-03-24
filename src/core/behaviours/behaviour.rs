use std::any::Any;

use crate::core::GameUtil;

pub trait Behaviour {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

/// A Behaviour is the basis for any logic you want to attach to an Entity. The majority of the logic of your game
/// will be performed by Behaviours attached to your various Entities.
pub trait CustomBehaviour: Behaviour {
    /// Invoked once.
    fn init(&mut self) {}

    /// Invoked on every frame.
    fn update(&mut self, _utils: &GameUtil) {}
}
