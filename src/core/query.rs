use std::ops::Deref;

use crate::{Component, Coords, Entity, Identity, StoredComponent, Transform};

pub struct Query {
    components: Vec<ComponentQueryData>,
    use_mut: bool,
}
impl Query {
    pub fn new() -> Self {
        Self {
            components: vec![],
            use_mut: false,
        }
    }

    pub fn has<T: Component>(mut self) -> Self {
        self.components.push(ComponentQueryData::new(T::name()));

        self
    }

    // pub fn has_where<T: Component>(&mut self, predicate: fn(&T) -> bool) {
    //     self.components.push(ComponentQueryData::new(T::name(), predicate));
    // }

    pub fn use_mut(&mut self) {
        self.use_mut = true;
    }

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

// struct ComponentQueryData<T> {
//     component_name: &'static str,
//     predicate: fn(&T) -> bool,
// }
// impl<T> ComponentQueryData<T> where T: Component {
//     fn new(component_name: &'static str, predicate: fn(&T) -> bool) -> Self {
//         Self {
//             component_name,
//             predicate,
//         }
//     }
// }

fn tmp_example_interface_use() {
    let q = Query::new().has::<Transform>().has::<Identity>();

    // em.query(q);
    let results: Vec<&dyn Component> = vec![
        &Transform {
            coords: Coords::zero(),
        },
        &Identity {
            id: "01".to_string(),
            name: "a guy".to_string(),
        },
    ];

    fn system(stuff: Vec<&dyn Component>) {}
}

// pub struct QueryResult<'a> {
//     pub(crate) entity: Entity,
//     pub(crate) components: Vec<&'a Box<dyn Component>>,
// }
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
