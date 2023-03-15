use std::collections::HashMap;

pub struct Input {
    // TODO: This will likely have to store more information than a bool.
    keylogger: HashMap<String, bool>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keylogger: HashMap::new(),
        }
    }

    /// Whether the key was pressed down this frame.
    pub fn is_key_down(&self) -> bool {
        false
    }

    /// Whether the key was let up this frame.
    pub fn is_key_up(&self) -> bool {
        false
    }

    /// Whether the key was pressed this frame.
    pub fn is_key_pressed(&self) -> bool {
        false
    }


    pub(crate) fn update_keylogger(&mut self) {

    }
}
