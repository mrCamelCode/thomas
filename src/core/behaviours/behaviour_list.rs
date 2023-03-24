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
    pub fn new() -> Self {
        BehaviourList { behaviours: vec![] }
    }

    pub fn from(behaviours: Vec<Box<dyn CustomBehaviour>>) -> Self {
        BehaviourList { behaviours }
    }

    /// Gets the `CustomBehaviour` by name. The `CustomBehaviour`'s name is equal to
    /// your behaviour's name. If there was no `CustomBehaviour` by that name in the
    /// list, `None` will be returned.
    ///
    /// # Examples
    /// ```
    /// use thomas::core::{BehaviourList, CustomBehaviour, Behaviour};
    /// use thomas_derive::*;
    ///
    /// #[derive(Behaviour)]
    /// struct MyBehaviour;
    /// impl MyBehaviour {
    ///     fn new() -> Self {
    ///         MyBehaviour { }
    ///     }
    /// }
    /// impl CustomBehaviour for MyBehaviour {}
    ///
    /// let list = BehaviourList::from(vec![Box::new(MyBehaviour::new())]);
    ///
    /// let my_behaviour: &MyBehaviour = list.get("MyBehaviour").unwrap();
    /// ```
    pub fn get<T>(&self, name: &str) -> Option<&T>
    where
        T: CustomBehaviour + 'static,
    {
        if let Some(behaviour) = self.behaviours.iter().find(|b| b.name() == name) {
            return behaviour.as_any().downcast_ref::<T>();
        }

        None
    }

    pub fn add(&mut self, behaviour: Box<dyn CustomBehaviour>) -> Result<(), ()> {
        if !self.behaviours.iter().any(|b| b.name() == behaviour.name()) {
            self.behaviours.push(behaviour);

            return Ok(());
        }

        Err(())
    }

    pub fn remove(&mut self, behaviour_name: &str) {
        self.behaviours.retain(|b| b.name() != behaviour_name);
    }

    pub fn iter(&self) -> BehaviourListIter {
        BehaviourListIter {
            values: &self.behaviours,
            index: 0,
        }
    }

    pub fn as_mut(&mut self) -> &mut Self {
        self
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
