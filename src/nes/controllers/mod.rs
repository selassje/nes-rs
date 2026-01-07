use super::{ram_controllers::*, ControllerAccess};

use super::ControllerId;
use super::ControllerType;
use super::StdNesControllerButton;
use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

mod null_controller;
mod std_nes_controller;
mod zapper;

use self::null_controller::NullController;
use self::std_nes_controller::StdNesController;
use self::zapper::Zapper;

pub struct NullControllerAccess {}
impl NullControllerAccess {
    pub fn new() -> Self {
        Self {}
    }
}
impl ControllerAccess for NullControllerAccess {
    fn is_button_pressed(
        &self,
        _controller_id: ControllerId,
        _button: StdNesControllerButton,
    ) -> bool {
        false
    }
    fn is_zapper_trigger_pressed(&self) -> Option<super::ZapperTarget> {
        None
    }
}

#[enum_dispatch::enum_dispatch(ControllerEnum)]
pub trait Controller {
    fn read(&self, callbac: Option<&dyn ControllerAccess>) -> u8;
    fn write(&mut self, byte: u8);
    fn power_cycle(&mut self);
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
        Self::NullController(null_controller::NullController::new())
    }
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

#[derive(Serialize, Deserialize)]
pub struct Controllers {
    controller_1: ControllerEnum,
    controller_2: ControllerEnum,
}

impl Default for Controllers {
    fn default() -> Self {
        Self {
            controller_1: ControllerEnum::StdNesController(StdNesController::new(
                ControllerId::Controller1,
            )),
            controller_2: ControllerEnum::StdNesController(StdNesController::new(
                ControllerId::Controller2,
            )),
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
        }
    }
    pub fn power_cycle(&mut self) {
        self.controller_1.power_cycle();
        self.controller_2.power_cycle();
    }

    pub fn update_zappers(&mut self, emulation_frame: &super::EmulationFrame, frame: u128) {
        for controller in [&mut self.controller_1, &mut self.controller_2] {
            if let ControllerEnum::Zapper(zapper) = controller {
                zapper.update(emulation_frame, frame);
            }
        }
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
    fn read(&self, port: InputRegister, callback: Option<&dyn ControllerAccess>) -> u8 {
        match port {
            InputRegister::Controller1 => self.controller_1.read(callback),
            InputRegister::Controller2 => self.controller_2.read(callback),
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
