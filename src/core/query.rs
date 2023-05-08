use std::ops::Deref;

use crate::{Component, Entity, StoredComponent};

pub struct Query {
    components: Vec<ComponentQueryData>,
}
impl Query {
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn has<T: Component>(mut self) -> Self {
        self.components.push(ComponentQueryData::new(T::name()));

        self
    }

    // TODO: Figure out how to implement.
    // pub fn has_where<T: Component>(&mut self, predicate: fn(&T) -> bool) {
    //     self.components.push(ComponentQueryData::new(T::name(), predicate));
    // }

    pub fn components(&self) -> &Vec<ComponentQueryData> {
        &self.components
    }

    pub fn component_names(&self) -> Vec<&'static str> {
        self.components
            .iter()
            .map(|component_query_data| component_query_data.component_name)
            .collect()
    }
}

pub struct QueryResult {
    pub(crate) entity: Entity,
    pub(crate) components: Vec<StoredComponent>,
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

    #[derive(Component)]
    struct EmptyComponent {}

    #[derive(Component)]
    struct AnotherEmptyComponent {}

    mod test_get_component {
        use std::{cell::RefCell, rc::Rc};

        use crate::get_component;

        use super::*;

        #[test]
        #[should_panic(
            expected = "get_component: Provided component type is present in query results."
        )]
        fn panics_when_the_component_is_not_present_in_results() {
            let qr = QueryResult {
                entity: Entity(0),
                components: vec![Rc::new(RefCell::new(
                    Box::new(EmptyComponent {}) as Box<dyn Component>
                ))],
            };

            get_component!(qr, AnotherEmptyComponent);
        }

        #[test]
        fn gives_back_component_when_it_is_present_in_results() {}
    }
}
