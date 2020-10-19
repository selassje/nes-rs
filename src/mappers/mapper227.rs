use super::{
    mapper_internal::ChrRomBankSize, mapper_internal::MapperInternal,
    mapper_internal::PrgRomBankSize, Mapper,
};
use crate::common::Mirroring;

trait Register {
    fn is_last_prg_page_mode(&self) -> bool;
    fn get_prg_bank(&self) -> usize;
    fn is_mode_1_enabled(&self) -> bool;
    fn get_mirroring(&self) -> Mirroring;
    fn get_pgr_bank_size(&self) -> PrgRomBankSize;
    fn is_menu_selection_mode(&self) -> bool;
}

impl Register for u16 {
    fn is_last_prg_page_mode(&self) -> bool {
        self & 0b00000010_00000000 != 0
    }

    fn get_prg_bank(&self) -> usize {
        let bits_0_4 = (0b00000000_01111100 & self) >> 2;
        let bit_5 = (0b00000001_00000000 & self) >> 3;
        bit_5 as usize | bits_0_4 as usize
    }

    fn is_mode_1_enabled(&self) -> bool {
        self & 0b00000000_10000000 != 0
    }

    fn get_mirroring(&self) -> Mirroring {
        if self & 0b00000000_00000010 != 0 {
            Mirroring::HORIZONTAL
        } else {
            Mirroring::VERTICAL
        }
    }

    fn get_pgr_bank_size(&self) -> PrgRomBankSize {
        if self & 0b00000000_00000001 != 0 {
            PrgRomBankSize::_32KB
        } else {
            PrgRomBankSize::_16KB
        }
    }

    fn is_menu_selection_mode(&self) -> bool {
        self & 0b00000100_00000000 != 0
    }
}

pub struct Mapper227 {
    mapper_internal: MapperInternal,
    register: u16,
    bank_1: usize,
    bank_2: usize,
}

impl Mapper227 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mut mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        mapper_internal.set_chr_bank_size(ChrRomBankSize::_8KB);
        mapper_internal.set_prg_bank_size(PrgRomBankSize::_16KB);
        Self {
            mapper_internal,
            register: 0,
            bank_1: 0,
            bank_2: 0,
        }
    }
    fn update_banks(&mut self) {
        let is_bank_size_32kb = self.register.get_pgr_bank_size() == PrgRomBankSize::_32KB;
        let bank = self.register.get_prg_bank();
        let is_last_prg_page_mode = self.register.is_last_prg_page_mode();
        let is_mode_1 = self.register.is_mode_1_enabled();

        if is_mode_1 {
            if is_bank_size_32kb {
                self.bank_1 = bank >> 1;
            } else {
                self.bank_1 = bank;
                self.bank_2 = bank;
            }
        } else {
            if is_bank_size_32kb {
                self.bank_1 = bank & 0x3E;
            } else {
                self.bank_1 = bank;
            }
            if is_last_prg_page_mode {
                self.bank_2 = bank | 0x07;
            } else {
                self.bank_2 = bank & 0x38;
            }
        }
    }
}
impl Mapper for Mapper227 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mapper_internal.get_chr_byte(address, 0)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.register.get_mirroring()
    }
    fn get_pgr_byte(&mut self, address: u16) -> u8 {
        let bank = if (self.register.get_pgr_bank_size() == PrgRomBankSize::_32KB
            && self.register.is_mode_1_enabled())
            || address < 0xC000
        {
            self.bank_1
        } else {
            self.bank_2
        };
        self.mapper_internal.get_pgr_byte(address, bank)
    }

    fn reset(&mut self) {
        self.register = 0;
        self.mapper_internal.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        if !self.register.is_mode_1_enabled() {
            self.mapper_internal.store_chr_byte(address, byte)
        }
    }

    fn store_pgr_byte(&mut self, address: u16, _: u8) {
        self.register = address;
        self.mapper_internal
            .set_prg_bank_size(self.register.get_pgr_bank_size());
        self.update_banks();
    }
}
