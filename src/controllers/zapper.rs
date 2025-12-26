use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerId;
use crate::io::ControllerAccess;

#[derive(Serialize, Deserialize)]
pub struct Zapper {
    id: ControllerId,
    #[serde(skip, default = "super::default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
}

impl Zapper {
    pub fn new(id: ControllerId) -> Self {
        Self {
            id,
            controller_access: super::default_controller_access(),
        }
    }
}

impl super::Controller for Zapper {
    fn read(&self) -> u8 {
        todo!();
    }

    fn write(&mut self, byte: u8) {
        todo!();
    }

    fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controller_access = controller_access;
    }
}
