use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use crate::{Component, Entity, Query, QueryResult, QueryResultList};

pub type StoredComponent = Rc<RefCell<Box<dyn Component>>>;
type EntitiesToComponents = HashMap<Entity, HashMap<String, StoredComponent>>;
type ComponentsToEntities = HashMap<String, BTreeSet<Entity>>;

pub struct EntityManager {
    entities_to_components: EntitiesToComponents,
    components_to_entities: ComponentsToEntities,
}
impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities_to_components: HashMap::new(),
            components_to_entities: HashMap::new(),
        }
    }

    pub fn add_entity(
        &mut self,
        entity: Entity,
        components: Vec<Box<dyn Component>>,
    ) -> Option<Entity> {
        if !self.entities_to_components.contains_key(&entity) {
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
                    let mut entity_set = BTreeSet::new();
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

            return Some(entity);
        }

        None
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        if let Some(component_map) = self.entities_to_components.remove(entity) {
            for component in component_map.values() {
                if let Some(entity_set) = self
                    .components_to_entities
                    .get_mut(component.borrow().component_name())
                {
                    entity_set.remove(entity);
                }
            }
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
                    let mut entity_set = BTreeSet::new();
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

    pub fn query(&self, query: &Query) -> QueryResultList {
        let allowed_component_names = query.component_names();
        let forbidden_component_names = query.forbidden_component_names();
        let entities_with_forbidden_components = Self::get_entities_with_components(
            &self.components_to_entities,
            &forbidden_component_names,
        );

        QueryResultList::new(
            Self::get_entities_with_components(
                &self.components_to_entities,
                &allowed_component_names,
            )
            .into_iter()
            .filter_map(|matching_entity| {
                if entities_with_forbidden_components.contains(&matching_entity) {
                    None
                } else {
                    Some(QueryResult {
                        entity: matching_entity,
                        components: Self::get_components_on_entity(
                            &self.entities_to_components,
                            &matching_entity,
                            &allowed_component_names,
                        ),
                    })
                }
            })
            .collect(),
        )
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

    fn get_components_on_entity(
        entities_to_components: &EntitiesToComponents,
        entity: &Entity,
        component_names: &Vec<&'static str>,
    ) -> Vec<StoredComponent> {
        Self::get_all_components_on_entity(entities_to_components, entity)
            .into_iter()
            .filter_map(|stored_component| {
                if component_names.contains(&stored_component.borrow().component_name()) {
                    Some(stored_component)
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_all_components_on_entity(
        entities_to_components: &EntitiesToComponents,
        entity: &Entity,
    ) -> Vec<StoredComponent> {
        if let Some(component_map) = entities_to_components.get(entity) {
            return component_map
                .values()
                .map(|stored_component| Rc::clone(stored_component))
                .collect();
        }

        vec![]
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

fn intersection<T: Ord>(vectors: &Vec<Vec<T>>) -> Vec<&T> {
    let mut values_tracker: BTreeSet<&T> = BTreeSet::new();
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

    mod test_add_entity {
        use super::*;

        #[test]
        fn returns_entity_and_adds_to_maps_when_entity_does_not_already_exist() {
            let mut em = EntityManager::new();

            let result = em
                .add_entity(Entity::new(), vec![])
                .expect("Entity addition should return ID of added entity.");

            let component_map = em.entities_to_components.get(&result);

            assert!(component_map.is_some());
            assert!(component_map.unwrap().is_empty());
            assert!(em.components_to_entities.is_empty());
        }

        #[test]
        fn returns_none_and_does_nothing_when_entity_already_exists() {
            let mut em = EntityManager::new();

            let entity = Entity(1);
            let entity_copy = Entity(1);

            em.add_entity(entity, vec![])
                .expect("Entity addition should return ID of added entity.");
            let result2 = em.add_entity(entity_copy, vec![]);

            let component_map = em.entities_to_components.get(&Entity(1));

            assert!(result2.is_none());

            assert!(component_map.is_some());
            assert!(component_map.unwrap().is_empty());
            assert!(em.components_to_entities.is_empty());
        }

        #[test]
        fn can_add_a_component_with_the_entity() {
            let mut em = EntityManager::new();

            let result = em
                .add_entity(Entity::new(), vec![Box::new(TestComponent { prop1: 1 })])
                .expect("Entity addition should return ID of added entity.");

            let component_map = em
                .entities_to_components
                .get(&result)
                .expect("The component map was added for the entity");

            let comp = component_map.get(TestComponent::name()).unwrap().borrow();
            let test_component =
                TestComponent::coerce(comp.as_ref()).expect("Component is TestComponent");

            assert!(component_map.get(TestComponent::name()).is_some());
            assert_eq!(test_component.prop1, 1);
        }

        #[test]
        fn can_add_multiple_components_with_the_entity() {
            let mut em = EntityManager::new();

            let result = em
                .add_entity(
                    Entity::new(),
                    vec![
                        Box::new(TestComponent { prop1: 1 }),
                        Box::new(OtherTestComponent { prop1: 3 }),
                        Box::new(AnotherTestComponent { prop1: 5 }),
                    ],
                )
                .expect("Entity addition should return ID of added entity.");

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
                TestComponent::coerce(comp1.as_ref()).expect("Component is TestComponent");
            let other_test_component = OtherTestComponent::coerce(comp2.as_ref())
                .expect("Component is OtherTestComponent");
            let another_test_component = AnotherTestComponent::coerce(comp3.as_ref())
                .expect("Component is AnotherTestComponent");

            assert!(component_map.get(TestComponent::name()).is_some());
            assert_eq!(test_component.prop1, 1);

            assert!(component_map.get(OtherTestComponent::name()).is_some());
            assert_eq!(other_test_component.prop1, 3);

            assert!(component_map.get(AnotherTestComponent::name()).is_some());
            assert_eq!(another_test_component.prop1, 5);
        }
    }

    mod test_remove_entity {
        use super::*;

        #[test]
        fn removing_a_nonexistent_entity_does_nothing() {
            let mut em = EntityManager::new();

            em.components_to_entities.insert(
                TestComponent::name().to_string(),
                BTreeSet::from([Entity(1)]),
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
                TestComponent::coerce(
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
                BTreeSet::from([Entity(1)]),
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
                BTreeSet::from([Entity(0)]),
            );

            em.add_component_to_entity(
                &Entity(0),
                Box::new(OtherTestComponent { prop1: 10 }) as Box<dyn Component>,
            );

            assert_eq!(em.components_to_entities.len(), 2);
            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(
                OtherTestComponent::coerce(
                    em.entities_to_components
                        .get(&Entity(0))
                        .expect("Entity 0 exists")
                        .get(OtherTestComponent::name())
                        .expect("OtherTestComponent is on Entity 0")
                        .borrow()
                        .as_ref()
                )
                .expect("OtherTestComponent could be coerced.")
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
                BTreeSet::from([Entity(0)]),
            );

            em.add_component_to_entity(
                &Entity(0),
                Box::new(TestComponent { prop1: 10 }) as Box<dyn Component>,
            );

            assert_eq!(em.components_to_entities.len(), 1);
            assert_eq!(em.entities_to_components.len(), 1);
            assert_eq!(
                TestComponent::coerce(
                    em.entities_to_components
                        .get(&Entity(0))
                        .expect("Entity 0 exists")
                        .get(TestComponent::name())
                        .expect("TestComponent is on Entity 0")
                        .borrow()
                        .as_ref()
                )
                .expect("TestComponent could be coerced.")
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
                BTreeSet::from([Entity(0)]),
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
                BTreeSet::from([Entity(0)]),
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
                BTreeSet::from([Entity(0)]),
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

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(&Query::new().has::<AnotherTestComponent>());

                assert_eq!((*query_results).len(), 1);
                assert_eq!(query_results.get(0).unwrap().entity, Entity(0));
            }

            #[test]
            fn complex_query_for_more_than_one_component_match_works() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                        Box::new(EmptyComponent {}),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(EmptyComponent {}),
                        Box::new(TestComponent { prop1: 1 }),
                    ],
                );

                let query_results =
                    em.query(&Query::new().has::<EmptyComponent>().has::<TestComponent>());

                assert_eq!((*query_results).len(), 2);
                assert!(query_results
                    .iter()
                    .find(|result| result.entity == Entity(0))
                    .is_some());
                assert!(query_results
                    .iter()
                    .find(|result| result.entity == Entity(2))
                    .is_some());
            }

            #[test]
            fn can_read_queried_components() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(&Query::new().has::<TestComponent>());

                assert_eq!((*query_results).len(), 2);

                for result in &*query_results {
                    match result.entity {
                        Entity(0) => assert_eq!(
                            TestComponent::coerce(result.components[0].borrow().as_ref())
                                .unwrap()
                                .prop1,
                            10
                        ),
                        Entity(1) => assert_eq!(
                            TestComponent::coerce(result.components[0].borrow().as_ref())
                                .unwrap()
                                .prop1,
                            20
                        ),
                        entity => {
                            panic!("Entity present in results that should not be: {:?}", entity)
                        }
                    }
                }
            }

            #[test]
            fn can_mutate_queried_components() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                        Box::new(OtherTestComponent { prop1: 20 }),
                    ],
                );
                em.add_entity(
                    Entity(1),
                    vec![
                        Box::new(TestComponent { prop1: 20 }),
                        Box::new(AnotherTestComponent { prop1: 2 }),
                    ],
                );

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has::<AnotherTestComponent>(),
                );

                assert_eq!(query_results.len(), 2);

                for result in &*query_results {
                    if result.entity == Entity(0) {
                        if let Some(raw_another_test_component) =
                            result.components.iter().find(|comp| {
                                comp.borrow().component_name() == AnotherTestComponent::name()
                            })
                        {
                            AnotherTestComponent::coerce_mut(
                                raw_another_test_component.borrow_mut().as_mut(),
                            )
                            .unwrap()
                            .prop1 = 50;
                        }

                        if let Some(raw_test_component) = result
                            .components
                            .iter()
                            .find(|comp| comp.borrow().component_name() == TestComponent::name())
                        {
                            TestComponent::coerce_mut(raw_test_component.borrow_mut().as_mut())
                                .unwrap()
                                .prop1 = 1;
                        }
                    } else if result.entity == Entity(1) {
                        if let Some(raw_test_component) = result
                            .components
                            .iter()
                            .find(|comp| comp.borrow().component_name() == TestComponent::name())
                        {
                            TestComponent::coerce_mut(raw_test_component.borrow_mut().as_mut())
                                .unwrap()
                                .prop1 = 240;
                        }
                    }
                }

                for result in &*query_results {
                    match result.entity {
                        Entity(0) => {
                            let test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == TestComponent::name()
                                })
                                .unwrap()
                                .borrow();
                            let another_test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == AnotherTestComponent::name()
                                })
                                .unwrap()
                                .borrow();

                            assert_eq!(
                                TestComponent::coerce(test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                1
                            );
                            assert_eq!(
                                AnotherTestComponent::coerce(another_test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                50
                            );
                        }
                        Entity(1) => {
                            let test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == TestComponent::name()
                                })
                                .unwrap()
                                .borrow();
                            let another_test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == AnotherTestComponent::name()
                                })
                                .unwrap()
                                .borrow();

                            assert_eq!(
                                TestComponent::coerce(test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                240
                            );
                            assert_eq!(
                                AnotherTestComponent::coerce(another_test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                2
                            );
                        }
                        entity => {
                            panic!("Entity present in results that should not be: {:?}", entity)
                        }
                    }
                }
            }
        }

        mod with_forbidden_components {
            use super::*;

            #[test]
            fn query_has_no_results_when_the_forbidden_components_remove_all_potential_matches() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);

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

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);
                assert_eq!(query_results.get(0).unwrap().entity, Entity(1));
            }

            #[test]
            fn complex_query_for_more_than_one_component_match_works() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                        Box::new(EmptyComponent {}),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(EmptyComponent {}),
                        Box::new(TestComponent { prop1: 1 }),
                    ],
                );

                let query_results = em.query(
                    &Query::new()
                        .has::<EmptyComponent>()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);
                assert!(query_results
                    .iter()
                    .find(|result| result.entity == Entity(2))
                    .is_some());
            }

            #[test]
            fn can_read_queried_components() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                    ],
                );
                em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 20 })]);

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has_no::<AnotherTestComponent>(),
                );

                assert_eq!((*query_results).len(), 1);

                for result in &*query_results {
                    match result.entity {
                        Entity(1) => assert_eq!(
                            TestComponent::coerce(result.components[0].borrow().as_ref())
                                .unwrap()
                                .prop1,
                            20
                        ),
                        entity => {
                            panic!("Entity present in results that should not be: {:?}", entity)
                        }
                    }
                }
            }

            #[test]
            fn can_mutate_queried_components() {
                let mut em = EntityManager::new();

                em.add_entity(
                    Entity(0),
                    vec![
                        Box::new(TestComponent { prop1: 10 }),
                        Box::new(AnotherTestComponent { prop1: 100 }),
                        Box::new(OtherTestComponent { prop1: 20 }),
                    ],
                );
                em.add_entity(
                    Entity(1),
                    vec![
                        Box::new(TestComponent { prop1: 20 }),
                        Box::new(AnotherTestComponent { prop1: 2 }),
                    ],
                );
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(TestComponent { prop1: 50 }),
                        Box::new(EmptyComponent {}),
                    ],
                );

                let query_results = em.query(
                    &Query::new()
                        .has::<TestComponent>()
                        .has::<AnotherTestComponent>()
                        .has_no::<EmptyComponent>(),
                );

                assert_eq!(query_results.len(), 2);

                for result in &*query_results {
                    if result.entity == Entity(0) {
                        if let Some(raw_another_test_component) =
                            result.components.iter().find(|comp| {
                                comp.borrow().component_name() == AnotherTestComponent::name()
                            })
                        {
                            AnotherTestComponent::coerce_mut(
                                raw_another_test_component.borrow_mut().as_mut(),
                            )
                            .unwrap()
                            .prop1 = 50;
                        }

                        if let Some(raw_test_component) = result
                            .components
                            .iter()
                            .find(|comp| comp.borrow().component_name() == TestComponent::name())
                        {
                            TestComponent::coerce_mut(raw_test_component.borrow_mut().as_mut())
                                .unwrap()
                                .prop1 = 1;
                        }
                    } else if result.entity == Entity(1) {
                        if let Some(raw_test_component) = result
                            .components
                            .iter()
                            .find(|comp| comp.borrow().component_name() == TestComponent::name())
                        {
                            TestComponent::coerce_mut(raw_test_component.borrow_mut().as_mut())
                                .unwrap()
                                .prop1 = 240;
                        }
                    }
                }

                for result in &*query_results {
                    match result.entity {
                        Entity(0) => {
                            let test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == TestComponent::name()
                                })
                                .unwrap()
                                .borrow();
                            let another_test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == AnotherTestComponent::name()
                                })
                                .unwrap()
                                .borrow();

                            assert_eq!(
                                TestComponent::coerce(test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                1
                            );
                            assert_eq!(
                                AnotherTestComponent::coerce(another_test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                50
                            );
                        }
                        Entity(1) => {
                            let test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == TestComponent::name()
                                })
                                .unwrap()
                                .borrow();
                            let another_test_component = result
                                .components
                                .iter()
                                .find(|comp| {
                                    comp.borrow().component_name() == AnotherTestComponent::name()
                                })
                                .unwrap()
                                .borrow();

                            assert_eq!(
                                TestComponent::coerce(test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                240
                            );
                            assert_eq!(
                                AnotherTestComponent::coerce(another_test_component.as_ref())
                                    .unwrap()
                                    .prop1,
                                2
                            );
                        }
                        entity => {
                            panic!("Entity present in results that should not be: {:?}", entity)
                        }
                    }
                }
            }
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

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                EmptyComponent::name(),
            );

            assert!(result.is_empty());
        }

        #[test]
        fn works_when_one_entity_matches() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(Entity(1), vec![Box::new(EmptyComponent {})]);

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                EmptyComponent::name(),
            );

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], Entity(1));
        }

        #[test]
        fn works_when_multiple_entities_match() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(
                Entity(1),
                vec![
                    Box::new(TestComponent { prop1: 100 }),
                    Box::new(EmptyComponent {}),
                ],
            );

            let result = EntityManager::get_entities_with_component(
                &em.components_to_entities,
                TestComponent::name(),
            );

            assert_eq!(result.len(), 2);
            assert!(result.contains(&Entity(0)) && result.contains(&Entity(1)));
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

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![EmptyComponent::name()],
            );

            assert!(result.is_empty());
        }

        #[test]
        fn returns_entity_list_for_component_when_searching_for_one_component() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(
                Entity(1),
                vec![
                    Box::new(AnotherTestComponent { prop1: 20 }),
                    Box::new(EmptyComponent {}),
                ],
            );

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![EmptyComponent::name()],
            );

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], Entity(1));
        }

        #[test]
        fn works_when_one_entity_has_all_provided_components() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(
                Entity(1),
                vec![
                    Box::new(AnotherTestComponent { prop1: 20 }),
                    Box::new(EmptyComponent {}),
                    Box::new(OtherTestComponent { prop1: 200 }),
                ],
            );

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![AnotherTestComponent::name(), OtherTestComponent::name()],
            );

            assert_eq!(result.len(), 1);
            assert!(result.contains(&Entity(1)));
        }

        #[test]
        fn works_when_multiple_entities_have_all_provided_components() {
            let mut em = EntityManager::new();

            em.add_entity(
                Entity(0),
                vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(EmptyComponent {}),
                ],
            );
            em.add_entity(Entity(1), vec![Box::new(TestComponent { prop1: 10 })]);
            em.add_entity(
                Entity(2),
                vec![Box::new(AnotherTestComponent { prop1: 10 })],
            );
            em.add_entity(Entity(3), vec![Box::new(EmptyComponent {})]);
            em.add_entity(
                Entity(4),
                vec![
                    Box::new(TestComponent { prop1: 20 }),
                    Box::new(EmptyComponent {}),
                    Box::new(OtherTestComponent { prop1: 200 }),
                ],
            );

            let result = EntityManager::get_entities_with_components(
                &em.components_to_entities,
                &vec![TestComponent::name(), EmptyComponent::name()],
            );

            assert_eq!(result.len(), 2);
            assert!(result.contains(&Entity(0)));
            assert!(result.contains(&Entity(4)));
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

            em.add_entity(Entity(0), vec![]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![TestComponent::name()],
            );

            assert!(results.is_empty());
        }

        #[test]
        fn is_empty_when_no_search_components_are_provided() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(EmptyComponent {})]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![],
            );

            assert!(results.is_empty());
        }

        #[test]
        fn works_when_searching_for_one_component_on_entity_with_that_component() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![TestComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(
                TestComponent::coerce(results[0].borrow().as_ref())
                    .expect("Only result is TestComponent")
                    .prop1,
                10
            );
        }

        #[test]
        fn works_when_searching_for_one_component_on_entity_with_multiple_components() {
            let mut em = EntityManager::new();

            em.add_entity(
                Entity(0),
                vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(EmptyComponent {}),
                ],
            );

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![TestComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(
                TestComponent::coerce(results[0].borrow().as_ref())
                    .expect("Only result is TestComponent")
                    .prop1,
                10
            );
        }

        #[test]
        fn works_when_searching_for_multiple_components_on_entity_with_one_component() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(TestComponent { prop1: 10 })]);

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![TestComponent::name(), EmptyComponent::name()],
            );

            assert_eq!(results.len(), 1);
            assert_eq!(
                TestComponent::coerce(results[0].borrow().as_ref())
                    .expect("Only result is TestComponent")
                    .prop1,
                10
            );
        }

        #[test]
        fn works_when_searching_for_multiple_components_on_entity_with_multiple_components() {
            let mut em = EntityManager::new();

            em.add_entity(
                Entity(0),
                vec![
                    Box::new(TestComponent { prop1: 10 }),
                    Box::new(EmptyComponent {}),
                    Box::new(OtherTestComponent { prop1: 100 }),
                    Box::new(AnotherTestComponent { prop1: 200 }),
                ],
            );

            let results = EntityManager::get_components_on_entity(
                &em.entities_to_components,
                &Entity(0),
                &vec![
                    TestComponent::name(),
                    EmptyComponent::name(),
                    AnotherTestComponent::name(),
                ],
            );

            let raw_test_component = results
                .iter()
                .find(|result| result.borrow().component_name() == TestComponent::name())
                .expect("Can find TestComponent")
                .borrow();
            let test_component = TestComponent::coerce(raw_test_component.as_ref()).unwrap();

            let raw_another_test_component = results
                .iter()
                .find(|result| result.borrow().component_name() == AnotherTestComponent::name())
                .expect("Can find AnotherTestComponent")
                .borrow();
            let another_test_component =
                AnotherTestComponent::coerce(raw_another_test_component.as_ref()).unwrap();

            let raw_empty_component = results
                .iter()
                .find(|result| result.borrow().component_name() == EmptyComponent::name())
                .expect("Can find EmptyComponent")
                .borrow();
            let empty_component_option = EmptyComponent::coerce(raw_empty_component.as_ref());

            assert_eq!(results.len(), 3);
            assert_eq!(test_component.prop1, 10);
            assert_eq!(another_test_component.prop1, 200);
            assert!(empty_component_option.is_some());
        }
    }

    mod test_get_all_components_on_entity {
        use super::*;

        #[test]
        fn is_empty_for_non_existent_entity() {
            let em = EntityManager::new();

            let result =
                EntityManager::get_all_components_on_entity(&em.entities_to_components, &Entity(0));

            assert!(result.is_empty());
        }

        #[test]
        fn works_for_entities_with_no_components() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![]);

            let result =
                EntityManager::get_all_components_on_entity(&em.entities_to_components, &Entity(0));

            assert!(result.is_empty());
        }

        #[test]
        fn works_for_entity_with_one_component() {
            let mut em = EntityManager::new();

            em.add_entity(Entity(0), vec![Box::new(EmptyComponent {})]);

            let result =
                EntityManager::get_all_components_on_entity(&em.entities_to_components, &Entity(0));

            assert_eq!(result.len(), 1);

            EmptyComponent::coerce(result[0].borrow().as_ref())
                .expect("The one component is an EmptyComponent");
        }

        #[test]
        fn works_for_entity_with_multiple_components() {
            let mut em = EntityManager::new();

            em.add_entity(
                Entity(0),
                vec![
                    Box::new(EmptyComponent {}),
                    Box::new(TestComponent { prop1: 10 }),
                ],
            );

            let result =
                EntityManager::get_all_components_on_entity(&em.entities_to_components, &Entity(0));

            assert_eq!(result.len(), 2);
            assert_eq!(
                TestComponent::coerce(
                    result
                        .iter()
                        .find(|comp| comp.borrow().component_name() == TestComponent::name())
                        .expect("It can find the TestComponent in the results")
                        .borrow()
                        .as_ref()
                )
                .unwrap()
                .prop1,
                10
            );
            assert!(result
                .iter()
                .find(|comp| comp.borrow().component_name() == EmptyComponent::name())
                .is_some())
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
