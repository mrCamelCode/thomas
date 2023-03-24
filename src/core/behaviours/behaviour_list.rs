use super::CustomBehaviour;
use std::slice::IterMut;
pub struct BehaviourList {
    behaviours: Vec<Box<dyn CustomBehaviour>>,
}

pub struct BehaviourListIter<'a> {
    values: &'a Vec<Box<dyn CustomBehaviour>>,
    index: usize,
}

impl BehaviourList {
    pub fn new(behaviours: Vec<&dyn CustomBehaviour>) -> Self {
        BehaviourList {
            behaviours: behaviours.into_iter().map(|b| Box::new(*b)).collect(),
        }
    }

    pub fn add(&mut self, behaviour: Box<dyn CustomBehaviour>) -> Result<(), ()> {
        if !self.behaviours.iter().any(|b| b.name() == behaviour.name()) {
            self.behaviours.push(behaviour);

            return Ok(());
        }

        Err(())
    }

    pub fn remove(&self, behaviour: Box<dyn CustomBehaviour>) {
        todo!();
    }

    pub fn iter(&self) -> BehaviourListIter {
        BehaviourListIter {
            values: &self.behaviours,
            index: 0,
        }
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, Box<dyn CustomBehaviour>> {
        self.behaviours.iter_mut()
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
