use crate::common::{CHR_ROM_UNIT_SIZE, PRG_ROM_UNIT_SIZE};

const CHR_DATA_SIZE: usize = 0x8000;
const PRG_DATA_SIZE: usize = 0x20000;

pub enum PrgRomBankSize {
    _16KB = 1,
    _32KB = 2,
}

pub enum ChrRomBankSize {
    _8KB = 1,
}
pub struct MapperInternal {
    prg_org: Vec<u8>,
    prg: Box<[u8; PRG_DATA_SIZE]>,
    chr_org: Vec<u8>,
    chr: Box<[u8; CHR_DATA_SIZE]>,
    prg_bank_size: usize,
    chr_bank_size: usize,
}
impl MapperInternal {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mut chr = Box::new([0; CHR_DATA_SIZE]);
        let mut prg = Box::new([0; PRG_DATA_SIZE]);

        chr[..chr_rom.len()].copy_from_slice(chr_rom.as_slice());
        prg[..prg_rom.len()].copy_from_slice(prg_rom.as_slice());
        Self {
            prg_org: prg_rom,
            chr_org: chr_rom,
            chr,
            prg,
            prg_bank_size: 0,
            chr_bank_size: 0,
        }
    }

    pub fn get_chr_byte(&mut self, address: u16, bank: usize) -> u8 {
        self.chr[CHR_ROM_UNIT_SIZE * bank + (address as usize % self.chr_bank_size)]
    }

    pub fn get_pgr_byte(&mut self, address: u16, bank: usize) -> u8 {
        self.prg[self.prg_bank_size * bank + (address as usize % self.prg_bank_size)]
    }

    pub fn get_pgr_bank_count(&mut self) -> usize {
        self.prg_org.len() / self.prg_bank_size
    }

    pub fn reset(&mut self) {
        self.chr[..self.chr_org.len()].copy_from_slice(self.chr_org.as_slice());
        self.prg[..self.prg_org.len()].copy_from_slice(self.prg_org.as_slice());
    }

    pub fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.chr[address as usize] = byte;
    }

    pub fn set_prg_bank_size(&mut self, size: PrgRomBankSize) {
        self.prg_bank_size = size as usize * PRG_ROM_UNIT_SIZE;
    }
    pub fn set_chr_bank_size(&mut self, size: ChrRomBankSize) {
        self.chr_bank_size = size as usize * CHR_ROM_UNIT_SIZE;
    }
}
