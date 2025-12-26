use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerId;
use crate::io::ControllerAccess;

#[derive(PartialEq, Serialize, Deserialize)]
enum TriggerState {
    Released,
    HalfPull,
    FullPull,
}

#[derive(Serialize, Deserialize)]
pub struct Zapper {
    id: ControllerId,
    trigger_state: RefCell<TriggerState>,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    frame_of_last_click: RefCell<u128>,
}

impl Zapper {
    pub fn new(id: ControllerId) -> Self {
        Self {
            id,
            controller_access: super::default_controller_access(),
            trigger_state: RefCell::new(TriggerState::Released),
            frame_of_last_click: RefCell::new(0),
        }
    }

    fn frames_since_last_click(&self, current_frame: u128) -> u128 {
        if current_frame >= *self.frame_of_last_click.borrow() {
            current_frame - *self.frame_of_last_click.borrow()
        } else {
            u128::MAX - *self.frame_of_last_click.borrow() + current_frame
        }
    }
}

impl super::Controller for Zapper {
    fn read(&self) -> u8 {
        let current_frame = self.controller_access.borrow().get_current_frame();
        let mouse_click = self.controller_access.borrow().get_mouse_click();
        let mut trigger_state = self.trigger_state.borrow_mut();
        if mouse_click.is_some() {
            if *trigger_state == TriggerState::Released {
                *trigger_state = TriggerState::HalfPull;
                *self.frame_of_last_click.borrow_mut() = current_frame;
                println!("Shot!");
            }
        } else if *trigger_state == TriggerState::FullPull {
            *trigger_state = TriggerState::Released;
        }

        if self.frames_since_last_click(current_frame) >= 2
            && *trigger_state == TriggerState::HalfPull
        {
            *trigger_state = TriggerState::FullPull;
        }

        let result = match *trigger_state {
            TriggerState::Released => 0b0000_1000,
            TriggerState::HalfPull => 0b0001_0000,
            TriggerState::FullPull => 0b0000_1000,
        };
        result
    }

    fn write(&mut self, _byte: u8) {}

    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access;
    }
}
