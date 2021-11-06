use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerId;
use crate::io::Button;
use crate::io::ControllerAccess;

#[derive(Serialize, Deserialize)]
pub struct StdNesController {
    id: ControllerId,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    button: RefCell<u8>,
    strobe: bool,
}

impl super::Controller for StdNesController {
    fn read(&self) -> u8 {
        if *self.button.borrow() < 8 {
            let button = Into::<Button>::into(*self.button.borrow());
            let mut val = self
                .controller_access
                .borrow()
                .is_button_pressed(self.id, button);
            if val
                && ((button == Button::Left
                    && self
                        .controller_access
                        .borrow()
                        .is_button_pressed(self.id, Button::Right))
                    || button == Button::Down
                        && self
                            .controller_access
                            .borrow()
                            .is_button_pressed(self.id, Button::Up))
            {
                val = false;
            }
            if !self.strobe {
                *self.button.borrow_mut() += 1;
            }
            if val {
                1
            } else {
                0
            }
        } else {
            1
        }
    }

    fn write(&mut self, byte: u8) {
        self.strobe = (1 & byte) != 0;
        if self.strobe {
            *self.button.borrow_mut() = Button::A as u8;
        }
    }

    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access;
    }
}
