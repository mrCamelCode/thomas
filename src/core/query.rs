use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

use crate::{Component, Entity, StoredComponent};

pub struct Query {
    allowed_components: Vec<ComponentQueryData>,
    forbidden_components: Vec<ComponentQueryData>,
}
impl Query {
    pub fn new() -> Self {
        Self {
            allowed_components: vec![],
            forbidden_components: vec![],
        }
    }

    pub fn has<T: Component>(mut self) -> Self {
        self.allowed_components
            .push(ComponentQueryData::new(T::name()));

        self
    }

    pub fn has_no<T: Component>(mut self) -> Self {
        self.forbidden_components
            .push(ComponentQueryData::new(T::name()));

        self
    }

    // TODO: Figure out how to implement.
    // pub fn has_where<T: Component>(&mut self, predicate: fn(&T) -> bool) {
    //     self.components.push(ComponentQueryData::new(T::name(), predicate));
    // }

    pub fn components(&self) -> &Vec<ComponentQueryData> {
        &self.allowed_components
    }

    pub fn forbidden_components(&self) -> &Vec<ComponentQueryData> {
        &self.forbidden_components
    }

    pub fn component_names(&self) -> Vec<&'static str> {
        self.allowed_components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }

    pub fn forbidden_component_names(&self) -> Vec<&'static str> {
        self.forbidden_components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }
}

pub struct QueryResult {
    pub(crate) entity: Entity,
    pub(crate) components: Vec<StoredComponent>,
}
impl QueryResult {
    pub fn try_get<T>(&self) -> Option<Ref<T>>
    where
        T: Component + 'static,
    {
        for component in &self.components {
            if (**component.borrow()).as_any().is::<T>() {
                return Some(Ref::map(component.borrow(), |component| {
                    (**component).as_any().downcast_ref::<T>().unwrap()
                }));
            }
        }

        None
    }

    pub fn try_get_mut<T>(&self) -> Option<RefMut<T>>
    where
        T: Component + 'static,
    {
        for component in &self.components {
            if (**component.borrow()).as_any().is::<T>() {
                return Some(RefMut::map(component.borrow_mut(), |component| {
                    (**component).as_any_mut().downcast_mut::<T>().unwrap()
                }));
            }
        }

        None
    }

    pub fn get<T>(&self) -> Ref<T>
    where
        T: Component + 'static,
    {
        if let Some(component) = self.try_get::<T>() {
            return component;
        }

        panic!(
            "Component {} was not present on Entity {}.",
            T::name(),
            *self.entity
        );
    }

    pub fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Component + 'static,
    {
        if let Some(component) = self.try_get_mut::<T>() {
            return component;
        }

        panic!(
            "Component {} was not present on Entity {}",
            T::name(),
            *self.entity
        );
    }
}

pub struct QueryResultList {
    list: Vec<QueryResult>,
}
impl QueryResultList {
    pub fn new(results: Vec<QueryResult>) -> Self {
        Self { list: results }
    }
}
impl Deref for QueryResultList {
    type Target = Vec<QueryResult>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

pub struct ComponentQueryData {
    component_name: &'static str,
}
impl ComponentQueryData {
    fn new(component_name: &'static str) -> Self {
        Self { component_name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Component)]
    struct TestComponent {
        prop: String,
    }

    #[derive(Component)]
    struct EmptyComponent {}

    #[derive(Component)]
    struct AnotherEmptyComponent {}

    mod test_try_get {

        use super::*;

        #[test]
        fn gives_back_component_when_it_is_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(Box::new(TestComponent {
                    prop: "val".to_string(),
                })
                    as Box<dyn Component>))],
            };

            let test_component = qr.try_get::<TestComponent>().unwrap();

            assert_eq!(test_component.prop, "val");
        }

        #[test]
        fn is_none_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(
                    Box::new(AnotherEmptyComponent {}) as Box<dyn Component>
                ))],
            };

            let empty_component_option = qr.try_get::<EmptyComponent>();

            assert!(empty_component_option.is_none());
        }
    }

    mod test_try_get_mut {
        use super::*;

        #[test]
        fn can_mutate_returned_component() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(Box::new(TestComponent {
                    prop: "val".to_string(),
                })
                    as Box<dyn Component>))],
            };

            let mut test_component = qr.try_get_mut::<TestComponent>().unwrap();

            assert_eq!(test_component.prop, "val");

            let new_prop = String::from("now for something totally different");

            test_component.prop = new_prop.clone();

            assert_eq!(test_component.prop, new_prop);
        }

        #[test]
        fn is_none_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(
                    Box::new(AnotherEmptyComponent {}) as Box<dyn Component>
                ))],
            };

            let empty_component_option = qr.try_get_mut::<EmptyComponent>();

            assert!(empty_component_option.is_none());
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn gives_back_component_when_it_is_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(Box::new(TestComponent {
                    prop: "val".to_string(),
                })
                    as Box<dyn Component>))],
            };

            let test_component = qr.get::<TestComponent>();

            assert_eq!(test_component.prop, "val");
        }

        #[test]
        #[should_panic(expected = "Component EmptyComponent was not present on Entity 0")]
        fn panics_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(
                    Box::new(AnotherEmptyComponent {}) as Box<dyn Component>
                ))],
            };

            qr.get::<EmptyComponent>();
        }
    }

    mod test_get_mut {
        use super::*;

        #[test]
        fn can_mutate_returned_component() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(Box::new(TestComponent {
                    prop: "val".to_string(),
                })
                    as Box<dyn Component>))],
            };

            let mut test_component = qr.get_mut::<TestComponent>();

            assert_eq!(test_component.prop, "val");

            let new_prop = String::from("now for something totally different");

            test_component.prop = new_prop.clone();

            assert_eq!(test_component.prop, new_prop);
        }

        #[test]
        #[should_panic(expected = "Component EmptyComponent was not present on Entity 0")]
        fn panics_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(
                    Box::new(AnotherEmptyComponent {}) as Box<dyn Component>
                ))],
            };

            qr.get_mut::<EmptyComponent>();
        }
    }
}
