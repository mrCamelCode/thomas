use super::{renderer::Renderer, Input, Scene, SceneManager, Time};

pub struct GameUtil {
    input: Input,
    time: Time,
    scene_manager: SceneManager,
}

impl GameUtil {
    pub fn new() -> Self {
        GameUtil {
            input: Input::new(),
            time: Time::new(),
            scene_manager: SceneManager::new(Scene::new("default")),
        }
    }

    pub fn input(&mut self) -> &mut Input {
      &mut self.input
    }

    pub fn time(&mut self) -> &mut Time {
      &mut self.time
    }

    pub fn scene_manager(&mut self) -> &mut SceneManager {
      &mut self.scene_manager
    }
}

pub struct Game {
    util: GameUtil,
}

impl Game {
    pub fn new() -> Game {
        Game {
            util: GameUtil::new(),
        }
    }

    pub fn util(&mut self) -> &mut GameUtil {
      &mut self.util
    }

    pub fn start(&mut self, starting_scene: Scene, renderer: &dyn Renderer) {
        self.util = GameUtil::new();
        self.util.scene_manager = SceneManager::new(starting_scene);

        loop {
            self.util.input.update_keylogger();

            self.util.time.update();

            self.util.scene_manager.active_scene().update_entities();

            renderer.render();
        }
    }
}
