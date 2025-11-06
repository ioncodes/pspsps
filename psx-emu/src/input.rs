use psx_core::sio::joy::ControllerState;
use std::collections::HashSet;
use winit::keyboard::{Key, KeyCode, NamedKey};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
enum InputKey {
    Physical(KeyCode),
    Character(char),
}

pub struct InputState {
    pressed_keys: HashSet<InputKey>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &winit::event::KeyEvent) {
        let input_key = match &event.logical_key {
            Key::Character(s) => {
                if let Some(ch) = s.chars().next() {
                    InputKey::Character(ch.to_ascii_lowercase())
                } else {
                    return;
                }
            }
            Key::Named(NamedKey::ArrowUp) => InputKey::Physical(KeyCode::ArrowUp),
            Key::Named(NamedKey::ArrowDown) => InputKey::Physical(KeyCode::ArrowDown),
            Key::Named(NamedKey::ArrowLeft) => InputKey::Physical(KeyCode::ArrowLeft),
            Key::Named(NamedKey::ArrowRight) => InputKey::Physical(KeyCode::ArrowRight),
            Key::Named(NamedKey::Enter) => InputKey::Physical(KeyCode::Enter),
            Key::Named(NamedKey::Space) => InputKey::Physical(KeyCode::Space),
            _ => return,
        };

        match event.state {
            winit::event::ElementState::Pressed => {
                self.pressed_keys.insert(input_key);
            }
            winit::event::ElementState::Released => {
                self.pressed_keys.remove(&input_key);
            }
        }
    }

    pub fn get_controller_state(&self) -> ControllerState {
        ControllerState {
            // D-Pad: Arrow keys
            d_up: self.pressed_keys.contains(&InputKey::Physical(KeyCode::ArrowUp)),
            d_down: self.pressed_keys.contains(&InputKey::Physical(KeyCode::ArrowDown)),
            d_left: self.pressed_keys.contains(&InputKey::Physical(KeyCode::ArrowLeft)),
            d_right: self.pressed_keys.contains(&InputKey::Physical(KeyCode::ArrowRight)),

            // Action buttons
            cross: self.pressed_keys.contains(&InputKey::Character('y')),
            circle: self.pressed_keys.contains(&InputKey::Character('x')),
            square: self.pressed_keys.contains(&InputKey::Character('a')),
            triangle: self.pressed_keys.contains(&InputKey::Character('s')),

            // Shoulder buttons
            l1: self.pressed_keys.contains(&InputKey::Character('q')),
            l2: self.pressed_keys.contains(&InputKey::Character('w')),
            r1: self.pressed_keys.contains(&InputKey::Character('e')),
            r2: self.pressed_keys.contains(&InputKey::Character('r')),

            // System buttons
            start: self.pressed_keys.contains(&InputKey::Physical(KeyCode::Enter)),
            select: self.pressed_keys.contains(&InputKey::Physical(KeyCode::Backspace)),
        }
    }
}
