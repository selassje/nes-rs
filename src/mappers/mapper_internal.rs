const CHR_DATA_SIZE: usize = 0x8000;
const PRG_DATA_SIZE: usize = 0x80000;
#[derive(PartialEq)]
pub enum PrgRomBankSize {
    _16KB = 0x4000,
    _32KB = 0x8000,
}

pub enum ChrRomBankSize {
    _4KB = 0x1000,
    _8KB = 0x2000,
}
pub struct MapperInternal {
    prg_org: Vec<u8>,
    prg: Box<[u8; PRG_DATA_SIZE]>,
    chr_org: Vec<u8>,
    chr: Box<[u8; CHR_DATA_SIZE]>,
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
        }
    }

    pub fn get_chr_byte(&mut self, address: u16, bank: usize, chr_bank_size: usize) -> u8 {
        self.chr[chr_bank_size * bank + (address as usize % chr_bank_size)]
    }

    pub fn get_pgr_byte(&mut self, address: u16, bank: usize, prg_bank_size: usize) -> u8 {
        self.prg[prg_bank_size * bank + (address as usize % prg_bank_size)]
    }

    pub fn get_pgr_bank_count(&mut self, prg_bank_size: usize) -> usize {
        self.prg_org.len() / prg_bank_size
    }

    pub fn reset(&mut self) {
        self.chr[..self.chr_org.len()].copy_from_slice(self.chr_org.as_slice());
        self.prg[..self.prg_org.len()].copy_from_slice(self.prg_org.as_slice());
    }

    pub fn store_chr_byte(&mut self, address: u16, bank: usize, chr_bank_size: usize, byte: u8) {
        self.chr[chr_bank_size * bank + (address as usize % chr_bank_size)] = byte;
    }
}
