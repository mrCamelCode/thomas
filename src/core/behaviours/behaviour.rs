use crate::core::GameUtil;

/// A Behaviour is the basis for any logic you want to attach to an Entity. The majority of the logic of your game
/// will be performed by Behaviours attached to your various Entities.
pub trait Behaviour {
  /// Invoked once.
  fn init(&self) {}

  /// Invoked on every frame.
  fn update(&self, _utils: &GameUtil) {}

  fn characteristics(&mut self) -> &mut BehaviourCharacteristics;

  // fn characteristics_as_mut(&mut self) -> &mut BehaviourCharacteristics;
}

pub struct BehaviourCharacteristics {
  name: String,
  is_init: bool,
}

pub(crate) fn do_behaviour_init(behaviour: &mut dyn Behaviour) {
  if !behaviour.characteristics().is_init {
    behaviour.init();

    behaviour.characteristics().is_init = true;  
  }
}
