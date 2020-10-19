use super::{
    mapper_internal::ChrRomBankSize, mapper_internal::MapperInternal,
    mapper_internal::PrgRomBankSize, Mapper,
};
use crate::common::Mirroring;

pub struct Mapper66 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    prg_bank: usize,
    chr_bank: usize,
}

impl Mapper66 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            mirroring,
            prg_bank: 0,
            chr_bank: 0,
        }
    }
}
impl Mapper for Mapper66 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal
            .get_chr_byte(address, self.chr_bank, ChrRomBankSize::_8KB as usize)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_pgr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal
            .get_pgr_byte(address, self.prg_bank, PrgRomBankSize::_16KB as usize)
    }

    fn reset(&mut self) {
        self.prg_bank = 0;
        self.chr_bank = 0;
        self.mapper_internal.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal
            .store_chr_byte(address, 0, ChrRomBankSize::_8KB as usize, byte)
    }

    fn store_pgr_byte(&mut self, _: u16, byte: u8) {
        self.chr_bank = (byte & 3) as usize;
        self.prg_bank = ((byte & 0b00110000) >> 4) as usize;
    }
}
