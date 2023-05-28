use std::{
    array::IntoIter,
    cell::{Ref, RefMut},
    ops::Deref,
};

use crate::{Component, Entity, StoredComponent, StoredComponentList};

pub type WherePredicate = dyn Fn(&dyn Component) -> bool + 'static;

pub struct Query {
    allowed_components: Vec<ComponentQueryData>,
    forbidden_components: Vec<ComponentQueryData>,
    included_components: Vec<ComponentQueryData>,
}
impl Query {
    pub fn new() -> Self {
        Self {
            allowed_components: vec![],
            forbidden_components: vec![],
            included_components: vec![],
        }
    }

    pub fn has<T: Component + 'static>(mut self) -> Self {
        self.allowed_components
            .push(ComponentQueryData::new(T::name(), None));

        self
    }

    pub fn has_no<T: Component + 'static>(mut self) -> Self {
        self.forbidden_components
            .push(ComponentQueryData::new(T::name(), None));

        self
    }

    /// Whether to always include matches with the specified component. An included component
    /// is included regardless of any constraints established in the query by
    /// other functions like `has`, `has_no`, etc.
    ///
    /// # Example
    /// ```
    /// use thomas::{Query, Component};
    ///
    /// #[derive(Component)]
    /// struct Comp1 {}
    ///
    /// #[derive(Component)]
    /// struct Comp2 {}
    ///
    /// #[derive(Component)]
    /// struct Comp3 {}
    ///
    /// // Assuming a world with entities:
    /// // 0: Comp1, Comp2
    /// // 1: Comp1
    /// // 2: Comp2
    /// // 3: Comp2, Comp3
    ///
    /// Query::new()
    ///     .has::<Comp1>()
    ///     .has_no::<Comp2>()
    ///     .include::<Comp3>();
    ///
    /// // => Query would produce results of:
    /// // matches: [
    /// //   Entity(1): { components: [Comp1] }
    /// // ],
    /// // inclusions: [
    /// //   Entity(3): { components: [Comp3] }
    /// // ]
    /// ```
    pub fn include<T: Component + 'static>(mut self) -> Self {
        self.included_components
            .push(ComponentQueryData::new(T::name(), None));

        self
    }

    pub fn has_where<T>(mut self, predicate: impl Fn(&T) -> bool + 'static) -> Self
    where
        T: Component + 'static,
    {
        self.allowed_components.push(ComponentQueryData::new(
            T::name(),
            Some(Box::new(move |comp| {
                predicate(T::cast(comp).expect(&format!(
                    "Component provided to where clause of query can be cast to concrete Component {}",
                    T::name()
                )))
            })),
        ));

        self
    }

    pub(super) fn allowed_components(&self) -> &Vec<ComponentQueryData> {
        &self.allowed_components
    }

    pub(super) fn forbidden_components(&self) -> &Vec<ComponentQueryData> {
        &self.forbidden_components
    }

    pub(super) fn included_components(&self) -> &Vec<ComponentQueryData> {
        &self.included_components
    }

    pub(super) fn allowed_component_names(&self) -> Vec<&'static str> {
        self.allowed_components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }

    pub(super) fn forbidden_component_names(&self) -> Vec<&'static str> {
        self.forbidden_components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }

    pub(super) fn included_component_names(&self) -> Vec<&'static str> {
        self.included_components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }

    pub(super) fn has_inclusions(&self) -> bool {
        return self.included_components.len() > 0;
    }
}

pub struct QueryResult {
    pub(crate) entity: Entity,
    pub(crate) components: StoredComponentList,
}
impl QueryResult {
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn components(&self) -> &StoredComponentList {
        &self.components
    }
}

pub struct QueryResultList {
    matches: Vec<QueryResult>,
    inclusions: Vec<QueryResult>,
}
impl QueryResultList {
    pub fn new(matches: Vec<QueryResult>) -> Self {
        Self {
            matches,
            inclusions: vec![],
        }
    }

    pub fn new_with_inclusions(matches: Vec<QueryResult>, inclusions: Vec<QueryResult>) -> Self {
        Self {
            matches,
            inclusions,
        }
    }

    pub fn inclusions(&self) -> &Vec<QueryResult> {
        &self.inclusions
    }
}
impl IntoIterator for QueryResultList {
    type Item = QueryResult;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.matches.into_iter()
    }
}
impl<'a> IntoIterator for &'a QueryResultList {
    type Item = <std::slice::Iter<'a, QueryResult> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, QueryResult>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.matches).into_iter()
    }
}
impl Deref for QueryResultList {
    type Target = Vec<QueryResult>;

    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}

pub struct ComponentQueryData {
    component_name: &'static str,
    where_predicate: Option<Box<WherePredicate>>,
}
impl ComponentQueryData {
    pub(crate) fn new(
        component_name: &'static str,
        where_predicate: Option<Box<WherePredicate>>,
    ) -> Self {
        Self {
            component_name,
            where_predicate,
        }
    }

    pub fn component_name(&self) -> &'static str {
        &self.component_name
    }

    pub fn where_predicate(&self) -> &Option<Box<WherePredicate>> {
        &self.where_predicate
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
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    TestComponent {
                        prop: "val".to_string(),
                    },
                )
                    as Box<dyn Component>))]),
            };

            let test_component = qr.components().try_get::<TestComponent>().unwrap();

            assert_eq!(test_component.prop, "val");
        }

        #[test]
        fn is_none_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    AnotherEmptyComponent {},
                )
                    as Box<dyn Component>))]),
            };

            let empty_component_option = qr.components().try_get::<EmptyComponent>();

            assert!(empty_component_option.is_none());
        }
    }

    mod test_try_get_mut {
        use super::*;

        #[test]
        fn can_mutate_returned_component() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    TestComponent {
                        prop: "val".to_string(),
                    },
                )
                    as Box<dyn Component>))]),
            };

            let mut test_component = qr.components().try_get_mut::<TestComponent>().unwrap();

            assert_eq!(test_component.prop, "val");

            let new_prop = String::from("now for something totally different");

            test_component.prop = new_prop.clone();

            assert_eq!(test_component.prop, new_prop);
        }

        #[test]
        fn is_none_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    AnotherEmptyComponent {},
                )
                    as Box<dyn Component>))]),
            };

            let empty_component_option = qr.components().try_get_mut::<EmptyComponent>();

            assert!(empty_component_option.is_none());
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn gives_back_component_when_it_is_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    TestComponent {
                        prop: "val".to_string(),
                    },
                )
                    as Box<dyn Component>))]),
            };

            let test_component = qr.components().get::<TestComponent>();

            assert_eq!(test_component.prop, "val");
        }

        #[test]
        #[should_panic(expected = "Component EmptyComponent was not present.")]
        fn panics_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    AnotherEmptyComponent {},
                )
                    as Box<dyn Component>))]),
            };

            qr.components().get::<EmptyComponent>();
        }
    }

    mod test_get_mut {
        use super::*;

        #[test]
        fn can_mutate_returned_component() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    TestComponent {
                        prop: "val".to_string(),
                    },
                )
                    as Box<dyn Component>))]),
            };

            let mut test_component = qr.components().get_mut::<TestComponent>();

            assert_eq!(test_component.prop, "val");

            let new_prop = String::from("now for something totally different");

            test_component.prop = new_prop.clone();

            assert_eq!(test_component.prop, new_prop);
        }

        #[test]
        #[should_panic(expected = "Component EmptyComponent was not present.")]
        fn panics_when_component_is_not_present_in_the_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(
                    AnotherEmptyComponent {},
                )
                    as Box<dyn Component>))]),
            };

            qr.components().get_mut::<EmptyComponent>();
        }
    }
}
