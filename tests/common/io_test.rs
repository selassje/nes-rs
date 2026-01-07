use std::collections::HashMap;

use nes_rs::{ControllerCallback, ControllerId, StdNesControllerButton};

pub struct IOTest {
    controller_buttons_state: [HashMap<StdNesControllerButton, bool>; 2],
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            controller_buttons_state: [HashMap::new(), HashMap::new()],
        }
    }
    pub fn set_button_state(
        &mut self,
        button: StdNesControllerButton,
        controller_id: ControllerId,
        state: bool,
    ) {
        self.controller_buttons_state[controller_id as usize].insert(button, state);
    }
}

impl ControllerCallback for IOTest {
    fn is_button_pressed(
        &self,
        controller_id: ControllerId,
        button: StdNesControllerButton,
    ) -> bool {
        if let Some(pressed) = self.controller_buttons_state[controller_id as usize].get(&button) {
            *pressed
        } else {
            false
        }
    }
    fn is_zapper_trigger_pressed(&self, _ : ControllerId) -> Option<nes_rs::ZapperTarget> {
        None
    }
}
