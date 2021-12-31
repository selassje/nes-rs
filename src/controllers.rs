use crate::{io::ControllerAccess, ram_controllers::*};
use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

mod null_controller;
mod std_nes_controller;

use self::null_controller::NullController;
use self::std_nes_controller::StdNesController;

#[enum_dispatch::enum_dispatch(ControllerEnum)]
pub trait Controller {
    fn read(&self) -> u8;
    fn write(&mut self, byte: u8);
    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>);
}

#[derive(PartialEq)]
pub enum ControllerType {
    NullController,
    StdNesController,
}

#[enum_dispatch::enum_dispatch]
#[derive(Serialize, Deserialize)]
enum ControllerEnum {
    NullController(self::null_controller::NullController),
    StdNesController(self::std_nes_controller::StdNesController),
}

impl ControllerEnum {
    fn get_type(&self) -> ControllerType {
        match self {
            ControllerEnum::StdNesController(_) => ControllerType::StdNesController,
            ControllerEnum::NullController(_) => ControllerType::NullController,
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
#[derive(Serialize, Deserialize, Default)]
pub struct Controllers {
    controller_1: ControllerEnum,
    controller_2: ControllerEnum,
}

fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(
        crate::io::DummyControllerAccessImplementation::new(),
    ))
}

impl Controllers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_controller(
        &mut self,
        id: ControllerId,
        controller_type: ControllerType,
        controller_access: Rc<RefCell<dyn ControllerAccess>>,
    ) {
        let controller = match id {
            ControllerId::Controller1 => &mut self.controller_1,
            ControllerId::Controller2 => &mut self.controller_2,
        };

        if controller.get_type() != controller_type {
            *controller = Self::new_controller(id, controller_type);
            controller.set_controller_access(controller_access);
        }
    }

    fn new_controller(id: ControllerId, controller_type: ControllerType) -> ControllerEnum {
        match controller_type {
            ControllerType::StdNesController => {
                ControllerEnum::StdNesController(StdNesController::new(id))
            }
            ControllerType::NullController => ControllerEnum::NullController(NullController::new()),
        }
    }

    pub fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_1
            .set_controller_access(controller_access.clone());
        self.controller_2.set_controller_access(controller_access);
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

impl ControllerRegisterAccess for Controllers {}
