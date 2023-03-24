use crate::core::GameUtil;

pub trait Behaviour {
    fn name(&self) -> &'static str;
}

/// A Behaviour is the basis for any logic you want to attach to an Entity. The majority of the logic of your game
/// will be performed by Behaviours attached to your various Entities.
pub trait CustomBehaviour: Behaviour {
    /// Invoked once.
    fn init(&self) {}

    /// Invoked on every frame.
    fn update(&self, _utils: &GameUtil) {}
}
