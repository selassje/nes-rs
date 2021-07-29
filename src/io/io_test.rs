use std::collections::HashMap;

use crate::io::{AudioAccess, ControllerAccess, RgbColor, VideoAccess, IO};
use crate::{controllers::Button, controllers::ControllerId, io::io_internal::IOInternal};

use super::{IOControl, IOState};

pub struct IOTest {
    io_internal: IOInternal,
    controller_buttons_state: [HashMap<Button, bool>; 2],
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            io_internal: IOInternal::new(),
            controller_buttons_state: [HashMap::new(), HashMap::new()],
        }
    }
    pub fn dump_frame(&self, path: &str) {
        self.io_internal.dump_frame(path);
    }

    pub fn set_button_state(&mut self, button: Button, controller_id: ControllerId, state: bool) {
        self.controller_buttons_state[controller_id as usize].insert(button, state);
    }
}

impl IO for IOTest {
    fn present_frame(&mut self, _: IOControl) -> IOState {
        Default::default()
    }

    fn is_audio_available(&self) -> bool {
        true
    }
}

impl AudioAccess for IOTest {
    fn add_sample(&mut self, _: crate::io::AudioSampleFormat) {}
}

impl VideoAccess for IOTest {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.io_internal.set_pixel(x, y, color);
    }
}

impl ControllerAccess for IOTest {
    fn is_button_pressed(&self, controller_id: ControllerId, button: Button) -> bool {
        if let Some(pressed) = self.controller_buttons_state[controller_id as usize].get(&button) {
            *pressed
        } else {
            false
        }
    }
}
