use std::convert::TryFrom;
#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum WriteAccessRegister {
    Pulse1_0 = 0x4000,
    Pulse1_1 = 0x4001,
    Pulse1_2 = 0x4002,
    Pulse1_3 = 0x4003,
    Pulse2_0 = 0x4004,
    Pulse2_1 = 0x4005,
    Pulse2_2 = 0x4006,
    Pulse2_3 = 0x4007,
    Triangle0 = 0x4008,
    Triangle1 = 0x4009,
    Triangle2 = 0x400A,
    Triangle3 = 0x400B,
    Noise0 = 0x400C,
    Noise1 = 0x400D,
    Noise2 = 0x400E,
    Noise3 = 0x400F,
    DMC0 = 0x4010,
    DMC1 = 0x4011,
    DMC2 = 0x4012,
    DMC3 = 0x4013,

    Status = 0x4015,
    FrameCounter = 0x4017,
}

pub trait WriteAcessRegisters {
    fn write(&mut self, register: WriteAccessRegister, value: u8);
}

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum ReadAccessRegister {
    Status = 0x4015,
}

pub trait ReadAccessRegisters {
    fn read(&mut self, register: ReadAccessRegister) -> u8;
}

pub trait ApuRegisterAccess: WriteAcessRegisters + ReadAccessRegisters {}

pub struct DummyApuRegisterAccessImpl {}

impl DummyApuRegisterAccessImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl WriteAcessRegisters for DummyApuRegisterAccessImpl {
    fn write(&mut self, _register: WriteAccessRegister, _value: u8) {
        todo!()
    }
}

impl ReadAccessRegisters for DummyApuRegisterAccessImpl {
    fn read(&mut self, _register: ReadAccessRegister) -> u8 {
        todo!()
    }
}

impl ApuRegisterAccess for DummyApuRegisterAccessImpl {}
