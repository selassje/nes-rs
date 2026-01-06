use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerId;
use crate::io::ControllerAccess;
use crate::nes::EmulationFrame;

#[derive(Serialize, Deserialize)]
pub struct Zapper {
    id: ControllerId,
    trigger_pressed: RefCell<bool>,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    frame_of_last_click: RefCell<u128>,
    x: RefCell<usize>,
    y: RefCell<usize>,
    right_button_pressed: RefCell<bool>,
    luminance: f32,
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
            right_button_pressed: RefCell::new(false),
            luminance: 0.0,
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
        if (mouse_click.left_button || mouse_click.right_button) && !*trigger_state {
            *self.x.borrow_mut() = mouse_click.x;
            *self.y.borrow_mut() = mouse_click.y;
            *trigger_state = true;
            *self.frame_of_last_click.borrow_mut() = current_frame;
            *self.right_button_pressed.borrow_mut() = mouse_click.right_button;
        }
        if self.frames_since_last_click(current_frame) >= 2 && *trigger_state {
            *trigger_state = false;
        }
        let mut light_bit = if self.luminance > 0.7 { 0b0000_0000 } else { 0b0000_1000 };
        if self.frames_since_last_click(current_frame) <= 4 || *self.right_button_pressed.borrow() {
            light_bit = 0b0000_1000;
        }
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

impl Zapper {
    pub fn update_luminance(&mut self, emulation_frame: &EmulationFrame) {
        let x = *self.x.borrow();
        let y = *self.y.borrow();
        let idx = (y * crate::common::FRAME_WIDTH + x) * 3;
        let pixels = emulation_frame.video.as_ref();
        let r = pixels[idx] as f32 / 255.0;
        let g = pixels[idx + 1] as f32 / 255.0;
        let b = pixels[idx + 2] as f32 / 255.0;
        self.luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
}
