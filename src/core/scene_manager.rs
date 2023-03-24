use super::Scene;

pub struct SceneManager {
    active_scene: Scene,
}

impl SceneManager {
    pub fn new(active_scene: Scene) -> Self {
      SceneManager { active_scene }
    }

    pub fn active_scene(&self) -> &Scene {
        &self.active_scene
    }
    pub(crate) fn active_scene_as_mut(&mut self) -> &mut Scene {
        &mut self.active_scene
    }

    pub fn change_scene(&mut self, new_scene: Scene) {
        self.active_scene = new_scene;
    }
}
