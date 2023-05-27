use crate::{Query, QueryResultList};

type OperatorFn = fn(QueryResultList) -> ();

pub struct System {
    query: Query,
    operator: OperatorFn,
}
impl System {
    pub fn new(query: Query, operator: OperatorFn) -> Self {
        Self { query, operator }
    }

    pub(crate) fn query(&self) -> &Query {
        &self.query
    }

    pub fn operator(&self) -> &OperatorFn {
        &self.operator
    }
}
