use crate::{Query, QueryResultList};

pub type OperatorFn = dyn Fn(QueryResultList) -> ();

pub struct System {
    query: Query,
    operator: Box<OperatorFn>,
}
impl System {
    pub fn new(query: Query, operator: impl Fn(QueryResultList) -> () + 'static) -> Self {
        Self {
            query,
            operator: Box::new(operator),
        }
    }

    pub(crate) fn query(&self) -> &Query {
        &self.query
    }

    pub fn operator(&self) -> &OperatorFn {
        &self.operator
    }
}
