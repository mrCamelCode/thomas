use crate::core::World;

pub(crate) trait PhysicsEngine {
    fn update(world: &mut World);
}
