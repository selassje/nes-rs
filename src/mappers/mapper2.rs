use super::{Mapper, CHR_DATA_SIZE};
use crate::common::{Mirroring, PRG_ROM_UNIT_SIZE};

pub struct Mapper2 {
    prg_rom_org: Vec<u8>,
    prg_rom: Vec<u8>,
    chr_rom_org: Vec<u8>,
    mirroring: Mirroring,
    chr: [u8; CHR_DATA_SIZE],
    switchable_bank_0: usize,
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        assert!(prg_rom.len() % PRG_ROM_UNIT_SIZE == 0);
        let mut chr = [0; CHR_DATA_SIZE];
        chr[..chr_rom.len()].copy_from_slice(chr_rom.as_slice());
        Self {
            prg_rom: prg_rom.clone(),
            prg_rom_org: prg_rom,
            chr_rom_org: chr_rom,
            mirroring,
            chr,
            switchable_bank_0: 0,
        }
    }
}

impl Mapper for Mapper2 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.chr[address as usize]
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn get_pgr_byte(&mut self, address: u16) -> u8 {
        let address = if address < 0xC000 {
            self.switchable_bank_0 * PRG_ROM_UNIT_SIZE + (address as usize - 0x8000)
        } else {
            (self.prg_rom.len() / PRG_ROM_UNIT_SIZE - 1) * PRG_ROM_UNIT_SIZE
                + (address as usize - 0xC000)
        };
        self.prg_rom[address as usize]
    }

    fn reset(&mut self) {
        self.chr = [0; CHR_DATA_SIZE];
        self.chr[..self.chr_rom_org.len()].copy_from_slice(self.chr_rom_org.as_slice());
        self.prg_rom = self.prg_rom_org.clone();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.chr[address as usize] = byte;
    }

    fn store_pgr_byte(&mut self, _: u16, byte: u8) {
        self.switchable_bank_0 = byte as usize;
    }
}
