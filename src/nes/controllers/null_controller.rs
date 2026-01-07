use serde::Deserialize;
use serde::Serialize;

use super::ControllerCallback;

#[derive(Serialize, Deserialize)]
pub struct NullController {}

impl NullController {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Controller for NullController {
    fn read(&self, _callback: Option<&dyn ControllerCallback>) -> u8 {
        0
    }
    fn write(&mut self, _byte: u8) {}


    fn power_cycle(&mut self) {}
}
