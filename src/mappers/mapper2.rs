use super::{
    mapper_internal::ChrRomBankSize, mapper_internal::MapperInternal,
    mapper_internal::PrgRomBankSize, Mapper,
};
use crate::common::Mirroring;
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
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal
            .get_chr_byte(address, 0, ChrRomBankSize::_8KB as usize)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_prg_byte(&mut self, address: u16) -> u8 {
        let bank = if address < 0xC000 {
            self.switchable_bank_0
        } else {
            self.mapper_internal
                .get_pgr_bank_count(PrgRomBankSize::_16KB as usize)
                - 1
        };
        self.mapper_internal
            .get_pgr_byte(address, bank, PrgRomBankSize::_16KB as usize)
    }

    fn reset(&mut self) {
        self.switchable_bank_0 = 0;
        self.mapper_internal.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal
            .store_chr_byte(address, 0, ChrRomBankSize::_8KB as usize, byte)
    }

    fn store_prg_byte(&mut self, _: u16, byte: u8) {
        self.switchable_bank_0 = byte as usize;
    }
}
