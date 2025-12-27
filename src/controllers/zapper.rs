use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerId;
use crate::io::ControllerAccess;


#[derive(Serialize, Deserialize)]
pub struct Zapper {
    id: ControllerId,
    trigger_pressed: RefCell<bool>,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    frame_of_last_click: RefCell<u128>,
    x: RefCell<usize>,
    y: RefCell<usize>,
}

impl Zapper {
    pub fn new(id: ControllerId) -> Self {
        Self {
            id,
            controller_access: super::default_controller_access(),
            trigger_pressed: RefCell::new(false),
            frame_of_last_click: RefCell::new(0),
            x: RefCell::new(0),
            y: RefCell::new(0),
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
        let mut trigger_state = self.trigger_pressed.borrow_mut();
        if mouse_click.is_some() {
            if !*trigger_state {
                *self.x.borrow_mut() = mouse_click.as_ref().unwrap().x;
                *self.y.borrow_mut() = mouse_click.as_ref().unwrap().y;
                *trigger_state = true;
                *self.frame_of_last_click.borrow_mut() = current_frame;
                println!("Shot!");
            }
        }
        if self.frames_since_last_click(current_frame) >= 2 && *trigger_state {
            *trigger_state = false;
            println!("Released!	");
        }
        let lum = self
            .controller_access
            .borrow()
            .get_luminance(*self.x.borrow(), *self.y.borrow());
        let light_bit = if lum > 0.7 { 0b0000_0000 } else { 0b0000_1000 };
        light_bit
            | if *trigger_state {
                0b0001_0000
            } else {
                0b0000_0000
            }
    }

    fn write(&mut self, _byte: u8) {}

    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access;
    }
}
