use super::Mapper;
use crate::nes::common::Mirroring;
use crate::nes::mappers::mapper_internal::BankSize::*;
use crate::nes::mappers::mapper_internal::MapperInternal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mapper10 {
    mapper_internal: MapperInternal,
    mirroring: Mirroring,
    chr_banks: [usize; 4],
    prg_bank: usize,
    prg_16kb_bank_count: usize,
    latch_0_fe: bool,
    latch_1_fe: bool,
}

impl Mapper10 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        let prg_16kb_bank_count = mapper_internal.get_prg_rom_bank_count(_16KB);
        assert!(
            prg_16kb_bank_count >= 1,
            "Mapper 10 requires at least 1 16KB PRG ROM banks"
        );
        Self {
            mapper_internal,
            mirroring,
            chr_banks: [0; 4],
            prg_bank: 0,
            prg_16kb_bank_count,
            latch_0_fe: false,
            latch_1_fe: false,
        }
    }
}

impl Mapper for Mapper10 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        let bank_index = if address < 0x1000 {
            if self.latch_0_fe { 1 } else { 0 }
        } else if self.latch_1_fe {
            3
        } else {
            2
        };
        let byte = self
            .mapper_internal
            .get_chr_byte(address, self.chr_banks[bank_index], _4KB);
        match address {
            0x0FD8..=0x0FDF => self.latch_0_fe = false,
            0x0FE8..=0x0FEF => self.latch_0_fe = true,
            0x1FD8..=0x1FDF => self.latch_1_fe = false,
            0x1FE8..=0x1FEF => self.latch_1_fe = true,
            _ => {}
        }
        byte
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        if (0x6000..=0x7FFF).contains(&address) {
            return self.mapper_internal.get_prg_ram_byte(address, 0, _8KB);
        }
        if (0x8000..=0xBFFF).contains(&address) {
            return self
                .mapper_internal
                .get_prg_rom_byte(address, self.prg_bank, _16KB);
        }
        let bank = self.prg_16kb_bank_count - 1;
        self.mapper_internal.get_prg_rom_byte(address, bank, _16KB)
    }

    fn power_cycle(&mut self) {
        self.chr_banks = [0; 4];
        self.prg_bank = 0;
        self.latch_0_fe = false;
        self.latch_1_fe = false;
        self.mapper_internal.power_cycle();
    }

    fn store_chr_byte(&mut self, _address: u16, _byte: u8) {}

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        if (0x6000..=0x7FFF).contains(&address) {
            self.mapper_internal
                .store_prg_ram_byte(address, 0, _8KB, byte);
        }
        if (0xA000..=0xAFFF).contains(&address) {
            self.prg_bank = (byte & 0xF) as usize;
        }
        if (0xB000..=0xEFFF).contains(&address) {
            let bank = (byte & 0b11111) as usize;
            let bank_index = match address {
                0xB000..=0xBFFF => 0,
                0xC000..=0xCFFF => 1,
                0xD000..=0xDFFF => 2,
                0xE000..=0xEFFF => 3,
                _ => unreachable!(),
            };
            self.chr_banks[bank_index] = bank;
        }
        if (0xF000..=0xFFFF).contains(&address) {
            self.mirroring = if byte & 0b1 == 0 {
                Mirroring::VERTICAL
            } else {
                Mirroring::HORIZONTAL
            };
        }
    }
}
