use std::collections::HashMap;

use crate::io::{AudioAccess, KeyboardAccess, RgbColor, VideoAccess, IO};
use crate::{controllers::Button, io::io_internal::IOInternal, keyboard::ButtonKeyMap};

use super::{IOState, KeyCode};
#[derive(PartialEq)]
pub enum Player {
    Player1,
    _Player2,
}

pub struct IOTest {
    io_internal: IOInternal,
    player_1_button_key_map: ButtonKeyMap,
    player_2_button_key_map: ButtonKeyMap,
    keys_state: HashMap<KeyCode, bool>,
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            io_internal: IOInternal::new(),
            player_1_button_key_map: HashMap::new(),
            player_2_button_key_map: HashMap::new(),
            keys_state: HashMap::new(),
        }
    }
    pub fn dump_frame(&self, path: &str) {
        self.io_internal.dump_frame(path);
    }

    pub fn set_button_state(&mut self, button: Button, player1: Player, state: bool) {
        let mapping = if player1 == Player::Player1 {
            self.player_1_button_key_map.clone()
        } else {
            self.player_2_button_key_map.clone()
        };
        let key = mapping.get(&button).unwrap();
        self.keys_state.insert(*key, state);
    }

    pub fn set_key_mappings(
        &mut self,
        player_1_button_key_map: ButtonKeyMap,
        player_2_button_key_map: ButtonKeyMap,
    ) {
        self.player_1_button_key_map = player_1_button_key_map;
        self.player_2_button_key_map = player_2_button_key_map;
    }
}

impl IO for IOTest {
    fn present_frame(&mut self) -> IOState {
        Default::default()
    }
}

impl AudioAccess for IOTest {
    fn add_sample(&mut self, _: crate::io::SampleFormat) {}
}

impl VideoAccess for IOTest {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.io_internal.set_pixel(x, y, color);
    }
}

impl KeyboardAccess for IOTest {
    fn is_key_pressed(&self, key: crate::io::KeyCode) -> bool {
        let key_state = self.keys_state.get(&key);
        *key_state.unwrap_or(&false)
    }
}
