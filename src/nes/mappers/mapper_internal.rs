use serde::{Deserialize, Serialize};

const PRG_RAM_DATA_SIZE: usize = 0x20000;
const PRG_ROM_DATA_SIZE: usize = 0x80000;
const CHR_ROM_DATA_SIZE: usize = 0x40000;
const CHR_RAM_DATA_SIZE: usize = 0x2000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(super) enum BankSize {
    _1KB = 0x0400,
    _2KB = 0x0800,
    _4KB = 0x1000,
    _8KB = 0x2000,
    _16KB = 0x4000,
    _32KB = 0x8000,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct BankSelect {
    pub size: BankSize,
    pub bank: usize,
}

impl Default for BankSelect {
    fn default() -> Self {
        Self {
            size: BankSize::_1KB,
            bank: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct MapperInternal {
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
    prg_rom_size: usize,
    chr_rom: Vec<u8>,
    chr_rom_size: usize,
    chr_ram: Vec<u8>,
}

impl MapperInternal {
    pub fn new(_prg_rom: Vec<u8>, _chr_rom: Vec<u8>) -> Self {
        assert!(_prg_rom.len() <= PRG_ROM_DATA_SIZE);
        assert!(_chr_rom.len() <= CHR_ROM_DATA_SIZE);
        let mut prg_rom = vec![0u8; PRG_ROM_DATA_SIZE];
        let mut chr_rom = vec![0u8; CHR_ROM_DATA_SIZE];

        prg_rom[.._prg_rom.len()].copy_from_slice(&_prg_rom);
        chr_rom[.._chr_rom.len()].copy_from_slice(&_chr_rom);

        Self {
            prg_ram: vec![0u8; PRG_RAM_DATA_SIZE],
            prg_rom,
            prg_rom_size: _prg_rom.len(),
            chr_rom,
            chr_rom_size: _chr_rom.len(),
            chr_ram: vec![0u8; CHR_RAM_DATA_SIZE],
        }
    }

    fn get_address_index(address: u16, bank: usize, bank_size: BankSize) -> usize {
        bank_size as usize * bank + (address as usize % bank_size as usize)
    }

    pub fn get_prg_rom_byte(&self, address: u16, bank: usize, prg_bank_size: BankSize) -> u8 {
        let bank_count = self.prg_rom_size / prg_bank_size as usize;
        let masked_bank = if bank_count > 0 { bank % bank_count } else { 0 };
        let index = Self::get_address_index(address, masked_bank, prg_bank_size);
        self.prg_rom[index]
    }

    pub fn get_prg_ram_byte(&self, address: u16, bank: usize, bank_size: BankSize) -> u8 {
        self.prg_ram[Self::get_address_index(address, bank, bank_size)]
    }

    pub fn store_prg_ram_byte(&mut self, address: u16, bank: usize, bank_size: BankSize, byte: u8) {
        self.prg_ram[Self::get_address_index(address, bank, bank_size)] = byte
    }

    pub fn get_chr_byte(&self, address: u16, bank: usize, chr_bank_size: BankSize) -> u8 {
        if self.chr_rom_size == 0 {
            self.chr_ram[address as usize]
        } else {
            let index = Self::get_address_index(address, bank, chr_bank_size);
            self.chr_rom[index]
        }
    }

    pub fn store_chr_byte(&mut self, address: u16, _: usize, _: BankSize, byte: u8) {
        self.chr_ram[address as usize] = byte;
    }

    pub fn get_prg_rom_bank_count(&self, prg_bank_size: BankSize) -> usize {
        self.prg_rom_size / prg_bank_size as usize
    }

    pub fn power_cycle(&mut self) {
        self.chr_ram.iter_mut().for_each(|m| *m = 0);
    }
}
