use crate::common::CHR_ROM_UNIT_SIZE;
use crate::common::PRG_ROM_UNIT_SIZE;

const CHR_DATA_SIZE: usize = 0x2000;
const PRG_DATA_SIZE: usize = 0x20000;
pub struct MapperInternal {
    prg_org: Vec<u8>,
    prg: [u8; PRG_DATA_SIZE],
    chr_org: Vec<u8>,
    chr: [u8; CHR_DATA_SIZE],
}

impl MapperInternal {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        assert!(prg_rom.len() % PRG_ROM_UNIT_SIZE == 0);
        assert!(chr_rom.len() % CHR_ROM_UNIT_SIZE == 0);
        let mut chr = [0; CHR_DATA_SIZE];
        let mut prg = [0; PRG_DATA_SIZE];

        chr[..chr_rom.len()].copy_from_slice(chr_rom.as_slice());
        prg[..prg_rom.len()].copy_from_slice(prg_rom.as_slice());
        Self {
            prg_org: prg_rom,
            chr_org: chr_rom,
            chr,
            prg,
        }
    }

    pub fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.chr[address as usize]
    }

    pub fn get_pgr_byte(&mut self, address: u16, bank: usize) -> u8 {
        self.prg[PRG_ROM_UNIT_SIZE * bank + (address as usize % PRG_ROM_UNIT_SIZE)]
    }

    pub fn get_pgr_bank_count(&mut self) -> usize {
        self.prg_org.len() / PRG_ROM_UNIT_SIZE
    }

    pub fn reset(&mut self) {
        self.chr = [0; CHR_DATA_SIZE];
        self.prg = [0; PRG_DATA_SIZE];
        self.chr[..self.chr_org.len()].copy_from_slice(self.chr_org.as_slice());
        self.prg[..self.prg_org.len()].copy_from_slice(self.prg_org.as_slice());
    }

    pub fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.chr[address as usize] = byte;
    }

    #[allow(dead_code)]
    pub fn store_pgr_byte(&mut self, address: u16, bank: usize, byte: u8) {
        self.prg[PRG_ROM_UNIT_SIZE * bank + address as usize % PRG_ROM_UNIT_SIZE] = byte;
    }
}
