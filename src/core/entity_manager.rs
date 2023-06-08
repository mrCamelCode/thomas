use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use crate::{Component, ComponentQueryData, Entity, Query, QueryResult, QueryResultList};

pub type StoredComponent = Rc<RefCell<Box<dyn Component>>>;
type EntitiesToComponents = HashMap<Entity, HashMap<String, StoredComponent>>;
type ComponentsToEntities = HashMap<String, HashSet<Entity>>;

/// A list of components that are currently stored in the game world.
pub struct StoredComponentList {
    components: Vec<StoredComponent>,
}
impl StoredComponentList {
    pub fn new(components: Vec<StoredComponent>) -> Self {
        Self { components }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Attempts to retrieve the specified component from the list. If no such component exists, returns `None`.
    pub fn try_get<T>(&self) -> Option<Ref<T>>
    where
        T: Component + 'static,
    {
        for component in &self.components {
            if component.try_borrow().is_ok() && (**component.borrow()).as_any().is::<T>() {
                return Some(Ref::map(component.borrow(), |component| {
                    (**component).as_any().downcast_ref::<T>().unwrap()
                }));
            }
        }

        None
    }

    /// Like `try_get`, but retrieves a mutable reference.
    pub fn try_get_mut<T>(&self) -> Option<RefMut<T>>
    where
        T: Component + 'static,
    {
        for component in &self.components {
            if component.try_borrow().is_ok() && (**component.borrow()).as_any().is::<T>() {
                return Some(RefMut::map(component.borrow_mut(), |component| {
                    (**component).as_any_mut().downcast_mut::<T>().unwrap()
                }));
            }
        }

        None
    }

    /// Gets a reference to the specified component in the list.
    ///
    /// # Panics
    /// If the component you specify isn't in the list, or you've already mutably borrowed that same component.
    pub fn get<T>(&self) -> Ref<T>
    where
        T: Component + 'static,
    {
        if let Some(component) = self.try_get::<T>() {
            return component;
        }

        panic!("Component {} was not present, or you're trying to borrow it while it's already mutably borrowed.", T::name());
    }

    /// Like `get`, but retrieves a mutable reference.
    /// # Panics
    /// If the component you specify isn't in the list, or you've already mutably borrowed that same component.
    pub fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Component + 'static,
    {
        if let Some(component) = self.try_get_mut::<T>() {
            return component;
        }

        panic!("Component {} was not present, or you're trying to borrow it while it's already mutably borrowed.", T::name());
    }
}

/// The core representation of the game world in memory, the `EntityManager` facilitates all operations on updating the
/// game world, its entities, and their components. Queries can be run against the `EntityManager` to produce matches
/// that can be used by systems.
///
/// For speed of retrieval, the `EntityManager` internally uses maps and sets to track all the entities in the world and
/// their components, as well as all components in the world and the entities that have them.
pub(crate) struct EntityManager {
    entities_to_components: EntitiesToComponents,
    components_to_entities: ComponentsToEntities,
    available_entity_ids: Vec<Entity>,
}
impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities_to_components: HashMap::new(),
            components_to_entities: HashMap::new(),
            available_entity_ids: vec![],
        }
    }

    /// Adds an entity to the world, reusing any available entity IDs before falling back to creating a new one.
    /// Returns a copy of the created `Entity`.
    pub fn add_entity(&mut self, components: Vec<Box<dyn Component>>) -> Entity {
        let entity = self.get_next_entity();

        for component in &components {
            if self
                .components_to_entities
                .contains_key(component.component_name())
            {
                if let Some(entity_set) = self
                    .components_to_entities
                    .get_mut(component.component_name())
                {
                    entity_set.insert(entity);
                }
            } else {
                let mut entity_set = HashSet::new();
                entity_set.insert(entity);

                self.components_to_entities
                    .insert(component.component_name().to_string(), entity_set);
            }
        }

        let mut component_map = HashMap::new();

        for component in components {
            component_map.insert(
                component.component_name().to_string(),
                Rc::new(RefCell::new(component)),
            );
        }

        self.entities_to_components.insert(entity, component_map);

        return entity;
    }

    /// Removes an entity from the world, freeing its ID for reuse.
    pub fn remove_entity(&mut self, entity: &Entity) {
        if let Some((removed_entity, component_map)) =
            self.entities_to_components.remove_entry(entity)
        {
            for component in component_map.values() {
                if let Some(entity_set) = self
                    .components_to_entities
                    .get_mut(component.borrow().component_name())
                {
                    entity_set.remove(entity);
                }
            }

            self.available_entity_ids.push(removed_entity);
        }
    }

    pub fn add_component_to_entity(&mut self, entity: &Entity, component: Box<dyn Component>) {
        if let Some(component_map) = self.entities_to_components.get_mut(&entity) {
            if !entity_has_component(&self.components_to_entities, &entity, &component) {
                let component_name = component.component_name();

                component_map.insert(component_name.to_string(), Rc::new(RefCell::new(component)));

                if let Some(entity_set) = self.components_to_entities.get_mut(component_name) {
                    entity_set.insert(*entity);
                } else {
                    let mut entity_set = HashSet::new();
                    entity_set.insert(*entity);

                    self.components_to_entities
                        .insert(component_name.to_string(), entity_set);
                }
            }
        }
    }

    pub fn remove_component_from_entity(&mut self, entity: &Entity, component_name: &'static str) {
        if let Some(component_map) = self.entities_to_components.get_mut(&entity) {
            if component_map.remove(component_name).is_some() {
                if let Some(entity_set) = self.components_to_entities.get_mut(component_name) {
                    entity_set.remove(entity);
                }
            }
        }
    }

    /// Allows a `Query` to be run against the `EntityManager`, producing a `QueryResultList` reflecting the matches in the
    /// current state of the game world.
    pub fn query(&self, query: &Query) -> QueryResultList {
        let allowed_component_names = query.allowed_component_names();
        let forbidden_component_names = query.forbidden_component_names();
        let entities_with_forbidden_components = Self::get_entities_with_components(
            &self.components_to_entities,
            &forbidden_component_names,
        );

        let matches = Self::get_entities_with_components(
            &self.components_to_entities,
            &allowed_component_names,
        )
        .into_iter()
        .filter_map(|entity_with_desired_components| {
            if entities_with_forbidden_components.contains(&entity_with_desired_components)
                || !Self::entity_components_pass_all_predicates(
                    &self.entities_to_components,
                    &entity_with_desired_components,
                    &query.allowed_components(),
                )
            {
                None
            } else {
                Some(QueryResult {
                    entity: entity_with_desired_components,
                    components: Self::get_components_on_entity(
                        &self.entities_to_components,
                        &entity_with_desired_components,
                        &allowed_component_names,
                    ),
                })
            }
        })
        .collect();

        QueryResultList::new(matches)
    }

    fn get_next_entity(&mut self) -> Entity {
        if self.available_entity_ids.len() > 0 {
            self.available_entity_ids.pop().unwrap()
        } else {
            Entity::new()
        }
    }

    fn entity_components_pass_all_predicates(
        entities_to_components: &EntitiesToComponents,
        entity: &Entity,
        component_query_data_list: &Vec<ComponentQueryData>,
    ) -> bool {
        component_query_data_list
            .iter()
            .all(|component_query_data| {
                if let Some(component) = Self::get_component_on_entity(
                    entities_to_components,
                    entity,
                    component_query_data.component_name(),
                ) {
                    if let Some(where_predicate) = component_query_data.where_predicate() {
                        return where_predicate(&**component.borrow());
                    } else {
                        return true;
                    }
                }

                false
            })
    }

    fn get_entities_with_component(
        components_to_entities: &ComponentsToEntities,
        component_name: &'static str,
    ) -> Vec<Entity> {
        if let Some(entities_with_component) = components_to_entities.get(component_name) {
            return entities_with_component
                .iter()
                .map(|entity_ref| *entity_ref)
                .collect();
        }

        vec![]
    }

    fn get_entities_with_components(
        components_to_entities: &ComponentsToEntities,
        component_names: &Vec<&'static str>,
    ) -> Vec<Entity> {
        let mut entity_lists: Vec<Vec<Entity>> = component_names
            .iter()
            .map(|component_name| {
                Self::get_entities_with_component(components_to_entities, component_name)
            })
            .collect();

        // TODO: This isn't the most efficient. Multiple conditional retrieval could likely be
        // sped up with the introduction of automatic archetype management.
        if entity_lists.len() == 1 {
            entity_lists.pop().unwrap()
        } else {
            intersection(&entity_lists)
                .into_iter()
                .map(|entity_ref| *entity_ref)
                .collect()
        }
    }

    fn get_component_on_entity(
        entities_to_components: &EntitiesToComponents,
        entity: &Entity,
        component_name: &'static str,
    ) -> Option<StoredComponent> {
        if let Some(stored_components) = entities_to_components.get(entity) {
            if let Some(stored_component) = stored_components.get(component_name) {
                return Some(Rc::clone(stored_component));
            }
        }

        None
    }

    fn get_components_on_entity(
        entities_to_components: &EntitiesToComponents,
        entity: &Entity,
        component_names: &Vec<&'static str>,
    ) -> StoredComponentList {
        StoredComponentList::new(
            component_names
                .iter()
                .filter_map(|component_name| {
                    Self::get_component_on_entity(&entities_to_components, entity, component_name)
                })
                .collect(),
        )
    }
}

fn entity_has_component(
    components_to_entities: &ComponentsToEntities,
    entity: &Entity,
    component: &Box<dyn Component>,
) -> bool {
    if let Some(entity_set) = components_to_entities.get(component.component_name()) {
        return entity_set.contains(&entity);
    }

    false
}

fn intersection<T: Hash + Eq + PartialEq>(vectors: &Vec<Vec<T>>) -> Vec<&T> {
    let mut values_tracker: HashSet<&T> = HashSet::new();
    let mut intersecting_values = vec![];

    for values_vector in vectors {
        for value in values_vector {
            if values_tracker.contains(value) {
                intersecting_values.push(value);
            } else {
                values_tracker.insert(value);
            }
        }
    }

    intersecting_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct EmptyComponent {}

    #[derive(Component)]
    struct TestComponent {
        prop1: u8,
    }

    #[derive(Component)]
    struct OtherTestComponent {
        prop1: u8,
    }

    #[derive(Component)]
    struct AnotherTestComponent {
        prop1: u8,
    }

    mod test_stored_component_list {
        use super::*;

        #[test]
        fn can_borrow_multiple_different_components_mutably_at_the_same_time() {
            let mut em = EntityManager::new();

            em.add_entity(vec![
                Box::new(EmptyComponent {}),
                Box::new(TestComponent { prop1: 5 }),
            ]);

            let results = em.query(&Query::new().has::<EmptyComponent>().has::<TestComponent>());

            for result in results {
                let mut _bind1 = result.components().get_mut::<EmptyComponent>();
                let mut _bind2 = result.components().get_mut::<TestComponent>();
            }
        }

        #[test]
        #[should_panic(
            expected = "Component EmptyComponent was not present, or you're trying to borrow it while it's already mutably borrowed."
        )]
        fn panics_when_trying_to_borrow_the_same_component_mutably_more_than_once() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(EmptyComponent {})]);

            let results = em.query(&Query::new().has::<EmptyComponent>());

            for result in results {
                let mut _bind1 = result.components().get_mut::<EmptyComponent>();
                let mut _bind2 = result.components().get_mut::<EmptyComponent>();
            }
        }

        #[test]
        #[should_panic(
            expected = "Component TestComponent was not present, or you're trying to borrow it while it's already mutably borrowed."
        )]
        fn panics_when_trying_to_borrow_a_component_that_is_not_present() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(EmptyComponent {})]);

            let results = em.query(&Query::new().has::<EmptyComponent>());

            for result in results {
                result.components().get::<TestComponent>();
            }
        }
    }

    mod test_add_entity {
        use super::*;

        #[test]
        fn returns_entity_and_adds_to_maps() {
            let mut em = EntityManager::new();

            let result = em.add_entity(vec![]);

            let component_map = em.entities_to_components.get(&result);

            assert!(component_map.is_some());
            assert!(component_map.unwrap().is_empty());
            assert!(em.components_to_entities.is_empty());
        }

        #[test]
        fn can_add_a_component_with_the_entity() {
            let mut em = EntityManager::new();

            let result = em.add_entity(vec![Box::new(TestComponent { prop1: 1 })]);

            let component_map = em
                .entities_to_components
                .get(&result)
                .expect("The component map was added for the entity");

            let comp = component_map.get(TestComponent::name()).unwrap().borrow();
            let test_component =
                TestComponent::cast(comp.as_ref()).expect("Component is TestComponent");

            assert!(component_map.get(TestComponent::name()).is_some());
            assert_eq!(test_component.prop1, 1);
        }

        #[test]
        fn can_add_multiple_components_with_the_entity() {
            let mut em = EntityManager::new();

            let result = em.add_entity(vec![
                Box::new(TestComponent { prop1: 1 }),
                Box::new(OtherTestComponent { prop1: 3 }),
                Box::new(AnotherTestComponent { prop1: 5 }),
            ]);

            let component_map = em
                .entities_to_components
                .get(&result)
                .expect("The component map was added for the entity");

            let (comp1, comp2, comp3) = (
                component_map.get(TestComponent::name()).unwrap().borrow(),
                component_map
                    .get(OtherTestComponent::name())
                    .unwrap()
                    .borrow(),
                component_map
                    .get(AnotherTestComponent::name())
                    .unwrap()
                    .borrow(),
            );

            let test_component =
                TestComponent::cast(comp1.as_ref()).expect("Component is TestComponent");
            let other_test_component =
                OtherTestComponent::cast(comp2.as_ref()).expect("Component is OtherTestComponent");
            let another_test_component = AnotherTestComponent::cast(comp3.as_ref())
                .expect("Component is AnotherTestComponent");

            assert!(component_map.get(TestComponent::name()).is_some());
            assert_eq!(test_component.prop1, 1);

            assert!(component_map.get(OtherTestComponent::name()).is_some());
            assert_eq!(other_test_component.prop1, 3);

            assert!(component_map.get(AnotherTestComponent::name()).is_some());
            assert_eq!(another_test_component.prop1, 5);
        }

        #[test]
        fn ids_are_reused_when_available() {
            let mut em = EntityManager::new();
            em.available_entity_ids.push(Entity(1000));

            let entity = em.add_entity(vec![]);

            assert_eq!(entity, Entity(1000));
            assert_eq!(em.available_entity_ids.len(), 0);
        }
    }

    mod test_remove_entity {
        use super::*;

        #[test]
        fn removing_a_nonexistent_entity_does_nothing() {
            let mut em = EntityManager::new();

            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(1)]),
            );
            em.entities_to_components.insert(
                Entity(1),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 1 }) as Box<dyn Component>
                    )),
                )]),
            );

            em.remove_entity(&Entity(2));

            let entity_set = em
                .components_to_entities
                .get(TestComponent::name())
                .unwrap();
            let component_map = em.entities_to_components.get(&Entity(1)).unwrap();

            assert_eq!(entity_set.len(), 1);
            assert!(entity_set.contains(&Entity(1)));

            assert_eq!(component_map.len(), 1);
            assert_eq!(
                TestComponent::cast(
                    component_map
                        .get(TestComponent::name())
                        .unwrap()
                        .borrow()
                        .as_ref()
                )
                .unwrap()
                .prop1,
                1
            );
        }

        #[test]
        fn can_remove_an_existing_entity() {
            let mut em = EntityManager::new();

            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(1)]),
            );
            em.entities_to_components.insert(
                Entity(1),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 1 }) as Box<dyn Component>
                    )),
                )]),
            );

            em.remove_entity(&Entity(1));

            let entity_set = em
                .components_to_entities
                .get(TestComponent::name())
                .expect("TestComponent entry wasn't wiped just because there are no longer any Entities with that component.");
            let component_map = em.entities_to_components.get(&Entity(1));

            assert_eq!(entity_set.len(), 0);
            assert!(component_map.is_none());
            assert_eq!(em.entities_to_components.len(), 0);
            assert_eq!(em.components_to_entities.len(), 1);
        }

        #[test]
        fn can_remove_an_entity_that_has_no_components() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(Entity(1), HashMap::new());

            em.remove_entity(&Entity(1));

            let component_map = em.entities_to_components.get(&Entity(1));

            assert!(component_map.is_none());
            assert_eq!(em.entities_to_components.len(), 0);
            assert_eq!(em.components_to_entities.len(), 0);
        }

        #[test]
        fn removing_an_entity_makes_its_id_available() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![]);

            em.remove_entity(&entity);

            assert_eq!(em.available_entity_ids.len(), 1);
            assert_eq!(em.available_entity_ids[0], entity);
        }
    }

    mod test_add_component_to_entity {
        use super::*;

        #[test]
        fn nothing_happens_when_adding_to_a_nonexistent_entity() {
            let mut em = EntityManager::new();

            em.add_component_to_entity(
                &Entity(0),
                Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
            );

            assert!(em.components_to_entities.is_empty());
            assert!(em.entities_to_components.is_empty());
        }

        #[test]
        fn component_is_correctly_added_on_an_existing_entity() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(
                Entity(0),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>
                    )),
                )]),
            );
            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(0)]),
            );

            em.add_component_to_entity(
                &Entity(0),
                Box::new(OtherTestComponent { prop1: 10 }) as Box<dyn Component>,
            );

            assert_eq!(em.components_to_entities.len(), 2);
            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(
                OtherTestComponent::cast(
                    em.entities_to_components
                        .get(&Entity(0))
                        .expect("Entity 0 exists")
                        .get(OtherTestComponent::name())
                        .expect("OtherTestComponent is on Entity 0")
                        .borrow()
                        .as_ref()
                )
                .expect("OtherTestComponent could be cast.")
                .prop1,
                10
            );
        }

        #[test]
        fn nothing_happens_when_adding_a_component_to_an_entity_that_it_already_has() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(
                Entity(0),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>
                    )),
                )]),
            );
            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(0)]),
            );

            em.add_component_to_entity(
                &Entity(0),
                Box::new(TestComponent { prop1: 10 }) as Box<dyn Component>,
            );

            assert_eq!(em.components_to_entities.len(), 1);
            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(
                TestComponent::cast(
                    em.entities_to_components
                        .get(&Entity(0))
                        .expect("Entity 0 exists")
                        .get(TestComponent::name())
                        .expect("TestComponent is on Entity 0")
                        .borrow()
                        .as_ref()
                )
                .expect("TestComponent could be cast.")
                .prop1,
                5
            );
        }
    }

    mod test_remove_component_from_entity {
        use super::*;

        #[test]
        fn removing_a_component_that_does_not_exist_on_the_entity_has_no_effect() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(
                Entity(0),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>
                    )),
                )]),
            );
            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(0)]),
            );

            em.remove_component_from_entity(&Entity(0), OtherTestComponent::name());

            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(em.components_to_entities.len(), 1);
            assert!(!em
                .entities_to_components
                .get(&Entity(0))
                .expect("Entity 0 exists")
                .contains_key(OtherTestComponent::name()));
        }

        #[test]
        fn removing_a_component_on_a_nonexistent_entity_has_no_effect() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(
                Entity(0),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>
                    )),
                )]),
            );
            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(0)]),
            );

            em.remove_component_from_entity(&Entity(1), TestComponent::name());

            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(em.components_to_entities.len(), 1);
            assert!(!em
                .components_to_entities
                .get(TestComponent::name())
                .expect("TestComponent is available")
                .contains(&Entity(1)));
        }

        #[test]
        fn removing_from_an_existent_entity_succeeds() {
            let mut em = EntityManager::new();

            em.entities_to_components.insert(
                Entity(0),
                HashMap::from([(
                    TestComponent::name().to_string(),
                    Rc::new(RefCell::new(
                        Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>
                    )),
                )]),
            );
            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                HashSet::from([Entity(0)]),
            );

            em.remove_component_from_entity(&Entity(0), TestComponent::name());

            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(em.components_to_entities.len(), 1);
            assert!(em
                .entities_to_components
                .get(&Entity(0))
                .expect("Entity 0 exists.")
                .is_empty());
            assert!(em
                .components_to_entities
                .get(TestComponent::name())
                .expect("There's an entry for TestComponent")
                .is_empty());
        }
    }

    mod test_query {
        use super::*;

        mod without_forbidden_components {
            use super::*;

            #[test]
            fn query_returns_only_relevant_matches() {
                let mut em = EntityManager::new();

                let entity = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                ]);
                em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(&Query::new().has::<AnotherTestComponent>());

                assert_eq!((*query_results).len(), 1);
                assert_eq!(*query_results.get(0).unwrap().entity(), entity);
            }

            #[test]
            fn complex_query_for_more_than_one_component_match_works() {
                let mut em = EntityManager::new();

                let entity1 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                    Box::new(EmptyComponent {}),
                ]);
                em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);
                let entity2 = em.add_entity(vec![
                    Box::new(EmptyComponent {}),
                    Box::new(TestComponent { prop1: 1 }),
                ]);

                let query_results =
                    em.query(&Query::new().has::<EmptyComponent>().has::<TestComponent>());

                assert_eq!((*query_results).len(), 2);
                assert!(query_results
                    .iter()
                    .find(|result| *result.entity() == entity1)
                    .is_some());
                assert!(query_results
                    .iter()
                    .find(|result| *result.entity() == entity2)
                    .is_some());
            }

            #[test]
            fn can_read_queried_components() {
                let mut em = EntityManager::new();

                let entity1 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                ]);
                let entity2 = em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(&Query::new().has::<TestComponent>());

                assert_eq!((*query_results).len(), 2);

                for result in &query_results {
                    if *result.entity() == entity1 {
                        assert_eq!(result.components().get::<TestComponent>().prop1, 10)
                    } else if *result.entity() == entity2 {
                        assert_eq!(result.components().get::<TestComponent>().prop1, 20)
                    } else {
                        panic!(
                            "Entity present in results that should not be: {:?}",
                            result.entity()
                        )
                    }
                }
            }

            #[test]
            fn can_mutate_queried_components() {
                let mut em = EntityManager::new();

                let entity1 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                    Box::new(OtherTestComponent { prop1: 20 }),
                ]);
                let entity2 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 20 }),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                ]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has::<AnotherTestComponent>(),
                );

                assert_eq!(query_results.len(), 2);

                for result in &query_results {
                    if *result.entity() == entity1 {
                        result.components().get_mut::<AnotherTestComponent>().prop1 = 50;

                        result.components().get_mut::<TestComponent>().prop1 = 1;
                    } else if *result.entity() == entity2 {
                        result.components().get_mut::<TestComponent>().prop1 = 240;
                    }
                }

                for result in &query_results {
                    if *result.entity() == entity1 {
                        let test_component = result.components().get::<TestComponent>();
                        let another_test_component =
                            result.components().get::<AnotherTestComponent>();

                        assert_eq!(test_component.prop1, 1);
                        assert_eq!(another_test_component.prop1, 50);
                    } else if *result.entity() == entity2 {
                        let test_component = result.components().get::<TestComponent>();
                        let another_test_component =
                            result.components().get::<AnotherTestComponent>();

                        assert_eq!(test_component.prop1, 240);
                        assert_eq!(another_test_component.prop1, 2);
                    } else {
                        panic!(
                            "Entity present in results that should not be: {:?}",
                            result.entity()
                        )
                    }
                }
            }

            #[test]
            fn where_clauses_exclude_any_potential_matches_that_fail_the_predicate() {
                let mut em = EntityManager::new();

                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                    Box::new(OtherTestComponent { prop1: 20 }),
                ]);
                let entity = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 20 }),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                ]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_where::<AnotherTestComponent>(|another_test| another_test.prop1 == 2),
                );

                assert_eq!(query_results.len(), 1);

                let result = query_results.get(0).unwrap();

                assert_eq!(*result.entity(), entity);
                assert_eq!(result.components().len(), 2);
                assert_eq!(result.components().get::<AnotherTestComponent>().prop1, 2);
            }
        }

        mod with_forbidden_components {
            use super::*;

            #[test]
            fn query_has_no_results_when_the_forbidden_components_remove_all_potential_matches() {
                let mut em = EntityManager::new();

                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                ]);
                em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(
                    &Query::new()
                        .has::<AnotherTestComponent>()
                        .has_no::<TestComponent>(),
                );

                assert!(query_results.is_empty());
            }

            #[test]
            fn query_returns_only_relevant_matches() {
                let mut em = EntityManager::new();

                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                ]);
                let entity = em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);
                assert_eq!(*query_results.get(0).unwrap().entity(), entity);
            }

            #[test]
            fn complex_query_for_more_than_one_component_match_works() {
                let mut em = EntityManager::new();

                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                    Box::new(EmptyComponent {}),
                ]);
                em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);
                let entity = em.add_entity(vec![
                    Box::new(EmptyComponent {}),
                    Box::new(TestComponent { prop1: 1 }),
                ]);

                let query_results = em.query(
                    &Query::new()
                        .has::<EmptyComponent>()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);
                assert!(query_results
                    .iter()
                    .find(|result| *result.entity() == entity)
                    .is_some());
            }

            #[test]
            fn can_read_queried_components() {
                let mut em = EntityManager::new();

                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                ]);
                let entity = em.add_entity(vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);

                for result in &query_results {
                    if *result.entity() == entity {
                        assert_eq!(result.components().get::<TestComponent>().prop1, 20)
                    } else {
                        panic!("Entity present in results that should not be: {:?}", entity)
                    }
                }
            }

            #[test]
            fn can_mutate_queried_components() {
                let mut em = EntityManager::new();

                let entity1 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 100 }),
                    Box::new(OtherTestComponent { prop1: 20 }),
                ]);
                let entity2 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 20 }),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                ]);
                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 50 }),
                    Box::new(EmptyComponent {}),
                ]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has::<AnotherTestComponent>()
                        .has_no::<EmptyComponent>(),
                );

                assert_eq!(query_results.len(), 2);

                for result in &query_results {
                    if *result.entity() == entity1 {
                        result.components().get_mut::<AnotherTestComponent>().prop1 = 50;
                        result.components().get_mut::<TestComponent>().prop1 = 1;
                    } else if *result.entity() == entity2 {
                        result.components().get_mut::<TestComponent>().prop1 = 240;
                    }
                }

                for result in &query_results {
                    if *result.entity() == entity1 {
                        let test_component = result.components().get::<TestComponent>();
                        let another_test_component =
                            result.components().get::<AnotherTestComponent>();

                        assert_eq!(test_component.prop1, 1);
                        assert_eq!(another_test_component.prop1, 50);
                    } else if *result.entity() == entity2 {
                        let test_component = result.components().get::<TestComponent>();
                        let another_test_component =
                            result.components().get::<AnotherTestComponent>();

                        assert_eq!(test_component.prop1, 240);
                        assert_eq!(another_test_component.prop1, 2);
                    } else {
                        panic!(
                            "Entity present in results that should not be: {:?}",
                            result.entity()
                        )
                    }
                }
            }

            #[test]
            fn where_clauses_exclude_any_potential_matches_that_fail_the_predicate() {
                let mut em = EntityManager::new();

                let entity1 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                    Box::new(OtherTestComponent { prop1: 20 }),
                ]);
                let entity2 = em.add_entity(vec![
                    Box::new(TestComponent { prop1: 20 }),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                ]);
                em.add_entity(vec![
                    Box::new(TestComponent { prop1: 50 }),
                    Box::new(EmptyComponent {}),
                    Box::new(AnotherTestComponent { prop1: 2 }),
                ]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_where::<AnotherTestComponent>(|another_test| another_test.prop1 == 2)
                        .has_no::<EmptyComponent>(),
                );

                assert_eq!(query_results.len(), 2);
                assert!(query_results
                    .iter()
                    .find(|result| *result.entity() == entity1)
                    .is_some());
                assert!(query_results
                    .iter()
                    .find(|result| *result.entity() == entity2)
                    .is_some());
            }
        }
    }

    mod test_entity_components_pass_all_predicates {
        use super::*;

        #[test]
        fn is_true_when_all_predicates_pass() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(AnotherTestComponent { prop1: 100 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::entity_components_pass_all_predicates(
                &em.entities_to_components,
                &entity,
                &vec![
                    ComponentQueryData::new(
                        TestComponent::name(),
                        Some(Box::new(|comp| {
                            let test_component = TestComponent::cast(comp).unwrap();

                            test_component.prop1 == 10
                        })),
                    ),
                    ComponentQueryData::new(
                        AnotherTestComponent::name(),
                        Some(Box::new(|comp| {
                            let another_test_component = AnotherTestComponent::cast(comp).unwrap();

                            another_test_component.prop1 == 100
                        })),
                    ),
                ],
            );

            assert!(result);
        }

        #[test]
        fn is_true_when_there_are_no_predicates() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(AnotherTestComponent { prop1: 100 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::entity_components_pass_all_predicates(
                &em.entities_to_components,
                &entity,
                &vec![
                    ComponentQueryData::new(TestComponent::name(), None),
                    ComponentQueryData::new(AnotherTestComponent::name(), None),
                ],
            );

            assert!(result);
        }

        #[test]
        fn is_false_when_all_predicates_fail() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(AnotherTestComponent { prop1: 100 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::entity_components_pass_all_predicates(
                &em.entities_to_components,
                &entity,
                &vec![
                    ComponentQueryData::new(
                        TestComponent::name(),
                        Some(Box::new(|comp| {
                            let test_component = TestComponent::cast(comp).unwrap();

                            test_component.prop1 == 99
                        })),
                    ),
                    ComponentQueryData::new(
                        AnotherTestComponent::name(),
                        Some(Box::new(|comp| {
                            let another_test_component = AnotherTestComponent::cast(comp).unwrap();

                            another_test_component.prop1 == 5
                        })),
                    ),
                ],
            );

            assert!(!result);
        }

        #[test]
        fn is_false_when_any_predicate_fails() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(AnotherTestComponent { prop1: 100 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::entity_components_pass_all_predicates(
                &em.entities_to_components,
                &entity,
                &vec![
                    ComponentQueryData::new(
                        TestComponent::name(),
                        Some(Box::new(|comp| {
                            let test_component = TestComponent::cast(comp).unwrap();

                            test_component.prop1 == 10
                        })),
                    ),
                    ComponentQueryData::new(
                        AnotherTestComponent::name(),
                        Some(Box::new(|comp| {
                            let another_test_component = AnotherTestComponent::cast(comp).unwrap();

                            another_test_component.prop1 == 5
                        })),
                    ),
                ],
            );

            assert!(!result);
        }
    }

    mod test_get_entities_with_component {
        use super::*;

        #[test]
        fn is_empty_when_there_are_no_entities() {
            let em = EntityManager::new();

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                EmptyComponent::name(),
            );

            assert!(result.is_empty());
        }

        #[test]
        fn is_empty_when_no_entities_have_the_provided_component() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                EmptyComponent::name(),
            );

            assert!(result.is_empty());
        }

        #[test]
        fn works_when_one_entity_matches() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);
            let entity1 = em.add_entity(vec![Box::new(EmptyComponent {})]);

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                EmptyComponent::name(),
            );

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], entity1);
        }

        #[test]
        fn works_when_multiple_entities_match() {
            let mut em = EntityManager::new();

            let entity1 = em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);
            let entity2 = em.add_entity(vec![
                Box::new(TestComponent { prop1: 100 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                TestComponent::name(),
            );

            assert_eq!(result.len(), 2);
            assert!(result.contains(&entity1) && result.contains(&entity2));
        }
    }

    mod test_get_entities_with_components {
        use super::*;

        #[test]
        fn is_empty_when_there_are_no_entities() {
            let em = EntityManager::new();

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![EmptyComponent::name()],
            );

            assert!(result.is_empty());
        }

        #[test]
        fn is_empty_when_no_entities_have_provided_components() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![EmptyComponent::name()],
            );

            assert!(result.is_empty());
        }

        #[test]
        fn returns_entity_list_for_component_when_searching_for_one_component() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);
            let entity2 = em.add_entity(vec![
                Box::new(AnotherTestComponent { prop1: 20 }),
                Box::new(EmptyComponent {}),
            ]);

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![EmptyComponent::name()],
            );

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], entity2);
        }

        #[test]
        fn works_when_one_entity_has_all_provided_components() {
            let mut em = EntityManager::new();

            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);
            let entity2 = em.add_entity(vec![
                Box::new(AnotherTestComponent { prop1: 20 }),
                Box::new(EmptyComponent {}),
                Box::new(OtherTestComponent { prop1: 200 }),
            ]);

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![AnotherTestComponent::name(), OtherTestComponent::name()],
            );

            assert_eq!(result.len(), 1);
            assert!(result.contains(&entity2));
        }

        #[test]
        fn works_when_multiple_entities_have_all_provided_components() {
            let mut em = EntityManager::new();

            let entity1 = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(EmptyComponent {}),
            ]);
            em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(vec![Box::new(AnotherTestComponent { prop1: 10 })]);
            em.add_entity(vec![Box::new(EmptyComponent {})]);
            let entity2 = em.add_entity(vec![
                Box::new(TestComponent { prop1: 20 }),
                Box::new(EmptyComponent {}),
                Box::new(OtherTestComponent { prop1: 200 }),
            ]);

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![TestComponent::name(), EmptyComponent::name()],
            );

            assert_eq!(result.len(), 2);
            assert!(result.contains(&entity1));
            assert!(result.contains(&entity2));
        }
    }

    mod test_get_components_on_entity {
        use super::*;

        #[test]
        fn is_empty_for_non_existent_entity() {
            let em = EntityManager::new();

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![TestComponent::name()],
            );

            assert!(results.is_empty());
        }

        #[test]
        fn is_empty_for_entity_with_no_components() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![TestComponent::name()],
            );

            assert!(results.is_empty());
        }

        #[test]
        fn is_empty_when_no_search_components_are_provided() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![Box::new(EmptyComponent {})]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![],
            );

            assert!(results.is_empty());
        }

        #[test]
        fn works_when_searching_for_one_component_on_entity_with_that_component() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![TestComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(results.get::<TestComponent>().prop1, 10);
        }

        #[test]
        fn works_when_searching_for_one_component_on_entity_with_multiple_components() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(EmptyComponent {}),
            ]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![TestComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(results.get::<TestComponent>().prop1, 10);
        }

        #[test]
        fn works_when_searching_for_multiple_components_on_entity_with_one_component() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![Box::new(TestComponent { prop1: 10 })]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![TestComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(results.get::<TestComponent>().prop1, 10);
        }

        #[test]
        fn works_when_searching_for_multiple_components_on_entity_with_multiple_components() {
            let mut em = EntityManager::new();

            let entity = em.add_entity(vec![
                Box::new(TestComponent { prop1: 10 }),
                Box::new(EmptyComponent {}),
                Box::new(OtherTestComponent { prop1: 100 }),
                Box::new(AnotherTestComponent { prop1: 200 }),
            ]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &entity,
                &vec![
                    TestComponent::name(),
                    EmptyComponent::name(),
                    AnotherTestComponent::name(),
                ],
            );

            let test_component = results.get::<TestComponent>();

            let another_test_component = results.get::<AnotherTestComponent>();

            let empty_component_option = results.try_get::<EmptyComponent>();

            assert_eq!(results.len(), 3);
            assert_eq!(test_component.prop1, 10);
            assert_eq!(another_test_component.prop1, 200);
            assert!(empty_component_option.is_some());
        }
    }

    mod test_intersection {
        use super::*;

        #[test]
        fn is_empty_when_there_are_no_vectors() {
            let vecs: Vec<Vec<u32>> = vec![];

            let result = intersection(&vecs);

            assert!(result.is_empty());
        }

        #[test]
        fn is_empty_when_there_is_only_one_vector() {
            let vecs: Vec<Vec<u32>> = vec![vec![1, 4, 5]];

            let result = intersection(&vecs);

            assert!(result.is_empty());
        }

        #[test]
        fn is_empty_when_there_are_no_intersections() {
            let vecs: Vec<Vec<u32>> = vec![vec![1, 4, 5], vec![10, 20, 30]];

            let result = intersection(&vecs);

            assert!(result.is_empty());
        }

        #[test]
        fn reports_intersections_for_two_vectors() {
            let vecs: Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![4, 3, 5]];

            let result = intersection(&vecs);

            assert_eq!(result.len(), 1);
            assert!(result.contains(&&3));
        }

        #[test]
        fn reports_intersections_for_more_than_two_vectors() {
            let vecs: Vec<Vec<u32>> = vec![
                vec![1, 2, 3],
                vec![4, 3, 5],
                vec![5, 10, 20],
                vec![40, 10, 20, 30],
            ];

            let result = intersection(&vecs);

            assert_eq!(result.len(), 4);
            assert!(result.contains(&&3));
            assert!(result.contains(&&5));
            assert!(result.contains(&&10));
            assert!(result.contains(&&20));
        }
    }
}
