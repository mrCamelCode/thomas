use super::{Scene, Input, Time};

pub struct Game {
  active_scene: Scene,
  input: Input,
  time: Time,
}

impl Game {
  pub fn new() -> Game {
    Game {
      active_scene: Scene::new("Main"),
      input: Input::new(),
      time: Time::new(),
    }
  }

  pub fn active_scene(&self) -> &Scene {
    &self.active_scene
  }
  pub fn active_scene_as_mut(&mut self) -> &mut Scene {
    &mut self.active_scene
  }

  pub fn input(&self) -> &Input {
    &self.input
  }

  pub fn change_scene(&mut self, new_scene: Scene) {
    self.active_scene = new_scene;
  }

  pub fn start(&mut self) {
    loop {
      self.input = Input::new();
      self.time = Time::new();

      self.input.update_keylogger();

      self.time.update();

      // run init and/or update on all behaviours of all entities in the current scene.
    }
  }
}