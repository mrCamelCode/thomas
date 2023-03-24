use std::rc::Rc;

use super::{renderer::{Renderer, TerminalRenderer}, Input, Scene, SceneManager, Time};

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

    pub fn input(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn time(&mut self) -> &mut Time {
        &mut self.time
    }
}

pub struct Game;

impl Game {
    pub fn new() -> Game {
        Game {}
    }

    pub fn start(&mut self, scene_manager: &mut SceneManager, renderer: Box<dyn Renderer>) {
        let mut util = GameUtil::new();

        loop {
            util.input.update_keylogger();

            util.time.update();

            scene_manager.active_scene_as_mut().update_entities(&util);

            renderer.render();
        }
    }
}
