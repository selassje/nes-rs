use super::mapper_internal::BankSize::*;
use super::mapper_internal::MapperInternal;
use super::Mapper;
use crate::nes::common::Mirroring;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mapper71 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    switchable_prg_rom_bank: usize,
    last_prg_rom_bank: usize,
}

impl Mapper71 {
    pub fn new(prg_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, vec![]);
        let last_prg_rom_bank = mapper_internal.get_prg_rom_bank_count(_16KB) - 1;
        Self {
            mapper_internal,
            mirroring,
            switchable_prg_rom_bank: 0,
            last_prg_rom_bank,
        }
    }
}

impl Mapper for Mapper71 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        self.mapper_internal.get_chr_byte(address, 0, _8KB)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_prg_byte(&self, address: u16) -> u8 {
        assert!(address >= 0x8000);
        let bank = if address < 0xC000 {
            self.switchable_prg_rom_bank
        } else {
            self.last_prg_rom_bank
        };
        self.mapper_internal.get_prg_rom_byte(address, bank, _16KB)
    }

    fn power_cycle(&mut self) {
        self.switchable_prg_rom_bank = 0;
        self.mapper_internal.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal.store_chr_byte(address, 0, _8KB, byte)
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x9000..=0x9FFF => {
                self.mirroring = if byte & 0x10 != 0 {
                    Mirroring::SingleScreenUpperBank
                } else {
                    Mirroring::SingleScreenLowerBank
                }
            }
            0xC000..=0xFFFF => {
                self.switchable_prg_rom_bank = (byte & self.last_prg_rom_bank as u8) as usize;
            }
            _ => (),
        }
    }
}
