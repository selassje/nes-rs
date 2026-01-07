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
    fn read(&self, _callback: Option<&dyn ControllerAccess>) -> u8 {
        0
    }
    fn write(&mut self, _byte: u8) {}


    fn power_cycle(&mut self) {}
}
