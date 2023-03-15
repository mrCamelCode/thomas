use super::Behaviour;

pub struct BehaviourList {
    behaviours: Vec<Box<dyn Behaviour>>,
}

// TODO: Implement Iterator so you can easily iterate over the behaviours in the list.
impl BehaviourList {
    pub fn new(behaviours: Vec<Box<dyn Behaviour>>) -> Self {
        BehaviourList { behaviours }
    }

    /// Provides the default Behaviour List, which is empty.
    pub fn default() -> Self {
        BehaviourList { behaviours: vec![] }
    }

    // TODO: Implement and don't allow a behaviour to be added if there's already a behaviour of that type in the list.
    pub fn add(&mut self, behaviour: Box<dyn Behaviour>) -> Result<(), ()> {
      Ok(())
    }

    pub fn remove(&self) {

    }
}
