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

const CPU_TEST_MODE_SPACE_START: u16 = 0x4018;

const CARTRIDGE_SPACE_START: u32 = 0x4020;
const CARTRIDGE_SPACE_END: u32 = 0xFFFF + 1;

const CARTRIDGE_SPACE_RANGE: Range<u32> = Range {
    start: CARTRIDGE_SPACE_START,
    end: CARTRIDGE_SPACE_END,
};

type RegisterLatch = RefCell<u8>;

pub struct Ram {
    memory: [u8; 0x0808],
    mapper: Rc<RefCell<dyn Mapper>>,
    ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
    controller_access: Rc<RefCell<dyn ControllerRegisterAccess>>,
    apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
    dmc_sample_address: usize,
    ppu_register_latch: RegisterLatch,
    apu_register_latch: RegisterLatch,
    controller_register_latch: RegisterLatch,
    oam_dma_register_latch: RegisterLatch,
}

impl Ram {
    pub fn new(
        ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
        controller_access: Rc<RefCell<dyn ControllerRegisterAccess>>,
        apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
        mapper: Rc<RefCell<dyn Mapper>>,
    ) -> Ram {
        Ram {
            memory: [0; 0x0808],
            mapper,
            ppu_access,
            controller_access,
            apu_access,
            dmc_sample_address: 0,
            ppu_register_latch: RegisterLatch::new(0),
            apu_register_latch: RegisterLatch::new(0),
            controller_register_latch: RegisterLatch::new(0),
            oam_dma_register_latch: RegisterLatch::new(0),
        }
    }

    pub fn set_mapper(&mut self, mapper: Rc<RefCell<dyn Mapper>>) {
        self.mapper = mapper;
    }

    pub fn power_cycle(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
        *self.ppu_register_latch.borrow_mut() = 0;
        *self.apu_register_latch.borrow_mut() = 0;
        *self.controller_register_latch.borrow_mut() = 0;
        *self.oam_dma_register_latch.borrow_mut() = 0;
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

impl Memory for Ram {
    fn get_byte(&self, address_org: u16) -> u8 {
        let addr = self.get_real_address(address_org);
        if let Ok(reg) = ReadAccessRegister::try_from(addr) {
            let mut ppu_register_value = self.ppu_access.borrow_mut().read(reg);
            if reg == ReadAccessRegister::PpuStatus {
                const LOW_5_BITS: u8 = 0b00011111;
                ppu_register_value &= !LOW_5_BITS;
                ppu_register_value |= *self.ppu_register_latch.borrow() & LOW_5_BITS
            }
            *self.ppu_register_latch.borrow_mut() = ppu_register_value;
            *self.ppu_register_latch.borrow()
        } else if let Ok(reg) = ram_apu::ReadAccessRegister::try_from(addr) {
            *self.apu_register_latch.borrow_mut() = self.apu_access.borrow_mut().read(reg);
            *self.apu_register_latch.borrow()
        } else if let Ok(input_port) = InputRegister::try_from(addr) {
            *self.controller_register_latch.borrow_mut() =
                self.controller_access.borrow_mut().read(input_port);
            *self.controller_register_latch.borrow()
        } else if WriteAccessRegister::try_from(addr).is_ok() {
            *self.ppu_register_latch.borrow_mut()
        } else if DmaWriteAccessRegister::try_from(addr).is_ok() {
            *self.oam_dma_register_latch.borrow()
        } else if OutputRegister::try_from(addr).is_ok() {
            *self.controller_register_latch.borrow()
        } else if ram_apu::WriteAccessRegister::try_from(addr).is_ok() {
            *self.apu_register_latch.borrow()
        } else if CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)) {
            self.mapper.borrow_mut().get_prg_byte(addr)
        } else if addr >= CPU_TEST_MODE_SPACE_START {
            self.memory[(INTERNAL_MIRROR_SIZE + addr - CPU_TEST_MODE_SPACE_START) as usize]
        } else if addr < INTERNAL_MIRROR_SIZE {
            self.memory[addr as usize]
        } else {
            panic!("Address org {:X} real {:X}", address_org, addr);
        }
    }

    fn store_byte(&mut self, address: u16, byte: u8) {
        let addr = self.get_real_address(address);
        if let Ok(reg) = WriteAccessRegister::try_from(addr) {
            self.ppu_access.borrow_mut().write(reg, byte);
            *self.ppu_register_latch.borrow_mut() = byte;
        } else if DmaWriteAccessRegister::try_from(addr).is_ok() {
            let mut dma_data = [0; 256];
            for (i, e) in dma_data.iter_mut().enumerate() {
                let page_adress = (byte as u16) << 8;
                *e = self.get_byte(page_adress + i as u16);
            }
            self.ppu_access.borrow_mut().write_oam_dma(dma_data);
            *self.oam_dma_register_latch.borrow_mut() = byte;
        } else if let Ok(output_port) = OutputRegister::try_from(addr) {
            self.controller_access.borrow_mut().write(output_port, byte);
            *self.controller_register_latch.borrow_mut() = byte;
        } else if let Ok(reg) = ram_apu::WriteAccessRegister::try_from(addr) {
            self.apu_access.borrow_mut().write(reg, byte);
            *self.apu_register_latch.borrow_mut() = byte;
        } else if InputRegister::try_from(addr).is_ok() {
            *self.controller_register_latch.borrow_mut() = byte;
        } else if ReadAccessRegister::try_from(addr).is_ok() {
            *self.ppu_register_latch.borrow_mut() = byte;
        } else if ram_apu::ReadAccessRegister::try_from(addr).is_ok() {
            *self.apu_register_latch.borrow_mut() = byte;
        } else if CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)) {
            self.mapper.borrow_mut().store_prg_byte(addr, byte)
        } else if addr < CPU_TEST_MODE_SPACE_START {
            assert!(addr < INTERNAL_MIRROR_SIZE);
            self.memory[addr as usize] = byte;
        }
    }

    fn get_word(&self, addr: u16) -> u16 {
        crate::common::convert_2u8_to_u16(self.get_byte(addr), self.get_byte(addr + 1))
    }

    fn store_bytes(&mut self, addr: u16, bytes: &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            self.store_byte(addr + i as u16, *b);
        }
    }

    fn store_word(&mut self, addr: u16, bytes: u16) {
        self.store_byte(addr, (bytes & 0x00FF) as u8);
        self.store_byte(addr + 1, ((bytes & 0xFF00) >> 8) as u8);
    }
}

impl DmcMemory for Ram {
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
