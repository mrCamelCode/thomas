use super::CustomBehaviour;
pub struct BehaviourList {
    behaviours: Vec<Box<dyn CustomBehaviour>>,
}

pub struct BehaviourListIter<'a> {
    values: &'a Vec<Box<dyn CustomBehaviour>>,
    index: usize,
}

impl BehaviourList {
    pub fn new() -> Self {
        BehaviourList { behaviours: vec![] }
    }

    pub fn add(&mut self, behaviour: Box<dyn CustomBehaviour>) -> Result<(), ()> {
        if !self.behaviours.iter().any(|b| b.name() == behaviour.name()) {
            self.behaviours.push(behaviour);

            return Ok(());
        }
        
        Err(())
    }

    pub fn remove(&self, behaviour: &dyn CustomBehaviour) {
        todo!();
    }

    pub fn iter(&self) -> BehaviourListIter {
        BehaviourListIter {
            values: &self.behaviours,
            index: 0,
        }
    }
}

impl<'a> Iterator for BehaviourListIter<'a> {
    type Item = &'a Box<dyn CustomBehaviour>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.values.len() {
            return None;
        }

        self.index += 1;

        Some(&self.values[self.index - 1])
    }
}
