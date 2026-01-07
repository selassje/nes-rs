use super::mappers::MapperEnum;
use super::ram_apu;
use super::ram_apu::ReadAccessRegisters;
use super::ram_apu::WriteAcessRegisters;
use super::ram_controllers::*;
use super::ram_ppu::*;
use super::RamBus;
use super::{mappers::Mapper, memory::*};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::ops::Range;

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
#[derive(Serialize, Deserialize, Default)]
pub struct Ram {
    memory: MemoryImpl<0x0808>,
    dmc_sample_address: usize,
    ppu_register_latch: RegisterLatch,
    apu_register_latch: RegisterLatch,
    controller_register_latch: RegisterLatch,
    oam_dma_register_latch: RegisterLatch,
}

impl Ram {
    pub fn new() -> Self {
        Self {
            memory: MemoryImpl::new(),
            dmc_sample_address: 0,
            ppu_register_latch: RegisterLatch::new(0),
            apu_register_latch: RegisterLatch::new(0),
            controller_register_latch: RegisterLatch::new(0),
            oam_dma_register_latch: RegisterLatch::new(0),
        }
    }

    pub fn power_cycle(&mut self) {
        self.memory.clear();
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
    fn get_byte(&self, address_org: u16, bus: &mut RamBus) -> u8 {
        let addr = self.get_real_address(address_org);
        if let Ok(reg) = ReadAccessRegister::try_from(addr) {
            let mut ppu_register_value = bus.ppu.read(reg, bus.mapper);
            if reg == ReadAccessRegister::PpuStatus {
                const LOW_5_BITS: u8 = 0b00011111;
                ppu_register_value &= !LOW_5_BITS;
                ppu_register_value |= *self.ppu_register_latch.borrow() & LOW_5_BITS
            }
            *self.ppu_register_latch.borrow_mut() = ppu_register_value;
            *self.ppu_register_latch.borrow()
        } else if let Ok(reg) = ram_apu::ReadAccessRegister::try_from(addr) {
            *self.apu_register_latch.borrow_mut() = bus.apu.read(reg);
            *self.apu_register_latch.borrow()
        } else if let Ok(input_port) = InputRegister::try_from(addr) {
            *self.controller_register_latch.borrow_mut() = bus.controllers.read(input_port);
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
            bus.mapper.get_prg_byte(addr)
        } else if addr >= CPU_TEST_MODE_SPACE_START {
            self.memory
                .get_byte(INTERNAL_MIRROR_SIZE + addr - CPU_TEST_MODE_SPACE_START)
        } else if addr < INTERNAL_MIRROR_SIZE {
            self.memory.get_byte(addr)
        } else {
            panic!("Address org {:X} real {:X}", address_org, addr);
        }
    }

    fn store_byte(&mut self, address: u16, byte: u8, bus: &mut RamBus) {
        let addr = self.get_real_address(address);
        if let Ok(reg) = WriteAccessRegister::try_from(addr) {
            bus.ppu.write(reg, byte, bus.mapper);
            *self.ppu_register_latch.borrow_mut() = byte;
        } else if DmaWriteAccessRegister::try_from(addr).is_ok() {
            let mut dma_data = [0; 256];
            for (i, e) in dma_data.iter_mut().enumerate() {
                let page_adress = (byte as u16) << 8;
                *e = self.get_byte(page_adress + i as u16, bus);
            }
            bus.ppu.write_oam_dma(dma_data);
            *self.oam_dma_register_latch.borrow_mut() = byte;
        } else if let Ok(output_port) = OutputRegister::try_from(addr) {
            bus.controllers.write(output_port, byte);
            *self.controller_register_latch.borrow_mut() = byte;
        } else if let Ok(reg) = ram_apu::WriteAccessRegister::try_from(addr) {
            bus.apu.write(reg, byte);
            *self.apu_register_latch.borrow_mut() = byte;
        } else if InputRegister::try_from(addr).is_ok() {
            *self.controller_register_latch.borrow_mut() = byte;
        } else if ReadAccessRegister::try_from(addr).is_ok() {
            *self.ppu_register_latch.borrow_mut() = byte;
        } else if ram_apu::ReadAccessRegister::try_from(addr).is_ok() {
            *self.apu_register_latch.borrow_mut() = byte;
        } else if CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)) {
            bus.mapper.store_prg_byte(addr, byte)
        } else if addr < CPU_TEST_MODE_SPACE_START {
            assert!(addr < INTERNAL_MIRROR_SIZE);
            self.memory.store_byte(addr, byte);
        }
    }
}

impl DmcMemory for Ram {
    fn set_sample_address(&mut self, address: u8) {
        self.dmc_sample_address = 0xC000 + (address as usize * 64);
    }

    fn get_next_sample_byte(&mut self, mapper: &mut MapperEnum) -> u8 {
        let addr = self.get_real_address(self.dmc_sample_address as u16);
        assert!(CARTRIDGE_SPACE_RANGE.contains(&(addr as u32)));
        let byte = mapper.get_prg_byte(addr);
        self.dmc_sample_address = if self.dmc_sample_address == 0xFFFF {
            0x8000
        } else {
            self.dmc_sample_address + 1
        };
        byte
    }
}
