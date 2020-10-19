use super::{mapper_internal::ChrRomBankSize, mapper_internal::PrgRomBankSize, Mapper};
use crate::common::Mirroring;
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
            0 => Mirroring::VERTICAL,
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
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
#[derive(Default)]
struct ShiftRegister {
    value: u8,
    write_count: u8,
}

pub struct Mapper1 {
    mapper_internal: MapperInternal,
    shift_register: ShiftRegister,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            shift_register: Default::default(),
            control: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    fn get_chr_bank_info_from_address(&self, address: u16) -> (usize, usize) {
        if self.control.get_chr_bank_mode() == 0 {
            (self.chr_bank_0 as usize >> 1, ChrRomBankSize::_8KB as usize)
        } else {
            match address {
                0x0000..=0x0FFF => (self.chr_bank_0 as usize, ChrRomBankSize::_4KB as usize),
                0x1000..=0x1FFF => (self.chr_bank_1 as usize, ChrRomBankSize::_4KB as usize),
                _ => panic!("Incorrect CHR address {:x}", address),
            }
        }
    }
}

impl Mapper for Mapper1 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        let (bank, bank_size) = self.get_chr_bank_info_from_address(address);
        self.mapper_internal.get_chr_byte(address, bank, bank_size)
    }

    fn get_pgr_byte(&mut self, address: u16) -> u8 {
        if self.control.get_prg_bank_mode() < 2 {
            self.mapper_internal.get_pgr_byte(
                address,
                self.prg_bank as usize & 0xF >> 1,
                PrgRomBankSize::_32KB as usize,
            )
        } else {
            self.mapper_internal.get_pgr_byte(
                address,
                self.prg_bank as usize & 0xF,
                PrgRomBankSize::_16KB as usize,
            )
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        let (bank, bank_size) = self.get_chr_bank_info_from_address(address);
        self.mapper_internal
            .store_chr_byte(address, bank, bank_size, byte)
    }

    fn store_pgr_byte(&mut self, address: u16, byte: u8) {
        if byte & 0b1000_0000 != 0 {
            self.shift_register = Default::default();
            self.control |= 0x0C;
        } else {
            self.shift_register.value <<= 1;
            self.shift_register.value |= byte & 1;
            self.shift_register.write_count += 1;

            if self.shift_register.write_count == 5 {
                let register = &mut match address {
                    0x8000..=0x9FFF => self.control,
                    0xA000..=0xBFFF => self.chr_bank_0,
                    0xC000..=0xDFFF => self.chr_bank_1,
                    0xE000..=0xFFFF => self.prg_bank,
                    _ => panic!("Incorrect address"),
                };
                *register = byte;
                self.shift_register = Default::default();
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.control.get_mirroring()
    }

    fn reset(&mut self) {
        self.mapper_internal.reset();
        self.chr_bank_0 = 0;
        self.chr_bank_1 = 0;
        self.shift_register.value = 0;
        self.shift_register.write_count = 0;
    }
}
