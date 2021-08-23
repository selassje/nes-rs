use self::Button::*;
use crate::{io::ControllerAccess, ram_controllers::*};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ControllerId {
    Controller1,
    Controller2,
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
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

impl Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            A => "A",
            B => "B",
            Select => "Select",
            Start => "Start",
            Up => "Up",
            Down => "Down",
            Left => "Left",
            Right => "Right",
        })
    }
}

pub struct Controllers {
    controller_1: ControllerState,
    controller_2: ControllerState,
    strobe: bool,
}

struct ControllerState {
    id: ControllerId,
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    button: u8,
}

impl ControllerState {
    fn read(&mut self, strobe: bool) -> u8 {
        if self.button < 8 {
            let button: Button = self.button.into();
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
            if !strobe {
                self.button += 1;
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
}

impl Controllers {
    pub fn new(controller_access: Rc<RefCell<dyn ControllerAccess>>) -> Self {
        Controllers {
            controller_1: ControllerState {
                id: ControllerId::Controller1,
                controller_access: controller_access.clone(),
                button: 0,
            },
            controller_2: ControllerState {
                id: ControllerId::Controller2,
                controller_access,
                button: 0,
            },
            strobe: true,
        }
    }
}

impl ReadInputRegisters for Controllers {
    fn read(&mut self, port: InputRegister) -> u8 {
        0x40 | match port {
            InputRegister::Controller1 => self.controller_1.read(self.strobe),
            InputRegister::Controller2 => self.controller_2.read(self.strobe),
        }
    }
}

impl WriteOutputRegisters for Controllers {
    fn write(&mut self, port: OutputRegister, value: u8) {
        assert!(port == OutputRegister::Controllers1And2);
        self.strobe = (1 & value) != 0;
        if self.strobe {
            self.controller_1.button = A as u8;
            self.controller_2.button = A as u8;
        }
    }
}

impl ControllerRegisterAccess for Controllers {}
