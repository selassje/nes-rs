use std::slice::Iter;
use self::WriteAccessRegister::*;
use self::ReadAccessRegister::*;

#[derive(Copy,Clone,Debug)]
pub enum WriteAccessRegister {
    PpuCtrl   = 0x2000,
    PpuMask   = 0x2001,
    OamAddr   = 0x2003,
    OamData   = 0x2004,
    PpuScroll = 0x2005,
    PpuAddr   = 0x2006,
    PpuData   = 0x2007,
}

impl WriteAccessRegister {
    pub fn iterator() -> Iter<'static, WriteAccessRegister> {
        static REGISTERS: [WriteAccessRegister; 7] = [PpuCtrl, PpuMask, OamAddr, WriteAccessRegister::OamData, PpuScroll, PpuAddr, WriteAccessRegister::PpuData];
        REGISTERS.iter()
    }
}

pub enum DmaWriteAccessRegister {
    OamDma  = 0x4014
}

#[derive(Copy,Clone)]
pub enum ReadAccessRegister {
    PpuStatus = 0x2002,
    OamData   = 0x2004,
    PpuData   = 0x2007,
}

impl ReadAccessRegister {
    pub fn iterator() -> Iter<'static, ReadAccessRegister> {
        static REGISTERS: [ReadAccessRegister; 3] = [PpuStatus, ReadAccessRegister::OamData, ReadAccessRegister::PpuData];
        REGISTERS.iter()
    }
}

pub trait WritePpuRegisters {
    fn write(&mut self , register : WriteAccessRegister, value : u8) -> ();
}

pub trait ReadPpuRegisters {
    fn read(&mut self, register : ReadAccessRegister) -> u8;
}

pub trait WriteOamDma {
    fn write_oam_dma(&mut self , data: [u8;256]) -> ();
}

pub trait PpuRegisterAccess : WritePpuRegisters + WriteOamDma + ReadPpuRegisters {

}