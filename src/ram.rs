use crate::ram_apu;
use crate::ram_controllers::*;
use crate::ram_ppu::*;
use crate::{mappers::Mapper, memory::*};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::ops::Range;
use std::rc::Rc;

const INTERNAL_START: u16 = 0x0000;
const INTERNAL_END: u16 = 0x2000;
const INTERNAL_MIRROR_SIZE: u16 = 0x0800;

const INTERNAL_RAM_RANGE: Range<u16> = Range {
    start: INTERNAL_START,
    end: INTERNAL_END,
};

const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_END: u16 = 0x4000;
const PPU_REGISTERS_MIRROR_SIZE: u16 = 0x0008;

const PPU_REGISTERS_RANGE: Range<u16> = Range {
    start: PPU_REGISTERS_START,
    end: PPU_REGISTERS_END,
};

const CARTRIDGE_SPACE_START: u32 = 0x4020;
const CARTRIDGE_SPACE_END: u32 = 0xFFFF + 1;

const CARTRIDGE_SPACE_RANGE: Range<u32> = Range {
    start: CARTRIDGE_SPACE_START,
    end: CARTRIDGE_SPACE_END,
};

pub struct RAM {
    memory: [u8; 65536],
    mapper: Rc<RefCell<dyn Mapper>>,
    ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
    controller_access: Rc<RefCell<dyn ControllerPortsAccess>>,
    apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
    dmc_sample_address: usize,
}

impl RAM {
    pub fn new(
        ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
        controller_access: Rc<RefCell<dyn ControllerPortsAccess>>,
        apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
        mapper: Rc<RefCell<dyn Mapper>>,
    ) -> RAM {
        RAM {
            memory: [0; 65536],
            mapper: mapper,
            ppu_access: ppu_access,
            controller_access: controller_access,
            apu_access: apu_access,
            dmc_sample_address: 0,
        }
    }

    pub fn reset(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
    }
    fn get_real_address(&self, address: u16) -> u16 {
        if PPU_REGISTERS_RANGE.contains(&address) {
            PPU_REGISTERS_START + (address % PPU_REGISTERS_MIRROR_SIZE)
        } else if INTERNAL_RAM_RANGE.contains(&address) {
            INTERNAL_START + (address % INTERNAL_MIRROR_SIZE)
        } else {
            address
        }
    }
}

impl Memory for RAM {
    fn get_byte(&self, address: u16) -> u8 {
        let addr = self.get_real_address(address);
        if let Ok(reg) = ReadAccessRegister::try_from(addr) {
            self.ppu_access.borrow_mut().read(reg)
        } else if let Ok(reg) = ram_apu::ReadAccessRegister::try_from(addr) {
            self.apu_access.borrow_mut().read(reg)
        } else if let Ok(input_port) = InputPort::try_from(addr) {
            self.controller_access.borrow_mut().read(input_port)
        } else if let Ok(_) = ReadAccessRegister::try_from(addr) {
            0
        } else if let Ok(_) = OutputPort::try_from(addr) {
            panic!(
                "Attempting to read from the controller output port {:#X}",
                addr
            );
        } else if let Ok(_) = ram_apu::WriteAccessRegister::try_from(addr) {
            panic!(
                "Attempting to read from a Apu write access register {:#X}",
                addr
            );
        } else if CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)) {
            self.mapper.borrow_mut().get_prg_byte(addr)
        } else {
            self.memory[addr as usize]
        }
    }

    fn store_byte(&mut self, address: u16, byte: u8) {
        let addr = self.get_real_address(address);
        if let Ok(reg) = WriteAccessRegister::try_from(addr) {
            self.ppu_access.borrow_mut().write(reg, byte);
        } else if let Ok(_) = DmaWriteAccessRegister::try_from(addr) {
            let mut dma_data = [0; 256];
            for (i, e) in dma_data.iter_mut().enumerate() {
                let page_adress = (byte as u16) << 8;
                *e = self.get_byte(page_adress + i as u16);
            }
            self.memory[addr as usize] = byte;
            self.ppu_access.borrow_mut().write_oam_dma(dma_data);
        } else if let Ok(output_port) = OutputPort::try_from(addr) {
            self.controller_access.borrow_mut().write(output_port, byte);
        } else if let Ok(reg) = ram_apu::WriteAccessRegister::try_from(addr) {
            self.apu_access.borrow_mut().write(reg, byte);
        } else if let Ok(_) = InputPort::try_from(addr) {
        } else if let Ok(_) = ReadAccessRegister::try_from(addr) {
            //panic!("Attempting to write to a read Ppu register");
        } else if let Ok(_) = ram_apu::ReadAccessRegister::try_from(addr) {
            //panic!("Attempting to write to a read Apu register");
        } else if CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)) {
            self.mapper.borrow_mut().store_prg_byte(addr, byte)
        } else {
            self.memory[addr as usize] = byte;
        }
    }
}

impl DmcMemory for RAM {
    fn set_sample_address(&mut self, address: u8) {
        self.dmc_sample_address = 0xC000 + (address as usize * 64);
    }

    fn get_next_sample_byte(&mut self) -> u8 {
        let byte = self.get_byte(self.dmc_sample_address as u16);
        self.dmc_sample_address = if self.dmc_sample_address == 0xFFFF {
            0x8000
        } else {
            self.dmc_sample_address + 1
        };
        byte
    }
}
