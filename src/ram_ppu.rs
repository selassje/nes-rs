use self::ReadAccessRegister::*;
use self::WriteAccessRegister::*;
use std::slice::Iter;

#[derive(Copy, Clone, Debug)]
pub enum WriteAccessRegister {
    PpuCtrl = 0x2000,
    PpuMask = 0x2001,
    OamAddr = 0x2003,
    OamData = 0x2004,
    PpuScroll = 0x2005,
    PpuAddr = 0x2006,
    PpuData = 0x2007,
}

impl WriteAccessRegister {
    pub fn iterator() -> Iter<'static, WriteAccessRegister> {
        static REGISTERS: [WriteAccessRegister; 7] = [
            PpuCtrl,
            PpuMask,
            OamAddr,
            WriteAccessRegister::OamData,
            PpuScroll,
            PpuAddr,
            WriteAccessRegister::PpuData,
        ];
        REGISTERS.iter()
    }
}

impl From<u16> for WriteAccessRegister {
    fn from(value: u16) -> Self {
        match value {
            0x2000 => PpuCtrl,
            0x2001 => PpuMask,
            0x2003 => OamAddr,
            0x2004 => WriteAccessRegister::OamData,
            0x2005 => PpuScroll,
            0x2006 => PpuAddr,
            0x2007 => WriteAccessRegister::PpuData,
            _ => panic!("Can't cast {} to WriteAccessRegister", value),
        }
    }
}

pub enum DmaWriteAccessRegister {
    OamDma = 0x4014,
}

#[derive(Copy, Clone)]
pub enum ReadAccessRegister {
    PpuStatus = 0x2002,
    OamData = 0x2004,
    PpuData = 0x2007,
}

impl ReadAccessRegister {
    pub fn iterator() -> Iter<'static, ReadAccessRegister> {
        static REGISTERS: [ReadAccessRegister; 3] = [
            PpuStatus,
            ReadAccessRegister::OamData,
            ReadAccessRegister::PpuData,
        ];
        REGISTERS.iter()
    }
}

impl From<u16> for ReadAccessRegister {
    fn from(value: u16) -> Self {
        match value {
            0x2002 => PpuStatus,
            0x2004 => ReadAccessRegister::OamData,
            0x2007 => ReadAccessRegister::PpuData,
            _ => panic!("Can't cast {} to ReadAccessRegister", value),
        }
    }
}

pub trait WritePpuRegisters {
    fn write(&mut self, register: WriteAccessRegister, value: u8) -> ();
}

pub trait ReadPpuRegisters {
    fn read(&mut self, register: ReadAccessRegister) -> u8;
}

pub trait WriteOamDma {
    fn write_oam_dma(&mut self, data: [u8; 256]) -> ();
}

pub trait PpuRegisterAccess: WritePpuRegisters + WriteOamDma + ReadPpuRegisters {}
