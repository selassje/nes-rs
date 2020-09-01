pub enum KeyEvent {
    KeyDown(u8),
    KeyUp(u8),
}

use crate::controllers::{Button, Controller};
use crate::io_sdl::get_key_status;
use sdl2::keyboard;
use std::collections::HashMap;
use std::iter::FromIterator;

type Keycode = keyboard::Scancode;

type ButtonKeyMap = HashMap<Button, Keycode>;

pub struct KeyboardController {
    button_key_map: ButtonKeyMap,
}

impl KeyboardController {
    pub fn get_default_keyboard_controller_player1() -> Self {
        KeyboardController {
            button_key_map: HashMap::from_iter(vec![
                (Button::A, Keycode::Z),
                (Button::B, Keycode::X),
                (Button::Select, Keycode::C),
                (Button::Start, Keycode::V),
                (Button::Up, Keycode::W),
                (Button::Down, Keycode::S),
                (Button::Left, Keycode::A),
                (Button::Right, Keycode::D),
            ]),
        }
    }

    pub fn get_default_keyboard_controller_player2() -> Self {
        KeyboardController {
            button_key_map: HashMap::from_iter(vec![
                (Button::A, Keycode::Kp4),
                (Button::B, Keycode::Kp5),
                (Button::Select, Keycode::Kp6),
                (Button::Start, Keycode::KpPlus),
                (Button::Up, Keycode::Up),
                (Button::Down, Keycode::Down),
                (Button::Left, Keycode::Left),
                (Button::Right, Keycode::Right),
            ]),
        }
    }
}

impl Controller for KeyboardController {
    fn is_button_pressed(&self, button: Button) -> u8 {
        let key_code = self.button_key_map.get(&button).unwrap();
        if get_key_status(*key_code) {
            1
        } else {
            0
        }
    }
}
