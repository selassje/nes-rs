use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum InputPort {
    Controller1 = 0x4016,
    Controller2 = 0x4017,
}

#[derive(Copy, Clone, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum OutputPort {
    Controllers1And2 = 0x4016,
}

pub trait ReadInputPorts {
    fn read(&self, port: InputPort) -> u8;
}

pub trait WriteOutputPorts {
    fn write(&mut self, port: OutputPort, value: u8) -> ();
}

pub trait ControllerPortsAccess: ReadInputPorts + WriteOutputPorts {}
