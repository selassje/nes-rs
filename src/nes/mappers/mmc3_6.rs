use std::ops::Range;

use super::Mapper;
use super::mapper_internal::BankSelect;
use super::mapper_internal::BankSize::*;
use super::mapper_internal::MapperInternal;
use crate::nes::common::Mirroring;
use serde::{Deserialize, Serialize};

const PRG_RAM_RANGE: Range<u16> = Range {
    start: 0x6000,
    end: 0x8000,
};

#[derive(Serialize, Deserialize)]
pub(super) enum MMC3_6Variant {
    MMC3HkROM,
    _MMC3TkTlSROM,
    _MMC3TqSROM,
    _MMC6,
}

trait BankSelectRegister {
    fn get_selected_bank(&self) -> usize;
    fn get_prg_rom_mode(&self) -> usize;
    fn get_chr_inversion_mode(&self) -> usize;
}

impl BankSelectRegister for u8 {
    fn get_selected_bank(&self) -> usize {
        (self & 0b0000_0111) as usize
    }

    fn get_prg_rom_mode(&self) -> usize {
        ((self & 0b0100_0000) >> 6) as usize
    }

    fn get_chr_inversion_mode(&self) -> usize {
        ((self & 0b1000_0000) >> 7) as usize
    }
}
#[derive(Serialize, Deserialize)]
pub(super) struct MMC3_6 {
    mapper_internal: MapperInternal,
    _variant: MMC3_6Variant,
    prg_rom_banks: [BankSelect; 4],
    chr_rom_banks: [BankSelect; 8],
    prg_rom_banks_count: usize,
    bank_select: u8,
    bank_data: u8,
    mirroring: u8,
    prg_ram_protect: u8,
    scanline_counter_reload_value: u8,
    reload_scanline_counter_at_next_edge: bool,
    irq_enabled: bool,
    irq_triggered: bool,
    scanline_counter: u8,
}

impl MMC3_6 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, variant: MMC3_6Variant) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        let prg_rom_banks_count = mapper_internal.get_prg_rom_bank_count(_8KB);
        let mut mapper = Self {
            mapper_internal,
            _variant: variant,
            prg_rom_banks: [BankSelect {
                size: _8KB,
                bank: prg_rom_banks_count - 1,
            }; 4],
            chr_rom_banks: [Default::default(); 8],
            prg_rom_banks_count,
            bank_select: 0,
            bank_data: 0,
            mirroring: 0,
            prg_ram_protect: 0,
            scanline_counter_reload_value: 0,
            reload_scanline_counter_at_next_edge: false,
            irq_enabled: false,
            irq_triggered: false,
            scanline_counter: 0,
        };
        mapper.init_bank_mapping();
        mapper
    }

    fn init_bank_mapping(&mut self) {
        self.prg_rom_banks[0].bank = 0;
        self.prg_rom_banks[0].size = _8KB;
        self.prg_rom_banks[1].bank = 1;
        self.prg_rom_banks[1].size = _8KB;
        self.prg_rom_banks[2].bank = 2;
        self.prg_rom_banks[2].size = _8KB;
        self.prg_rom_banks[3].bank = self.prg_rom_banks_count - 1;
        self.prg_rom_banks[3].size = _8KB;

        self.chr_rom_banks[0].bank = 0;
        self.chr_rom_banks[0].size = _2KB;
        self.chr_rom_banks[1].bank = 0;
        self.chr_rom_banks[1].size = _2KB;
        self.chr_rom_banks[2].bank = 1;
        self.chr_rom_banks[2].size = _2KB;
        self.chr_rom_banks[3].bank = 1;
        self.chr_rom_banks[3].size = _2KB;
        self.chr_rom_banks[4].bank = 4;
        self.chr_rom_banks[4].size = _1KB;
        self.chr_rom_banks[5].bank = 5;
        self.chr_rom_banks[5].size = _1KB;
        self.chr_rom_banks[6].bank = 6;
        self.chr_rom_banks[6].size = _1KB;
        self.chr_rom_banks[7].bank = 7;
        self.chr_rom_banks[7].size = _1KB;
    }

    fn update_selected_bank(&mut self, selected_bank: usize) {
        if selected_bank < 6 {
            let mode = self.bank_select.get_chr_inversion_mode();
            let _1kb_bank = (self.bank_data) as usize;
            let _2kb_bank = (self.bank_data & 0b1111_1110) as usize;
            let _2kb_bank = (self.bank_data >> 1) as usize;
            const CHR_MAP: [[(usize, usize); 2]; 6] = [
                [(0, 1), (4, 5)],
                [(2, 3), (6, 7)],
                [(4, 4), (0, 0)],
                [(5, 5), (1, 1)],
                [(6, 6), (2, 2)],
                [(7, 7), (3, 3)],
            ];
            let (bank_index_1, bank_index_2) = CHR_MAP[selected_bank][mode];

            if bank_index_1 != bank_index_2 {
                self.chr_rom_banks[bank_index_1].size = _2KB;
                self.chr_rom_banks[bank_index_1].bank = _2kb_bank;
                self.chr_rom_banks[bank_index_2].size = _2KB;
                self.chr_rom_banks[bank_index_2].bank = _2kb_bank;
            } else {
                self.chr_rom_banks[bank_index_1].size = _1KB;
                self.chr_rom_banks[bank_index_1].bank = _1kb_bank;
            }
        } else {
            let mode = self.bank_select.get_prg_rom_mode();
            let bank = (self.bank_data & 0b00111111) as usize;
            if selected_bank == 6 {
                if mode == 0 {
                    self.prg_rom_banks[0].bank = bank;
                    self.prg_rom_banks[2].bank = self.prg_rom_banks_count - 2;
                } else {
                    self.prg_rom_banks[0].bank = self.prg_rom_banks_count - 2;
                    self.prg_rom_banks[2].bank = bank;
                }
            } else {
                self.prg_rom_banks[1].bank = self.bank_data as usize;
            }
        }
    }
}

impl Mapper for MMC3_6 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        let bank_select = self.chr_rom_banks[address as usize / _1KB as usize];
        self.mapper_internal
            .get_chr_byte(address, bank_select.bank, bank_select.size)
    }

    fn get_prg_byte(&self, address: u16) -> u8 {
        if PRG_RAM_RANGE.contains(&address) {
            self.mapper_internal.get_prg_ram_byte(address, 0, _8KB)
        } else if address >= PRG_RAM_RANGE.end {
            let bank_select =
                self.prg_rom_banks[(address - PRG_RAM_RANGE.end) as usize / _8KB as usize];
            self.mapper_internal
                .get_prg_rom_byte(address, bank_select.bank, bank_select.size)
        } else {
            0
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mapper_internal.store_chr_byte(address, 0, _1KB, byte);
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        if PRG_RAM_RANGE.contains(&address) {
            self.mapper_internal
                .store_prg_ram_byte(address, 0, _8KB, byte)
        } else {
            let is_even = address.is_multiple_of(2);
            match address {
                0x8000..=0x9FFF => {
                    if is_even {
                        self.bank_select = byte;
                    } else {
                        self.bank_data = byte;
                        self.update_selected_bank(self.bank_select.get_selected_bank());
                    }
                }
                0xA000..=0xBFFF => {
                    if is_even {
                        self.mirroring = byte;
                    } else {
                        self.prg_ram_protect = byte;
                    }
                }
                0xC000..=0xDFFF => {
                    if is_even {
                        self.scanline_counter_reload_value = byte;
                    } else {
                        self.scanline_counter = 0;
                        self.reload_scanline_counter_at_next_edge = true;
                    }
                }
                0xE000..=0xFFFF => {
                    self.irq_enabled = !is_even;
                    if !self.irq_enabled {
                        self.irq_triggered = false;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        if self.mirroring & 1 == 1 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {
        if self.reload_scanline_counter_at_next_edge || self.scanline_counter == 0 {
            self.scanline_counter = self.scanline_counter_reload_value;
        } else {
            self.scanline_counter -= 1;
        }
        self.reload_scanline_counter_at_next_edge = false;

        if self.scanline_counter == 0 && self.irq_enabled {
            self.irq_triggered = true;
        }
    }

    fn is_irq_pending(&self) -> bool {
        self.irq_triggered
    }

    fn power_cycle(&mut self) {
        self.mapper_internal.reset();
        self.init_bank_mapping();
        self.bank_select = 0;
        self.bank_data = 0;
        self.mirroring = 0;
        self.prg_ram_protect = 0;
        self.reload_scanline_counter_at_next_edge = false;
        self.scanline_counter_reload_value = 0;
        self.irq_enabled = false;
        self.irq_triggered = false;
        self.scanline_counter = 0;
    }
}
