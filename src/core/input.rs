use std::{
    collections::HashMap,
    io,
    sync::mpsc::{self, Receiver},
    thread,
};

// TODO: The crate device_query should enable this service's existence.
pub struct Input {
    keylogger: HashMap<String, bool>
}

impl Input {
    pub fn new() -> Self {
        Input {
            keylogger: HashMap::new(),
        }
    }

    /// Whether the key was pressed down this frame.
    pub fn is_key_down(&self) -> bool {
        todo!();
    }

    /// Whether the key was let up this frame.
    pub fn is_key_up(&self) -> bool {
        todo!();
    }

    /// Whether the key was pressed this frame.
    pub fn is_key_pressed(&self) -> bool {
        todo!();
    }

    pub(crate) fn update(&mut self) {
        todo!();
    }
}
