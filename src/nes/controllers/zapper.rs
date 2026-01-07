use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;

use super::ControllerAccess;
use super::ControllerId;
use crate::nes::EmulationFrame;
use crate::nes::ZapperTarget;

#[derive(Serialize, Deserialize)]
pub struct Zapper {
    id: ControllerId,
    trigger_pressed: RefCell<bool>,
    frame_of_last_trigger: RefCell<u128>,
    current_frame: u128,
    x: RefCell<usize>,
    y: RefCell<usize>,
    offscreen_targeted: RefCell<bool>,
    luminance: f32,
}

impl Zapper {
    pub fn new(id: ControllerId) -> Self {
        Self {
            id,
            trigger_pressed: RefCell::new(false),
            frame_of_last_trigger: RefCell::new(0),
            current_frame: 1,
            x: RefCell::new(0),
            y: RefCell::new(0),
            offscreen_targeted: RefCell::new(false),
            luminance: 0.0,
        }
    }

    fn frames_since_last_trigger(&self, current_frame: u128) -> u128 {
        if current_frame >= *self.frame_of_last_trigger.borrow() {
            current_frame - *self.frame_of_last_trigger.borrow()
        } else {
            u128::MAX - *self.frame_of_last_trigger.borrow() + current_frame
        }
    }
}

impl super::Controller for Zapper {
    fn read(&self, callback: Option<&dyn ControllerAccess>) -> u8 {
        let zapper_trigger = if let Some(cb) = callback {
            cb.is_zapper_trigger_pressed()
        } else {
            None
        };
        let mut trigger_state = self.trigger_pressed.borrow_mut();
        if zapper_trigger.is_some() && !*trigger_state {
            if let Some(ZapperTarget::OnScreen(x, y)) = zapper_trigger {
                *self.x.borrow_mut() = x as usize;
                *self.y.borrow_mut() = y as usize;
                *self.offscreen_targeted.borrow_mut() = false;
            } else {
                *self.offscreen_targeted.borrow_mut() = true;
            }
            *trigger_state = true;
            *self.frame_of_last_trigger.borrow_mut() = self.current_frame;
        }
        if self.frames_since_last_trigger(self.current_frame) >= 2 && *trigger_state {
            *trigger_state = false;
        }
        let mut light_bit = if self.luminance > 0.7 {
            0b0000_0000
        } else {
            0b0000_1000
        };
        if self.frames_since_last_trigger(self.current_frame) <= 4
            || *self.offscreen_targeted.borrow()
        {
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

    fn power_cycle(&mut self) {
        self.current_frame = 1;
        *self.trigger_pressed.borrow_mut() = false;
        *self.frame_of_last_trigger.borrow_mut() = 0;
        *self.x.borrow_mut() = 0;
        *self.y.borrow_mut() = 0;
        *self.offscreen_targeted.borrow_mut() = false;
        self.luminance = 0.0;
    }
}

impl Zapper {
    pub fn update(&mut self, emulation_frame: &EmulationFrame, frame: u128) {
        self.current_frame = frame;
        let x = *self.x.borrow();
        let y = *self.y.borrow();
        let idx = (y * crate::nes::FRAME_WIDTH + x) * 3;
        let pixels = emulation_frame.video.as_ref();
        let r = pixels[idx] as f32 / 255.0;
        let g = pixels[idx + 1] as f32 / 255.0;
        let b = pixels[idx + 2] as f32 / 255.0;
        self.luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
}
