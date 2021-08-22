use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum WriteAccessRegister {
    PpuCtrl = 0x2000,
    PpuMask = 0x2001,
    OamAddr = 0x2003,
    OamData = 0x2004,
    PpuScroll = 0x2005,
    PpuAddr = 0x2006,
    PpuData = 0x2007,
}

#[derive(TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum DmaWriteAccessRegister {
    OamDma = 0x4014,
}

#[derive(Copy, Clone, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u16"]
pub enum ReadAccessRegister {
    PpuStatus = 0x2002,
    OamData = 0x2004,
    PpuData = 0x2007,
}

pub trait WritePpuRegisters {
    fn write(&mut self, register: WriteAccessRegister, value: u8);
}

pub trait ReadPpuRegisters {
    fn read(&mut self, register: ReadAccessRegister) -> u8;
}

pub trait WriteOamDma {
    fn write_oam_dma(&mut self, data: [u8; 256]);
}

pub trait PpuRegisterAccess: WritePpuRegisters + WriteOamDma + ReadPpuRegisters {}
