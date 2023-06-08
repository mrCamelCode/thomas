use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

use crate::{Component, Entity, StoredComponentList};

pub type WherePredicate = dyn Fn(&dyn Component) -> bool + 'static;

/// Represents how to pull specific entities and components out of the world for use by a `System`. Methods
/// in a `Query` can be chained to create more complex queries. All chains are treated like logical ANDs.
/// 
/// For example, the following `Query`:
/// ```
/// use thomas::{Query, TerminalTransform, Identity, Component};
/// 
/// #[derive(Component)]
/// struct CustomComponent {}
/// 
/// Query::new()
///     .has::<TerminalTransform>()
///     .has_no::<CustomComponent>()
///     .has_where::<Identity>(|identity| identity.id == String::from("PLAYER"));
/// ```
/// Will match _only_ for entities that have a `TerminalTransform`, AND do NOT have a `CustomComponent`, AND have an `Identity`
/// component where its `id` property equals `"PLAYER"`, .
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

    /// Specifies that a matching entity must have the provided component to be a match for the query.
    pub fn has<T: Component + 'static>(mut self) -> Self {
        self.allowed_components
            .push(ComponentQueryData::new(T::name(), None));

        self
    }

    /// Specifies that a matching entity may _not_ have the provided component to be a match for the query.
    /// Regardless of any potential matches from `has`, if an entity has any component specified by any
    /// `has_no` calls on the query, that will entirely remove that entity from the ultimate list of matches.
    pub fn has_no<T: Component + 'static>(mut self) -> Self {
        self.forbidden_components
            .push(ComponentQueryData::new(T::name(), None));

        self
    }

    /// Specifies that a matching entity must have the provided component _and_ the component must pass the provided
    /// predicate to be a match for the query.
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
}

/// Represents a single match from a query.
pub struct QueryResult {
    pub(crate) entity: Entity,
    pub(crate) components: StoredComponentList,
}
impl QueryResult {
    /// The entity that matched the query.
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    /// All components that were specified as required by the query. Note that this **is not** all components
    /// on the matched entity. It's **only** the components specified by the query.
    pub fn components(&self) -> &StoredComponentList {
        &self.components
    }
}

/// A collection of matches against a query. Queries will typically match on more than one entity in the world,
/// so this is the representation you'll see when interacting with the results of a query.
pub struct QueryResultList {
    matches: Vec<QueryResult>,
}
impl QueryResultList {
    pub fn new(matches: Vec<QueryResult>) -> Self {
        Self { matches }
    }

    pub fn matches(&self) -> &Vec<QueryResult> {
        &self.matches
    }

    /// A convenience method that gets the first match and retrieves the specified component from its list of matched components.
    /// This is useful when you have a query that will only ever match on exactly **one** entity in the world. In that case, you're
    /// `get`ting the `only` match that query will ever have.
    /// 
    /// # Panics
    /// If there isn't at least one match in the `QueryResultList`, or the specified component is not in the list of
    /// matched components on the first match.
    pub fn get_only<T: Component + 'static>(&self) -> Ref<T> {
        self[0].components().get::<T>()
    }

    /// Like `get_only`, but provides a mutable reference.
    /// 
    /// # Panics
    /// If there isn't at least one match in the `QueryResultList`, or the specified component is not in the list of
    /// matched components on the first match.
    pub fn get_only_mut<T: Component + 'static>(&self) -> RefMut<T> {
        self[0].components().get_mut::<T>()
    }

    /// Like `get_only`, but doesn't panic.
    pub fn try_get_only<T: Component + 'static>(&self) -> Option<Ref<T>> {
        if let Some(query_match) = self.get(0) {
            return query_match.components().try_get::<T>();
        }

        None
    }

    /// Like `try_get_only`, but provides a mutable reference.
    pub fn try_get_only_mut<T: Component + 'static>(&self) -> Option<RefMut<T>> {
        if let Some(query_match) = self.get(0) {
            return query_match.components().try_get_mut::<T>();
        }

        None
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

pub(crate) struct ComponentQueryData {
    component_name: &'static str,
    where_predicate: Option<Box<WherePredicate>>,
}
impl ComponentQueryData {
    pub fn new(
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
        #[should_panic(
            expected = "Component EmptyComponent was not present, or you're trying to borrow it while it's already mutably borrowed."
        )]
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
        #[should_panic(
            expected = "Component EmptyComponent was not present, or you're trying to borrow it while it's already mutably borrowed."
        )]
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
