use device_query::{DeviceQuery, DeviceState, Keycode};
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
enum KeyState {
    Up,
    Down,
}

#[derive(Clone)]
struct KeyStateData {
    prev_state: KeyState,
    current_state: KeyState,
}

pub struct Input {
    keylogger: HashMap<Keycode, KeyStateData>,
    device_state: DeviceState,
}
impl Input {
    pub fn new() -> Self {
        Input {
            keylogger: HashMap::new(),
            device_state: DeviceState::new(),
        }
    }

    /// Whether the key was pressed down this frame.
    pub fn is_key_down(&self, keycode: &Keycode) -> bool {
        if let Some(key_state_data) = self.keylogger.get(keycode) {
            return key_state_data.current_state == KeyState::Down
                && key_state_data.prev_state == KeyState::Up;
        }

        false
    }

    /// Whether the key was released this frame.
    pub fn is_key_up(&self, keycode: &Keycode) -> bool {
        if let Some(key_state_data) = self.keylogger.get(keycode) {
            return key_state_data.current_state == KeyState::Up
                && key_state_data.prev_state == KeyState::Down;
        }

        false
    }

    /// Whether the key is pressed. This will return `true` on every frame while the key is pressed
    /// down. If you want to do something on the one frame in which the key was pressed down, use `is_key_down`.
    pub fn is_key_pressed(&self, keycode: &Keycode) -> bool {
        if let Some(key_state_data) = self.keylogger.get(keycode) {
            return key_state_data.current_state == KeyState::Down;
        }

        false
    }

    pub(crate) fn update(&mut self) {
        let current_keys = self.device_state.get_keys();

        self.keylogger.iter_mut().for_each(|(_, key_state_data)| {
            key_state_data.prev_state = key_state_data.current_state.clone();
            key_state_data.current_state = KeyState::Up;
        });

        current_keys.iter().for_each(|keycode| {
            if let Some(key_state_data) = self.keylogger.get(keycode) {
                let mut new_key_state_data = key_state_data.clone();

                new_key_state_data.current_state = KeyState::Down;

                self.keylogger.insert(keycode.clone(), new_key_state_data);
            } else {
                self.keylogger.insert(
                    keycode.clone(),
                    KeyStateData {
                        prev_state: KeyState::Up,
                        current_state: KeyState::Down,
                    },
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod key_state {
        use super::*;

        #[test]
        fn equality_works() {
            assert!(KeyState::Up == KeyState::Up);
            assert!(KeyState::Down == KeyState::Down);
        }

        #[test]
        fn inequality_works() {
            assert!(KeyState::Up != KeyState::Down);
        }
    }
}
