const CHR_DATA_SIZE: usize = 0x8000;
const PRG_ROM_DATA_SIZE: usize = 0x80000;
const PRG_RAM_DATA_SIZE: usize = 0x20000;

#[derive(Clone, Copy, PartialEq)]
pub enum BankSize {
    _1KB = 0x0400,
    _2KB = 0x0800,
    _4KB = 0x1000,
    _8KB = 0x2000,
    _16KB = 0x4000,
    _32KB = 0x8000,
}

pub struct MapperInternal {
    prg_ram: Box<[u8; PRG_RAM_DATA_SIZE]>,
    prg_rom: Box<[u8; PRG_ROM_DATA_SIZE]>,
    prg_rom_size: usize,
    chr_rom: Vec<u8>,
    chr: Box<[u8; CHR_DATA_SIZE]>,
}

impl MapperInternal {
    pub fn new(_prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mut chr = Box::new([0; CHR_DATA_SIZE]);
        let mut prg_rom = Box::new([0; PRG_ROM_DATA_SIZE]);
        chr[..chr_rom.len()].copy_from_slice(chr_rom.as_slice());
        prg_rom[.._prg_rom.len()].copy_from_slice(_prg_rom.as_slice());
        let prg_rom_size = _prg_rom.len();
        Self {
            prg_ram: Box::new([0; PRG_RAM_DATA_SIZE]),
            prg_rom,
            prg_rom_size,
            chr_rom,
            chr,
        }
    }

    fn get_address_index(&self, address: u16, bank: usize, bank_size: BankSize) -> usize {
        bank_size as usize * bank + (address as usize % bank_size as usize)
    }

    pub fn get_prg_rom_byte(&mut self, address: u16, bank: usize, prg_bank_size: BankSize) -> u8 {
        self.prg_rom[self.get_address_index(address, bank, prg_bank_size)]
    }

    #[allow(dead_code)]
    pub fn get_prg_ram_byte(&mut self, address: u16, bank: usize, bank_size: BankSize) -> u8 {
        self.prg_ram[self.get_address_index(address, bank, bank_size)]
    }

    #[allow(dead_code)]
    pub fn store_prg_ram_byte(&mut self, address: u16, bank: usize, bank_size: BankSize, byte: u8) {
        self.prg_ram[self.get_address_index(address, bank, bank_size)] = byte
    }

    pub fn get_chr_byte(&mut self, address: u16, bank: usize, chr_bank_size: BankSize) -> u8 {
        self.chr[self.get_address_index(address, bank, chr_bank_size)]
    }

    pub fn store_chr_byte(&mut self, address: u16, bank: usize, chr_bank_size: BankSize, byte: u8) {
        self.chr[self.get_address_index(address, bank, chr_bank_size)] = byte;
    }

    pub fn get_prg_rom_bank_count(&mut self, prg_bank_size: BankSize) -> usize {
        self.prg_rom_size / prg_bank_size as usize
    }

    pub fn reset(&mut self) {
        self.chr.iter_mut().for_each(|m| *m = 0);
        self.prg_ram.iter_mut().for_each(|m| *m = 0);
        self.chr[..self.chr_rom.len()].copy_from_slice(self.chr_rom.as_slice());
    }
}
