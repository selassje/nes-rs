use super::Mapper;
use super::mapper_internal::MapperInternal;
use crate::nes::common::Mirroring;

use serde::{Deserialize, Serialize};

const PRG_MODE_SELECTION_REGISTER: u16 = 0x5100;
const CHR_MODE_SELECTION_REGISTER: u16 = 0x5101;
const PRG_RAM_PROTECT_REGISTER_1: u16 = 0x5102;
const PRG_RAM_PROTECT_REGISTER_2: u16 = 0x5103;
const PRG_BANK_REGISTER_1: u16 = 0x5113;
const PRG_BANK_REGISTER_2: u16 = 0x5114;
const PRG_BANK_REGISTER_3: u16 = 0x5115;
const PRG_BANK_REGISTER_4: u16 = 0x5116;
const PRG_BANK_REGISTER_5: u16 = 0x5117;

#[derive(Serialize, Deserialize)]
pub struct Mapper5 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    prg_selection_mode: u8,
    chr_selection_mode: u8,
    prg_ram_protect_1: u8,
    prg_ram_protect_2: u8,
    bank_registers: [u8; 5],
}

enum PrgBankType {
    Rom,
    Ram,
    RomRam,
}

struct BankRegister {
    bank: u8,
    ce: bool,
    rom: bool,
}

fn decode_bank_register(byte: u8, address: u16, mode: u8) -> BankRegister {
    assert!(address >= PRG_BANK_REGISTER_1 && address <= PRG_BANK_REGISTER_5);
    let mut bank_register = BankRegister {
        bank: byte & 0b0011_1111,
        ce: (byte & 0b0000_0100) != 0,
        rom: (byte & 0b1000_0000) != 0,
    };
    if address == PRG_BANK_REGISTER_1 {
        bank_register.bank &= 0b0000_1111;
        bank_register.rom =  false;
    }




    bank_register
}

impl Mapper5 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            mirroring,
            prg_selection_mode: 3,
            chr_selection_mode: 3,
            prg_ram_protect_1: 0,
            prg_ram_protect_2: 0,
            bank_registers: [0; 5],
        }
    }

    fn is_prg_ram_writable(&self) -> bool {
        (self.prg_ram_protect_1 & 0b11) == 0b10 && (self.prg_ram_protect_2 & 0b11) == 0b01
    }
}

impl Mapper for Mapper5 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        self.mapper_internal
            .get_chr_byte(address, 0, super::mapper_internal::BankSize::_8KB)
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        match address {
            PRG_MODE_SELECTION_REGISTER => {
                self.prg_selection_mode = byte & 0b11;
            }
            CHR_MODE_SELECTION_REGISTER => {
                self.chr_selection_mode = byte & 0b11;
            }
            PRG_RAM_PROTECT_REGISTER_1 => {
                self.prg_ram_protect_1 = byte;
            }
            PRG_RAM_PROTECT_REGISTER_2 => {
                self.prg_ram_protect_2 = byte;
            }
            PRG_BANK_REGISTER_1..=PRG_BANK_REGISTER_5 => {
                let index = (address - PRG_BANK_REGISTER_1) as usize;
                self.bank_registers[index] = byte;
            }
            _ => {}
        }
    }
    fn store_chr_byte(&mut self, _address: u16, _byte: u8) {}

    fn get_prg_byte(&self, address: u16) -> u8 {
        let bank = match self.prg_selection_mode {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => {
                self.mapper_internal
                    .get_prg_rom_bank_count(super::mapper_internal::BankSize::_8KB)
                    - 1
            }
            _ => panic!("Unsupported PRG selection mode {}", self.prg_selection_mode),
        };
        self.mapper_internal
            .get_prg_rom_byte(address, bank, super::mapper_internal::BankSize::_8KB)
    }
    fn get_mirroring(&self) -> crate::nes::common::Mirroring {
        self.mirroring
    }

    fn power_cycle(&mut self) {
        self.prg_selection_mode = 3;
        self.mapper_internal.power_cycle();
    }
}
