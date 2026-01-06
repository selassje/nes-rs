use std::collections::HashMap;

use crate::io::{ControllerAccess, IO};
use crate::{nes::ControllerId, nes::StdNesControllerButton};

use super::{IOControl, IOState};

pub struct IOTest {
    controller_buttons_state: [HashMap<StdNesControllerButton, bool>; 2],
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            controller_buttons_state: [HashMap::new(), HashMap::new()],
        }
    }
    pub fn set_button_state(&mut self, button: StdNesControllerButton, controller_id: ControllerId, state: bool) {
        self.controller_buttons_state[controller_id as usize].insert(button, state);
    }
}

impl IO for IOTest {
    fn present_frame(&mut self, _: IOControl, _: &crate::nes::EmulationFrame) -> IOState {
        Default::default()
    }

    fn is_audio_available(&self) -> bool {
        true
    }
}

impl ControllerAccess for IOTest {
    fn is_button_pressed(&self, controller_id: ControllerId, button: StdNesControllerButton) -> bool {
        if let Some(pressed) = self.controller_buttons_state[controller_id as usize].get(&button) {
            *pressed
        } else {
            false
        }
    }
    fn is_zapper_trigger_pressed(&self) -> Option<crate::nes::ZapperTarget> {
        None
    }
}
