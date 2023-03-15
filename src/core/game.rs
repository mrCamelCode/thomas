use super::Scene;

pub struct Game {
  active_scene: Scene,
}

impl Game {
  pub fn new() -> Game {
    Game {
      active_scene: Scene::new("Main"),
    }
  }

  pub fn active_scene(&self) -> &Scene {
    &self.active_scene
  }

  pub fn start(&self) {
    loop {
      // Update Input service

      // Update Time service?

      // run init and/or update on all behaviours of all entities in the current scene.
    }
  }
}