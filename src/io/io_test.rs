use std::collections::HashMap;

use crate::{controllers::Button, io::io_internal::IOInternal, keyboard::ButtonKeyMap, keyboard::KeyboardController};
use crate::io::{AudioAccess, KeyboardAccess, RgbColor, VideoAccess, IO};

use super::KeyCode;

pub struct IOTest {
    io_internal: IOInternal,
    player_1_button_key_map: ButtonKeyMap,
    player_2_button_key_map: ButtonKeyMap,
    keys_state: HashMap<KeyCode,bool>,
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            io_internal: IOInternal::new(),
            player_1_button_key_map: KeyboardController::get_default_player_1_mapping(),
            player_2_button_key_map: KeyboardController::get_default_player_2_mapping(),
            keys_state : HashMap::new(),
        }
    }
    pub fn dump_frame(&self, path: &str) {
        self.io_internal.dump_frame(path);
    }

    fn set_button_state(&mut self, button: Button, mapping : &ButtonKeyMap, state: bool ) {
        let key = mapping.get(&button).unwrap();
        self.keys_state.insert(*key, state);
    }    
    fn press_button(&mut self, button: Button, mapping : &ButtonKeyMap  ) {
        self.set_button_state(button, mapping, true);
    }    

    fn release_button(&mut self, button: Button, mapping : &ButtonKeyMap  ) {
        self.set_button_state(button, mapping, false);
    }
    
    pub fn press_button_player_1(&mut self, button: Button) {
        self.press_button(button, &self.player_1_button_key_map.clone());
    }
   
    pub fn press_button_player_2(&mut self, button: Button) {
        self.press_button(button, &self.player_2_button_key_map.clone());
    }

    pub fn release_button_player_1(&mut self, button: Button) {
        self.release_button(button, &self.player_1_button_key_map.clone());
    }
    
    pub fn release_button_player_2(&mut self, button: Button) {
        self.release_button(button, &self.player_2_button_key_map.clone());
    }
}

impl IO for IOTest {
    fn present_frame(&mut self) {}
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
