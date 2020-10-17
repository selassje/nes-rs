use super::{Mapper, CHR_DATA_SIZE};
use crate::common::Mirroring;

pub struct Mapper0 {
    prg_rom_org: Vec<u8>,
    prg_rom: Vec<u8>,
    chr_rom_org: Vec<u8>,
    mirroring: Mirroring,
    chr: [u8; CHR_DATA_SIZE],
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Mapper0 {
        let mut final_pgr_rom = prg_rom.clone();
        if final_pgr_rom.len() <= 16384 {
            final_pgr_rom.extend_from_slice(prg_rom.as_slice())
        }

        let mut chr = [0; CHR_DATA_SIZE];
        chr[..chr_rom.len()].copy_from_slice(chr_rom.as_slice());
        Mapper0 {
            prg_rom: final_pgr_rom.clone(),
            prg_rom_org: final_pgr_rom,
            chr_rom_org: chr_rom,
            mirroring,
            chr,
        }
    }
}

impl Mapper for Mapper0 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.chr[address as usize]
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_pgr_byte(&mut self, address: u16) -> u8 {
        self.prg_rom[address as usize - 0x8000]
    }

    fn reset(&mut self) {
        self.chr = [0; CHR_DATA_SIZE];
        self.chr[..self.chr_rom_org.len()].copy_from_slice(self.chr_rom_org.as_slice());
        self.prg_rom = self.prg_rom_org.clone();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.chr[address as usize] = byte;
    }

    fn store_pgr_byte(&mut self, _: u16, _: u8) {
        unimplemented!();
    }
}
