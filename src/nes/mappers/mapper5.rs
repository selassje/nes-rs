use super::Mapper;
use super::mapper_internal::BankSize;
use super::mapper_internal::MapperInternal;
use crate::nes::common::Mirroring;
use crate::nes::mappers::PRG_RAM_RANGE;
use crate::nes::mappers::PRG_RANGE;
use BankSize::*;

use serde::{Deserialize, Serialize};

const PRG_MODE_SELECTION_REGISTER: u16 = 0x5100;
const CHR_MODE_SELECTION_REGISTER: u16 = 0x5101;
const PRG_RAM_PROTECT_REGISTER_1: u16 = 0x5102;
const PRG_RAM_PROTECT_REGISTER_2: u16 = 0x5103;
const PRG_BANK_REGISTER_1: u16 = 0x5113;
const PRG_BANK_REGISTER_5: u16 = 0x5117;
const IRQ_SCANLINE_COMPARE_REGISTER: u16 = 0x5203;
const IRQ_SCANLINE_STATUS_REGISTER: u16 = 0x5204;

#[derive(Serialize, Deserialize)]
pub struct Mapper5 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    prg_selection_mode: u8,
    chr_selection_mode: u8,
    prg_ram_protect_1: u8,
    prg_ram_protect_2: u8,
    bank_registers: [u8; 5],
    scanline_compare_value: u8,
    scanline_counter: u8,
    scanline_irq_enabled: bool,
    scanline_irq_pending: bool,
    in_frame: bool,
}

#[derive(PartialEq)]
enum PrgBankRegisterType {
    Rom,
    Ram,
    RomRam,
}

const PRG_BANK_REGISTER_TYPES: [PrgBankRegisterType; 5] = [
    PrgBankRegisterType::Ram,
    PrgBankRegisterType::RomRam,
    PrgBankRegisterType::RomRam,
    PrgBankRegisterType::RomRam,
    PrgBankRegisterType::Rom,
];

#[derive(Debug)]
struct PrgBankRegister {
    bank: usize,
    rom: bool,
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
            bank_registers: [0xFF; 5],
            scanline_compare_value: 0,
            scanline_counter: 0,
            scanline_irq_enabled: false,
            scanline_irq_pending: false,
            in_frame: false,
        }
    }

    fn get_prg_bank_register_index_and_size(&self, address: u16) -> (usize, BankSize) {
        let index_8_kb = (address - PRG_RAM_RANGE.start) / _8KB as u16;
        const INDEX_AND_MODE_TO_REGISTER_AND_SIZE: [[(usize, BankSize); 5]; 4] = [
            [(0, _8KB), (4, _32KB), (4, _32KB), (4, _32KB), (4, _32KB)],
            [(0, _8KB), (2, _16KB), (2, _16KB), (4, _16KB), (4, _16KB)],
            [(0, _8KB), (2, _16KB), (2, _16KB), (3, _8KB), (4, _8KB)],
            [(0, _8KB), (1, _8KB), (2, _8KB), (3, _8KB), (4, _8KB)],
        ];
        INDEX_AND_MODE_TO_REGISTER_AND_SIZE[self.prg_selection_mode as usize][index_8_kb as usize]
    }

    fn decode_prg_bank_register(
        &self,
        index: u8,
        bank_size: BankSize,
        address: u16,
    ) -> PrgBankRegister {
        let byte = self.bank_registers[index as usize];
        let mut bank_register = PrgBankRegister {
            bank: (byte & 0b0111_1111) as usize,
            rom: (byte & 0b1000_0000) != 0,
        };
        if index == 0 {
            bank_register.bank &= 0b0000_1111;
            bank_register.rom = false;
        }
        if bank_size == _16KB {
            bank_register.bank &= 0b0111_1110;
            bank_register.bank |= ((address >> 13) & 0b0000_0001) as usize;
        }
        if bank_size == _32KB {
            bank_register.bank &= 0b0111_1100;
            bank_register.bank |= ((address >> 13) & 0b0000_0011) as usize
        }
        bank_register
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
            IRQ_SCANLINE_COMPARE_REGISTER => {
                self.scanline_compare_value = byte;
            }
            IRQ_SCANLINE_STATUS_REGISTER => {
                self.scanline_irq_enabled = byte & 0b1000_0000 != 0;
            }

            address if PRG_RANGE.contains(&address) => {
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let bank_register = self.decode_prg_bank_register(index as u8, bank_size, address);
                let can_be_ram = PRG_BANK_REGISTER_TYPES[index] != PrgBankRegisterType::Rom;
                if can_be_ram && self.is_prg_ram_writable() && !bank_register.rom {
                    self.mapper_internal.store_prg_ram_byte(
                        address,
                        bank_register.bank as usize,
                        bank_size,
                        byte,
                    );
                } else {
                    println!(
                        "Mapper5: Ignored write to PRG address {:04X} with bank register {:?}, index={} Type={} RAMProtected={} ",
                        address,
                        bank_register,
                        index,
                        match PRG_BANK_REGISTER_TYPES[index] {
                            PrgBankRegisterType::Rom => "ROM",
                            PrgBankRegisterType::Ram => "RAM",
                            PrgBankRegisterType::RomRam => "ROM/RAM",
                        },
                        self.is_prg_ram_writable()
                    );
                }
            }
            _ => {
                println!("Unknown adress {:04X}", address)
            }
        }
    }
    fn store_chr_byte(&mut self, _address: u16, _byte: u8) {}

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        match address {
            IRQ_SCANLINE_STATUS_REGISTER => {
                let mut byte: u8 = 0;
                if self.scanline_irq_pending {
                    byte |= 0b1000_0000;
                }
                if self.in_frame {
                    byte |= 0b0100_0000
                }
                self.scanline_irq_pending = false;
                byte
            }
            address if PRG_RANGE.contains(&address) => {
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let bank_register = self.decode_prg_bank_register(index as u8, bank_size, address);
                if bank_register.rom {
                    self.mapper_internal.get_prg_rom_byte(
                        address,
                        bank_register.bank as usize,
                        bank_size,
                    )
                } else {
                    self.mapper_internal.get_prg_ram_byte(
                        address,
                        bank_register.bank as usize,
                        bank_size,
                    )
                }
            }
            _ => 0,
        }
    }
    fn get_mirroring(&self) -> crate::nes::common::Mirroring {
        self.mirroring
    }

    fn power_cycle(&mut self) {
        self.prg_selection_mode = 3;
        self.chr_selection_mode = 3;
        self.prg_ram_protect_1 = 0;
        self.prg_ram_protect_2 = 0;
        self.bank_registers = [0xFF; 5];
        self.scanline_compare_value = 0;
        self.scanline_counter = 0;
        self.scanline_irq_enabled = false;
        self.scanline_irq_pending = false;
        self.in_frame = false;
        self.mapper_internal.power_cycle();
    }

    fn notify_scanline(&mut self) {
        if !self.in_frame {
            self.in_frame = true;
            self.scanline_counter = 0;
        } else {
            self.scanline_counter += 1;
            if self.scanline_compare_value != 0
                && self.scanline_counter == self.scanline_compare_value
            {
                self.scanline_irq_pending = true;
            }
        }
    }

    fn is_irq_pending(&self) -> bool {
        self.scanline_irq_enabled && self.scanline_irq_pending
    }

    fn notify_vblank(&mut self) {
        self.in_frame = false;
        self.scanline_counter = 0;
        self.scanline_irq_pending = false;
    }
}
