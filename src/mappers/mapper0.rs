use super::Mapper;
use crate::common::Mirroring;

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Mapper0 {
        let mut final_pgr_rom = prg_rom.clone();
        if final_pgr_rom.len() <= 16384 {
            final_pgr_rom.extend_from_slice(prg_rom.as_slice())
        }
        Mapper0 {
            prg_rom: final_pgr_rom,
            chr_rom: chr_rom,
            mirroring,
        }
    }
}

impl Mapper for Mapper0 {
    fn get_rom_start(&self) -> u16 {
        0x8000
    }

    fn get_pgr_rom(&self) -> &[u8] {
        self.prg_rom.as_slice()
    }

    fn get_chr_rom(&self) -> &[u8] {
        self.chr_rom.as_slice()
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_byte(&mut self, address: u16) -> u8 {
        self.prg_rom[address as usize - 0x8000]
    }
}
