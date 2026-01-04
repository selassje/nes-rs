use super::Mapper;
use crate::common::Mirroring;
use crate::mappers::mapper_internal::BankSize;
use crate::mappers::mapper_internal::BankSize::*;
use crate::mappers::mapper_internal::MapperInternal;
trait ControlRegister {
    fn get_prg_bank_mode(&self) -> u8;
    fn get_chr_bank_mode(&self) -> u8;
    fn get_mirroring(&self) -> Mirroring;
}

impl ControlRegister for u8 {
    fn get_mirroring(&self) -> Mirroring {
        let mirroring = self & 3;
        match mirroring {
            0 => Mirroring::SingleScreenLowerBank,
            1 => Mirroring::SingleScreenUpperBank,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => panic!("Unsupported mirroring {}", mirroring),
        }
    }

    fn get_prg_bank_mode(&self) -> u8 {
        (self & 0b01100) >> 2
    }

    fn get_chr_bank_mode(&self) -> u8 {
        (self & 0b10000) >> 4
    }
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ShiftRegister {
    value: u8,
    write_count: u8,
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Mapper1 {
    mapper_internal: MapperInternal,
    shift_register: ShiftRegister,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
    last_16k_bank: usize,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let last_16k_bank = prg_rom.len() / BankSize::_16KB as usize - 1;
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            shift_register: Default::default(),
            control: 0x0C,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            last_16k_bank,
        }
    }

    fn get_chr_bank_info_from_address(&self, address: u16) -> (usize, BankSize) {
        if self.control.get_chr_bank_mode() == 0 {
            (self.chr_bank_0 as usize >> 1, _8KB)
        } else {
            match address {
                0x0000..=0x0FFF => (self.chr_bank_0 as usize, _4KB),
                0x1000..=0x1FFF => (self.chr_bank_1 as usize, _4KB),
                _ => panic!("Incorrect CHR address {:x}", address),
            }
        }
    }
}

impl Mapper for Mapper1 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        let (bank, bank_size) = self.get_chr_bank_info_from_address(address);
        self.mapper_internal.get_chr_byte(address, bank, bank_size)
    }

    fn get_prg_byte(&self, address: u16) -> u8 {
        let bank_mode = self.control.get_prg_bank_mode();
        if bank_mode < 2 {
            self.mapper_internal.get_prg_rom_byte(
                address,
                (self.prg_bank as usize & 0xF) >> 1,
                _32KB,
            )
        } else {
            let [bank_1, bank_2] = match bank_mode {
                2 => [0, self.prg_bank & 0xF],
                3 => [self.prg_bank & 0xF, self.last_16k_bank as _],
                _ => panic!("Mapper1: Unsupported PRG Mode"),
            };
            let bank = match address {
                0x8000..=0xBFFF => bank_1,
                0xC000..=0xFFFF => bank_2,
                _ => 0,
            };
            self.mapper_internal
                .get_prg_rom_byte(address, bank.into(), _16KB)
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        let (bank, bank_size) = self.get_chr_bank_info_from_address(address);
        self.mapper_internal
            .store_chr_byte(address, bank, bank_size, byte)
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        if byte & 0b1000_0000 != 0 {
            self.shift_register = Default::default();
            self.control |= 0x0C;
        } else {
            self.shift_register.value >>= 1;
            self.shift_register.value |= (byte & 1) << 4;
            self.shift_register.write_count += 1;

            if self.shift_register.write_count == 5 {
                let register = match address {
                    0x8000..=0x9FFF => &mut self.control,
                    0xA000..=0xBFFF => &mut self.chr_bank_0,
                    0xC000..=0xDFFF => &mut self.chr_bank_1,
                    0xE000..=0xFFFF => &mut self.prg_bank,
                    _ => panic!("Incorrect address {:X}", address),
                };
                *register = self.shift_register.value;
                self.shift_register = Default::default();
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.control.get_mirroring()
    }

    fn power_cycle(&mut self) {
        self.mapper_internal.reset();
        self.control = 0x0C;
        self.chr_bank_0 = 0;
        self.chr_bank_1 = 0;
        self.shift_register.value = 0;
        self.shift_register.write_count = 0;
    }
}
