use crate::{io::ControllerAccess, ram_controllers::*};
use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

mod null_controller;
mod std_nes_controller;
mod zapper;

use self::null_controller::NullController;
use self::std_nes_controller::StdNesController;
use self::zapper::Zapper;

#[enum_dispatch::enum_dispatch(ControllerEnum)]
pub trait Controller {
    fn read(&self) -> u8;
    fn write(&mut self, byte: u8);
    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>);
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ControllerType {
    NullController,
    #[default]
    StdNesController,
    Zapper,
}

impl ControllerType {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(ControllerType::NullController),
            1 => Some(ControllerType::StdNesController),
            2 => Some(ControllerType::Zapper),
            _ => None,
        }
    }
}

#[enum_dispatch::enum_dispatch]
#[derive(Serialize, Deserialize)]
enum ControllerEnum {
    NullController(self::null_controller::NullController),
    StdNesController(self::std_nes_controller::StdNesController),
    Zapper(self::zapper::Zapper),
}

impl ControllerEnum {
    fn get_type(&self) -> ControllerType {
        match self {
            ControllerEnum::StdNesController(_) => ControllerType::StdNesController,
            ControllerEnum::NullController(_) => ControllerType::NullController,
            ControllerEnum::Zapper(_) => ControllerType::Zapper,
        }
    }
}

impl Default for ControllerEnum {
    fn default() -> Self {
        Self::NullController(NullController::new())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerId {
    Controller1,
    Controller2,
}

impl ControllerId {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(ControllerId::Controller1),
            1 => Some(ControllerId::Controller2),
            _ => None,
        }
    }
}
fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(
        crate::io::DummyControllerAccessImplementation::new(),
    ))
}

#[derive(Serialize, Deserialize)]
pub struct Controllers {
    controller_1: ControllerEnum,
    controller_2: ControllerEnum,
    #[serde(skip, default = "default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
}

impl Default for Controllers {
    fn default() -> Self {
        Self {
            controller_1: ControllerEnum::NullController(NullController::new()),
            controller_2: ControllerEnum::NullController(NullController::new()),
            controller_access: default_controller_access(),
        }
    }
}

impl Controllers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_controller(&mut self, id: ControllerId, controller_type: ControllerType) {
        let controller = match id {
            ControllerId::Controller1 => &mut self.controller_1,
            ControllerId::Controller2 => &mut self.controller_2,
        };

        if controller.get_type() != controller_type {
            *controller = Self::new_controller(id, controller_type);
            controller.set_controller_access(self.controller_access.clone());
        }
    }

    pub fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access.clone();
        self.controller_1
            .set_controller_access(controller_access.clone());
        self.controller_2.set_controller_access(controller_access);
    }
    pub fn get_controller_access(&self) -> Rc<RefCell<dyn ControllerAccess>> {
        self.controller_access.clone()
    }

    pub fn get_controller_type(&self, id: ControllerId) -> ControllerType {
        match id {
            ControllerId::Controller1 => self.controller_1.get_type(),
            ControllerId::Controller2 => self.controller_2.get_type(),
        }
    }

    fn new_controller(id: ControllerId, controller_type: ControllerType) -> ControllerEnum {
        match controller_type {
            ControllerType::StdNesController => {
                ControllerEnum::StdNesController(StdNesController::new(id))
            }
            ControllerType::NullController => ControllerEnum::NullController(NullController::new()),
            ControllerType::Zapper => ControllerEnum::Zapper(Zapper::new(id)),
        }
    }

}

impl ReadInputRegisters for Controllers {
    fn read(&self, port: InputRegister) -> u8 {
        match port {
            InputRegister::Controller1 => self.controller_1.read(),
            InputRegister::Controller2 => self.controller_2.read(),
        }
    }
}

impl WriteOutputRegisters for Controllers {
    fn write(&mut self, port: OutputRegister, value: u8) {
        assert!(port == OutputRegister::Controllers1And2);
        self.controller_1.write(value);
        self.controller_2.write(value);
    }
}
