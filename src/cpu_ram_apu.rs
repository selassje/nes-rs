use self::WriteAccessRegister::*;
use std::slice::Iter;

#[derive(Copy, Clone, Debug)]
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

impl WriteAccessRegister {
    pub fn iterator() -> Iter<'static, WriteAccessRegister> {
        static REGISTERS: [WriteAccessRegister; 22] = [
            Pulse1_0,
            Pulse1_1,
            Pulse1_2,
            Pulse1_3,
            Pulse2_0,
            Pulse2_1,
            Pulse2_2,
            Pulse2_3,
            Triangle0,
            Triangle1,
            Triangle2,
            Triangle3,
            Noise0,
            Noise1,
            Noise2,
            Noise3,
            DMC0,
            DMC1,
            DMC2,
            DMC3,
            WriteAccessRegister::Status,
            FrameCounter,
        ];
        REGISTERS.iter()
    }
}

pub trait WriteAcessRegisters {
    fn write(&mut self, register: WriteAccessRegister, value: u8) -> ();
}

#[derive(Copy, Clone, Debug)]
pub enum ReadAccessRegister {
    Status = 0x4015,
}

impl ReadAccessRegister {
    pub fn iterator() -> Iter<'static, ReadAccessRegister> {
        static REGISTERS: [ReadAccessRegister; 1] = [ReadAccessRegister::Status];
        REGISTERS.iter()
    }
}

pub trait ReadAccessRegisters {
    fn read(&mut self, register: ReadAccessRegister) -> u8;
}

pub trait ApuRegisterAccess: WriteAcessRegisters + ReadAccessRegisters {}
