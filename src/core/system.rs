use crate::{Query, QueryResult};

pub struct System {
    query: Query,
    operator: dyn Fn(QueryResult) -> (),
}
