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
                        entity_set.insert(Entity::copy(&entity));
                    }
                } else {
                    let mut entity_set = BTreeSet::new();
                    entity_set.insert(Entity::copy(&entity));

                    self.components_to_entities
                        .insert(component.component_name().to_string(), entity_set);
                }
            }

            let mut component_map = HashMap::new();

            for component in components {
                component_map.insert(component.component_name().to_string(), component);
            }

            self.entities_to_components
                .insert(Entity::copy(&entity), component_map);

            return Some(Entity::copy(&entity));
        }

        None
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(components) = self.entities_to_components.remove(&entity) {
            for component in components.values() {
                if let Some(entity_set) = self
                    .components_to_entities
                    .get_mut(component.component_name())
                {
                    entity_set.remove(&entity);
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
                    entity_set.insert(Entity::copy(&entity));
                } else {
                    let mut entity_set = BTreeSet::new();
                    entity_set.insert(Entity::copy(&entity));

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

            let entity = Entity::new();
            let entity_copy = Entity::copy(&entity);

            let result1 = em
                .add_entity(entity, vec![])
                .expect("Entity addition should return ID of added entity.");
            let result2 = em.add_entity(entity_copy, vec![]);

            let component_map = em.entities_to_components.get(&result1);

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
    }

    mod test_add_component_to_entity {
        use super::*;
    }

    mod test_remove_component_from_entity {
        use super::*;
    }
}
