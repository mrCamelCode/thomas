use super::CustomBehaviour;
use std::collections::{
    hash_map::{Values, ValuesMut},
    HashMap,
};

pub struct BehaviourList {
    behaviours: HashMap<String, Box<dyn CustomBehaviour>>,
}

impl BehaviourList {
    pub fn new() -> Self {
        BehaviourList {
            behaviours: HashMap::new(),
        }
    }

    pub fn from(behaviours: Vec<Box<dyn CustomBehaviour>>) -> Self {
        let mut behaviours_map = HashMap::new();

        behaviours.into_iter().for_each(|behaviour| {
            behaviours_map.insert(behaviour.name().to_string(), behaviour);
        });

        BehaviourList {
            behaviours: behaviours_map,
        }
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
        if let Some(behaviour) = self.behaviours.get(name) {
            return behaviour.as_any().downcast_ref::<T>();
        }

        None
    }

    pub fn add(&mut self, behaviour: Box<dyn CustomBehaviour>) -> Result<(), ()> {
        if !self.behaviours.contains_key(behaviour.name()) {
            self.behaviours
                .insert(behaviour.name().to_string(), behaviour);

            return Ok(());
        }

        Err(())
    }

    pub fn remove(&mut self, behaviour_name: &str) {
        self.behaviours.remove(behaviour_name);
    }

    pub fn iter(&self) -> Values<'_, String, Box<dyn CustomBehaviour>> {
        self.behaviours.values()
    }

    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    pub(crate) fn iter_mut(&mut self) -> ValuesMut<'_, String, Box<dyn CustomBehaviour>> {
        self.behaviours.values_mut()
    }
}
