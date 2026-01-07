use serde::Deserialize;
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

use super::ControllerAccess;

#[derive(Serialize, Deserialize)]
pub struct NullController {}

impl NullController {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Controller for NullController {
    fn read(&self) -> u8 {
        0
    }
    fn write(&mut self, _byte: u8) {}

    fn set_controller_access(&mut self, _controller_access: Rc<RefCell<dyn ControllerAccess>>) {}

    fn power_cycle(&mut self) {}
}
