use std::{ops::Deref, rc::Rc};

use crate::{Component, Entity, Identity, StoredComponent};

// TODO: Currently doesn't work because of trying to use a borrowed value that
// isn't available after all this.
#[macro_export]
macro_rules! get_component {
    ($results:ident, $typ:ty) => {{
        $results
            .components
            .iter()
            .find(|comp| comp.borrow().component_name() == <$typ>::name())
            .expect("get_component: Provided component type is present in query results.")
            .borrow()

        // let comp_ref = comp.as_ref();

        // <$typ>::coerce(comp_ref)
    }};
}

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
    // pub fn get<T: Component>(&self) -> &T {
    //     let comp = self
    //         .components
    //         .iter()
    //         .find(|comp| comp.borrow().component_name() == T::name())
    //         .expect("Provided component type is present in query results.")
    //         .borrow();

    //     T::coerce(comp.as_ref()).unwrap()
    // }
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

fn test() {
    let qr = QueryResult {
        entity: Entity(0),
        components: vec![],
    };

    // let comp = Identity::coerce(get_component!(qr, Identity).as_ref());

    let comp = {
        let comp_ref = Rc::clone(
            qr.components
                .iter()
                .find(|comp| comp.borrow().component_name() == Identity::name())
                .unwrap(),
        );

        let t = comp_ref.borrow();
        let comp: Option<&Identity> = Identity::coerce(t.as_ref());    
        
        comp
    };

    println!("id: {}", comp.unwrap().id);
}

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::Component;

    #[derive(Component)]
    struct EmptyComponent {}

    #[derive(Component)]
    struct AnotherEmptyComponent {}

    mod test_get_component {
        use std::{cell::RefCell, rc::Rc};

        use crate::{Entity, QueryResult};

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

        // #[test]
        // fn gives_back_component_when_it_is_present_in_results() {
        //     let qr = QueryResult {
        //         entity: Entity(0),
        //         components: vec![Rc::new(RefCell::new(
        //             Box::new(EmptyComponent {}) as Box<dyn Component>
        //         ))],
        //     };

        //     let comp = get_component!(qr, EmptyComponent);

        //     assert_eq!(comp.unwrap().component_name(), EmptyComponent::name());
        // }
    }
}
