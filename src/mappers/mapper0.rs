use super::{mapper_internal::ChrRomBankSize, mapper_internal::PrgRomBankSize, Mapper};
use crate::common::Mirroring;
use crate::mappers::mapper_internal::MapperInternal;

pub struct Mapper0 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mut final_prg_rom = prg_rom.clone();
        if final_prg_rom.len() <= 16384 {
            final_prg_rom.extend_from_slice(prg_rom.as_slice())
        }
        let mapper_internal = MapperInternal::new(final_prg_rom, chr_rom);

        Self {
            mirroring,
            mapper_internal,
        }
    }
}

impl Mapper for Mapper0 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal
            .get_chr_byte(address, 0, ChrRomBankSize::_8KB as usize)
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        if address < 0xC000 {
            self.mapper_internal
                .get_pgr_byte(address, 0, PrgRomBankSize::_16KB as usize)
        } else {
            self.mapper_internal
                .get_pgr_byte(address, 1, PrgRomBankSize::_16KB as usize)
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal
            .store_chr_byte(address, 0, ChrRomBankSize::_8KB as usize, byte)
    }

    fn store_prg_byte(&mut self, address: u16, _: u8) {
        //todo!("address {:X}", address);
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self) {
        self.mapper_internal.reset();
    }
}
