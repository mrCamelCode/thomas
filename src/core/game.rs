use super::Scene;

pub struct Game<'a> {
  active_scene: Scene<'a>,
}

impl<'a> Game<'a> {
  pub(crate) fn new() -> Game<'a> {
    Game {
      active_scene: Scene::new("Main"),
    }
  }

  pub fn active_scene(&self) -> &Scene<'a> {
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