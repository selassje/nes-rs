use super::Mapper;
use crate::common::Mirroring;
use crate::mappers::mapper_internal::BankSize::*;
use crate::mappers::mapper_internal::MapperInternal;
pub struct Mapper7 {
    mapper_internal: MapperInternal,
    register: usize,
}

impl Mapper7 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            register: 0,
        }
    }
}

impl Mapper for Mapper7 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal.get_chr_byte(address, 0, _8KB)
    }

    fn get_mirroring(&self) -> Mirroring {
        if self.register & 0x10 != 0 {
            Mirroring::SingleScreenUpperBank
        } else {
            Mirroring::SingleScreenLowerBank
        }
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        if address >= 0x8000 {
            let bank = (self.register & 7) as usize;
            self.mapper_internal.get_prg_rom_byte(address, bank, _32KB)
        } else {
            0            
        }
    }

    fn reset(&mut self) {
        self.register = 0;
        self.mapper_internal.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal.store_chr_byte(address, 0, _8KB, byte)
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        if address >= 0x8000 {
            self.register = byte as usize;
        }
    }
}
