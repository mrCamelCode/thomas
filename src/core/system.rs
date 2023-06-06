use crate::{GameCommandsArg, Priority, Query, QueryResultList};

pub type OperatorFn = dyn Fn(Vec<QueryResultList>, GameCommandsArg) -> ();
pub struct System {
    queries: Vec<Query>,
    operator: Box<OperatorFn>,
    priority: Priority,
}
impl System {
    pub fn new(
        queries: Vec<Query>,
        operator: impl Fn(Vec<QueryResultList>, GameCommandsArg) -> () + 'static,
    ) -> Self {
        Self {
            queries,
            operator: Box::new(operator),
            priority: Priority::default(),
        }
    }

    pub fn new_with_priority(
        priority: Priority,
        queries: Vec<Query>,
        operator: impl Fn(Vec<QueryResultList>, GameCommandsArg) -> () + 'static,
    ) -> Self {
        Self {
            queries,
            operator: Box::new(operator),
            priority,
        }
    }

    pub(crate) fn queries(&self) -> &Vec<Query> {
        &self.queries
    }

    pub(crate) fn operator(&self) -> &OperatorFn {
        &self.operator
    }

    pub(crate) fn priority(&self) -> &Priority {
        &self.priority
    }
}

pub trait SystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)>;
}
