use super::{Mapper, mapper_internal::BankSize::*};
use crate::nes::common::Mirroring;
use crate::nes::mappers::mapper_internal::MapperInternal;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Mapper0 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mut final_prg_rom = prg_rom.clone();
        if final_prg_rom.len() <= _16KB as usize {
            final_prg_rom.extend_from_slice(prg_rom.as_slice())
        }
        let mapper_internal = MapperInternal::new(final_prg_rom, chr_rom);

        Self {
            mapper_internal,
            mirroring,
        }
    }
}

impl Mapper for Mapper0 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        self.mapper_internal.get_chr_byte(address, 0, _8KB)
    }

    fn get_prg_byte(&self, address: u16) -> u8 {
        if address < 0xC000 {
            self.mapper_internal.get_prg_rom_byte(address, 0, _16KB)
        } else {
            self.mapper_internal.get_prg_rom_byte(address, 1, _16KB)
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal.store_chr_byte(address, 0, _8KB, byte)
    }

    fn store_prg_byte(&mut self, _: u16, _: u8) {}

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn power_cycle(&mut self) {
        self.mapper_internal.reset();
    }
}
