use crate::{GameCommandsArg, Priority, Query, QueryResultList};

/// The function that's given to a `System` to run against its queries' matches.
pub type OperatorFn = dyn Fn(Vec<QueryResultList>, GameCommandsArg) -> ();

/// A System represents a function that uses the result of a collection of queries to act on and potentially mutate
/// the game world. Systems are a core aspect of ECS. Systems are where the bulk of the logic of your game will live,
/// as they're responsible for changing game state based on existing state.
/// 
/// Systems may also have a Priority, which will determine when Thomas will execute them relative to other Systems in
/// the same event. You can use the priority if it's imperative in your game that a particular system run before or after
/// other ones.
pub struct System {
    queries: Vec<Query>,
    operator: Box<OperatorFn>,
    priority: Priority,
}
impl System {
    /// Makes a new System that will operate on the results of the provided queries. Even if a system's queries have
    /// no matches (or the System has no queries), the System will _still run_. Some Systems may not need any query results
    /// to operate, such as an initialization system that populates the world with some basic entities. 
    /// 
    /// Because Systems always run even if they get no matches, your provided operator should be written to account
    /// for the possibility of no matches unless you can be confident a particular query will always produce a result.
    /// For example, Thomas injects an entity with the Input component for you very early on in the game's lifecyle, so
    /// you can be confident that query asking for Input will always give you exactly one match.
    /// 
    /// The keen-eyed may also notice a potential issue with the fact that Systems can operate on the results of multiple
    /// queries: if you have two queries that match on the same component, attempting to borrow that component mutably
    /// twice in your operator will result in a panic when your operator runs. Because of this, it may be best to disjoint
    /// your query by making it more specific. In this way, you limit the results of the two queries such that you don't
    /// match on the same component more than once. For example:
    /// ```
    /// use thomas::{Query, System, Identity};
    /// 
    /// System::new(vec![
    ///     Query::new()
    ///         .has::<Identity>(),
    ///     Query::new()
    ///         .has_where::<Identity>(|identity| identity.id == String::from("PLAYER")),
    /// ], |_, _| {});
    /// ```
    /// When run, those queries will produce results where the first query has matched on the same `Identity` as the
    /// second query. You can disjoint your query by using `has_where` or `has_no` to further filter the results to make
    /// sure the matches of one query are excluded from the other:
    /// ```
    /// use thomas::{Query, System, Identity};
    /// 
    /// System::new(vec![
    ///     Query::new()
    ///         .has_where::<Identity>(|identity| identity.id != String::from("PLAYER")),
    ///     Query::new()
    ///         .has_where::<Identity>(|identity| identity.id == String::from("PLAYER")),
    /// ], |_, _| {});
    /// ```
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

    /// Makes a new `System`, but allows you to specify a `Priority` for the `System`. Priority allows you to designate the order
    /// in which systems should run in a particular event relative to other systems in that event.
    /// 
    /// For example, if you add a `System` to the update event with `System::new(...)`, it will be added with the default priority. 
    /// If you then add another `System` to the update event with `System::new_with_priority(Priority::higher_than(Priority::default()), ...)`,
    /// that `System` will be guaranteed to run before the first `System` you added whenever the update event is invoked by
    /// Thomas.
    /// 
    /// You should use priorities sparingly. In general, letting application logic run in an ambiguous order at default
    /// priority will be ideal. It won't require as much micro-management of priorities, which is difficult to maintain
    /// and increases the cognitive load of trying to understand your execution order in a larger, busier game.
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

/// A simple way to organize related systems into a unit. You can easily add all systems created by a `SystemsGenerator`
/// to your game using the `Game::add_systems_from_generator` method.
pub trait SystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)>;
}
