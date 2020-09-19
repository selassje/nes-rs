use self::InputPort::*;
use self::OutputPort::*;
use std::slice::Iter;
#[derive(Copy, Clone, Debug)]
pub enum InputPort {
    Controller1 = 0x4016,
    Controller2 = 0x4017,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputPort {
    Controllers1And2 = 0x4016,
}

impl InputPort {
    pub fn iterator() -> Iter<'static, InputPort> {
        static CONTROLLERS: [InputPort; 2] = [Controller1, Controller2];
        CONTROLLERS.iter()
    }
}

impl OutputPort {
    pub fn iterator() -> Iter<'static, OutputPort> {
        static CONTROLLERS: [OutputPort; 1] = [Controllers1And2];
        CONTROLLERS.iter()
    }
}

pub trait ReadInputPorts {
    fn read(&self, port: InputPort) -> u8;
}

pub trait WriteOutputPorts {
    fn write(&mut self, port: OutputPort, value: u8) -> ();
}

pub trait ControllerPortsAccess: ReadInputPorts + WriteOutputPorts {}
