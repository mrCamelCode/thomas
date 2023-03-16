use super::Behaviour;

pub struct BehaviourList {
    behaviours: Vec<Box<dyn Behaviour>>,
}

pub struct BehaviourListIter<'a> {
    values: &'a Vec<Box<dyn Behaviour>>,
    index: usize,
}

impl BehaviourList {
    pub fn new() -> Self {
        BehaviourList { behaviours: vec![] }
    }

    pub fn add(&mut self, behaviour: &dyn Behaviour) -> &mut Self {
        todo!();

        self
    }

    pub fn remove(&self, behaviour: &dyn Behaviour) {
        todo!();
    }
}

impl<'a> Iterator for BehaviourListIter<'a> {
    type Item = &'a Box<dyn Behaviour>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.values.len() {
            return None;
        }

        self.index += 1;

        Some(&self.values[self.index - 1])
    }
}
