use std::collections::{BTreeSet, HashMap};

use crate::{Component, Entity, Query};

pub struct EntityManager {
    entities_to_components: HashMap<Entity, HashMap<String, Box<dyn Component>>>,
    components_to_entities: HashMap<String, BTreeSet<Entity>>,
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
                component_map.insert(component.component_name().to_string(), component);
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
                    .get_mut(component.component_name())
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

                component_map.insert(component_name.to_string(), component);

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
            if let Some(component) = component_map.remove(component_name) {
                if let Some(entity_set) = self.components_to_entities.get_mut(component_name) {
                    entity_set.remove(entity);
                }
            }
        }
    }

    // pub fn query(&self, query: Query) -> impl Iterator<Item = &dyn Component> {
    //     todo!("implement")
    // }

    // pub fn query_mut(&mut self, query: Query) -> impl Iterator<Item = &mut dyn Component> {
    //     todo!("implement")
    // }
}

fn entity_has_component(
    components_to_entities: &HashMap<String, BTreeSet<Entity>>,
    entity: &Entity,
    component: &Box<dyn Component>,
) -> bool {
    if let Some(entity_set) = components_to_entities.get(component.component_name()) {
        return entity_set.contains(&entity);
    }

    false
}
#[cfg(test)]
mod tests {
    use super::*;

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

            let comp = component_map.get(TestComponent::name()).unwrap();
            let test_component = TestComponent::coerce(comp).expect("Component is TestComponent");

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
                component_map.get(TestComponent::name()).unwrap(),
                component_map.get(OtherTestComponent::name()).unwrap(),
                component_map.get(AnotherTestComponent::name()).unwrap(),
            );

            let test_component = TestComponent::coerce(comp1).expect("Component is TestComponent");
            let other_test_component =
                OtherTestComponent::coerce(comp2).expect("Component is OtherTestComponent");
            let another_test_component =
                AnotherTestComponent::coerce(comp3).expect("Component is AnotherTestComponent");

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
                    Box::new(TestComponent { prop1: 1 }) as Box<dyn Component>,
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
                TestComponent::coerce(component_map.get(TestComponent::name()).unwrap())
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
                    Box::new(TestComponent { prop1: 1 }) as Box<dyn Component>,
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
                    Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
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
                    Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
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
                    Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
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
                    Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
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
                    Box::new(TestComponent { prop1: 5 }) as Box<dyn Component>,
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
}
