use super::{renderer::Renderer, Input, SceneManager, Time, Scene};

pub struct GameUtil {
    input: Input,
    time: Time,
}

impl GameUtil {
    pub fn new() -> Self {
        GameUtil {
            input: Input::new(),
            time: Time::new(),
        }
    }

    pub fn input(&mut self) -> &Input {
        &self.input
    }

    pub fn time(&mut self) -> &Time {
        &self.time
    }
}

pub struct Game;

impl Game {
    pub fn new() -> Game {
        Game {}
    }

    pub fn start(&mut self, starting_scene: Scene, renderer: Box<dyn Renderer>) {
        // TODO: Gotta think of some way to allow behaviours to access the game's scene manager instance mutably
        // so behaviours can trigger scene changes.

        let mut util = GameUtil::new();
        let mut scene_manager = SceneManager::new(starting_scene);

        loop {
            util.input.update();
            util.time.update();

            scene_manager.active_scene_as_mut().update_entities(&util);

            renderer.render();
        }
    }
}
