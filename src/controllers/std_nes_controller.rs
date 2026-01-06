use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc,fmt::Display};

use super::ControllerId;
use crate::StdNesControllerButton;
use crate::ControllerAccess;

#[derive(Serialize, Deserialize)]
pub struct StdNesController {
    id: ControllerId,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    button: RefCell<u8>,
    strobe: bool,
}

impl Default for StdNesController {
    fn default() -> Self {
        Self::new(ControllerId::Controller1)
    }
}

impl StdNesController {
    pub fn new(id: ControllerId) -> Self {
        Self {
            id,
            button: Default::default(),
            strobe: true,
            controller_access: super::default_controller_access(),
        }
    }
}

impl From<u8> for StdNesControllerButton {
    fn from(value: u8) -> Self {
        use StdNesControllerButton::*;
        match value {
            0 => A,
            1 => B,
            2 => Select,
            3 => Start,
            4 => Up,
            5 => Down,
            6 => Left,
            7 => Right,
            _ => panic!("Can't cast {} to Button", value),
        }
    }
}
impl Display for StdNesControllerButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StdNesControllerButton::A => "A",
            StdNesControllerButton::B => "B",
            StdNesControllerButton::Select => "Select",
            StdNesControllerButton::Start => "Start",
            StdNesControllerButton::Up => "Up",
            StdNesControllerButton::Down => "Down",
            StdNesControllerButton::Left => "Left",
            StdNesControllerButton::Right => "Right",
        })
    }
}
impl super::Controller for StdNesController {
    fn read(&self) -> u8 {
        0x40 | if *self.button.borrow() < 8 {
            let button = Into::<StdNesControllerButton>::into(*self.button.borrow());
            let mut val = self
                .controller_access
                .borrow()
                .is_button_pressed(self.id, button);
            if val
                && ((button == StdNesControllerButton::Left
                    && self
                        .controller_access
                        .borrow()
                        .is_button_pressed(self.id, StdNesControllerButton::Right))
                    || button == StdNesControllerButton::Down
                        && self
                            .controller_access
                            .borrow()
                            .is_button_pressed(self.id, StdNesControllerButton::Up))
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
            *self.button.borrow_mut() = StdNesControllerButton::A as u8;
        }
    }

    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access;
    }

    fn power_cycle(&mut self) {
        self.strobe = true;
        *self.button.borrow_mut() = Default::default()
    }
}
