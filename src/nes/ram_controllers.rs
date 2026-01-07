use enum_tryfrom_derive::TryFromPrimitive;
use std::convert::TryFrom;

use crate::ControllerCallback;

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum InputRegister {
    Controller1 = 0x4016,
    Controller2 = 0x4017,
}

#[derive(Copy, Clone, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum OutputRegister {
    Controllers1And2 = 0x4016,
}

pub trait ReadInputRegisters {
    fn read(&self, port: InputRegister,callback: Option<&dyn ControllerCallback>) -> u8;
}

pub trait WriteOutputRegisters {
    fn write(&mut self, port: OutputRegister, value: u8);
}
