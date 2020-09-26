use crate::{
    controllers::{Button, Controller},
    io::KeyCode,
    io::KeyboardAccess,
};
use std::iter::FromIterator;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type ButtonKeyMap = HashMap<Button, KeyCode>;

pub struct KeyboardController {
    button_key_map: ButtonKeyMap,
    keyboard_access: Rc<RefCell<dyn KeyboardAccess>>,
}

impl KeyboardController {
    pub fn get_default_keyboard_controller_player1(
        keyboard_access: Rc<RefCell<dyn KeyboardAccess>>,
    ) -> Self {
        KeyboardController {
            button_key_map: HashMap::from_iter(vec![
                (Button::A, KeyCode::Q),
                (Button::B, KeyCode::E),
                (Button::Select, KeyCode::C),
                (Button::Start, KeyCode::Space),
                (Button::Up, KeyCode::W),
                (Button::Down, KeyCode::S),
                (Button::Left, KeyCode::A),
                (Button::Right, KeyCode::D),
            ]),
            keyboard_access,
        }
    }

    pub fn get_default_keyboard_controller_player2(
        keyboard_access: Rc<RefCell<dyn KeyboardAccess>>,
    ) -> Self {
        KeyboardController {
            button_key_map: HashMap::from_iter(vec![
                (Button::A, KeyCode::Kp4),
                (Button::B, KeyCode::Kp5),
                (Button::Select, KeyCode::Kp6),
                (Button::Start, KeyCode::KpPlus),
                (Button::Up, KeyCode::Up),
                (Button::Down, KeyCode::Down),
                (Button::Left, KeyCode::Left),
                (Button::Right, KeyCode::Right),
            ]),
            keyboard_access,
        }
    }
}

impl Controller for KeyboardController {
    fn is_button_pressed(&self, button: Button) -> u8 {
        let key_code = self.button_key_map.get(&button).unwrap();
        if self.keyboard_access.borrow().is_key_pressed(*key_code) {
            1
        } else {
            0
        }
    }
}
