use crate::io::Button;
use crate::io::Button::*;
use crate::{io::ControllerAccess, ram_controllers::*};
use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

mod std_nes_controller;

//pub use self::std_nes_controller::Mapper0;

#[enum_dispatch::enum_dispatch(ControllerEnum)]
pub trait Controller {
    fn read(&self) -> u8;
    fn write(&mut self, byte: u8);
    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>);
}

#[enum_dispatch::enum_dispatch]
#[derive(Serialize, Deserialize)]
pub enum ControllerEnum {
    StdNesController(self::std_nes_controller::StdNesController),
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerId {
    Controller1,
    Controller2,
}
#[derive(Serialize, Deserialize)]
pub struct Controllers {
    controller_1: ControllerState,
    controller_2: ControllerState,
    strobe: bool,
}

impl Default for Controllers {
    fn default() -> Self {
        Self {
            controller_1: Default::default(),
            controller_2: Default::default(),
            strobe: Default::default(),
        }
    }
}
fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(
        crate::io::DummyControllerAccessImplementation::new(),
    ))
}
#[derive(Serialize, Deserialize)]
struct ControllerState {
    id: ControllerId,
    #[serde(skip, default = "default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    button: RefCell<u8>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            id: ControllerId::Controller1,
            controller_access: Rc::new(RefCell::new(
                crate::io::DummyControllerAccessImplementation::new(),
            )),
            button: Default::default(),
        }
    }
}

impl ControllerState {
    fn read(&self, strobe: bool) -> u8 {
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
            if !strobe {
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
}

impl Controllers {
    pub fn new(controller_access: Rc<RefCell<dyn ControllerAccess>>) -> Self {
        Controllers {
            controller_1: ControllerState {
                id: ControllerId::Controller1,
                controller_access: controller_access.clone(),
                button: RefCell::new(0),
            },
            controller_2: ControllerState {
                id: ControllerId::Controller2,
                controller_access,
                button: RefCell::new(0),
            },
            strobe: true,
        }
    }
    pub fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_1.controller_access = controller_access.clone();
        self.controller_2.controller_access = controller_access;
    }
}

impl ReadInputRegisters for Controllers {
    fn read(&self, port: InputRegister) -> u8 {
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
            *self.controller_1.button.borrow_mut() = A as u8;
            *self.controller_2.button.borrow_mut() = A as u8;
        }
    }
}

impl ControllerRegisterAccess for Controllers {}
