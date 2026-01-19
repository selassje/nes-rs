use super::Mapper;
use crate::nes::common::Mirroring;
use crate::nes::mappers::mapper_internal::BankSize::*;
use crate::nes::mappers::mapper_internal::MapperInternal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mapper2 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    switchable_bank_0: usize,
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            mirroring,
            switchable_bank_0: 0,
        }
    }
}

impl Mapper for Mapper2 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        self.mapper_internal.get_chr_byte(address, 0, _8KB)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_prg_byte(&mut self, address: u16) -> u8 {
        let bank = if address < 0xC000 {
            self.switchable_bank_0
        } else {
            self.mapper_internal.get_prg_rom_bank_count(_16KB) - 1
        };
        self.mapper_internal.get_prg_rom_byte(address, bank, _16KB)
    }

    fn power_cycle(&mut self) {
        self.switchable_bank_0 = 0;
        self.mapper_internal.power_cycle();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal.store_chr_byte(address, 0, _8KB, byte)
    }

    fn store_prg_byte(&mut self, _: u16, byte: u8) {
        self.switchable_bank_0 = byte as usize;
    }
}
