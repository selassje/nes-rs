use super::Mapper;
use super::mapper_internal::MapperInternal;
use crate::nes::common::Mirroring;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mapper5 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    prg_selection_mode: u8,
}

impl Mapper5 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            mirroring,
            prg_selection_mode: 3,
        }
    }
}

impl Mapper for Mapper5 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        self.mapper_internal
            .get_chr_byte(address, 0, super::mapper_internal::BankSize::_8KB)
    }

    fn store_prg_byte(&mut self, _: u16, byte: u8) {
        self.prg_selection_mode = byte & 0b11;
    }

    fn store_chr_byte(&mut self, _address: u16, _byte: u8) {}

    fn get_prg_byte(&self, address: u16) -> u8 {
        let bank = match self.prg_selection_mode {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => {
                self.mapper_internal
                    .get_prg_rom_bank_count(super::mapper_internal::BankSize::_8KB)
                    - 1
            }
            _ => panic!("Unsupported PRG selection mode {}", self.prg_selection_mode),
        };
        self.mapper_internal
            .get_prg_rom_byte(address, bank, super::mapper_internal::BankSize::_8KB)
    }
    fn get_mirroring(&self) -> crate::nes::common::Mirroring {
        self.mirroring
    }

    fn power_cycle(&mut self) {
        self.prg_selection_mode = 3;
        self.mapper_internal.power_cycle();
    }
}
