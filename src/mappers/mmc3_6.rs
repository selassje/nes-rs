use std::ops::Range;

use super::Mapper;
use crate::common::Mirroring;
use crate::mappers::mapper_internal::BankSelect;
use crate::mappers::mapper_internal::BankSize::*;
use crate::mappers::mapper_internal::MapperInternal;

const PRG_RAM_RANGE: Range<u16> = Range {
    start: 0x6000,
    end: 0x8000,
};

#[allow(dead_code)]
pub(super) enum MMC3_6Variant {
    MMC3HkROM,
    MMC3TkTlSROM,
    MMC3TqSROM,
    MMC6,
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
#[allow(dead_code)]
pub(super) struct MMC3_6 {
    mapper_internal: MapperInternal,
    variant: MMC3_6Variant,
    prg_rom_banks: [BankSelect; 4],
    chr_rom_banks: [BankSelect; 8],
    prg_rom_banks_count: usize,
    bank_select: u8,
    is_bank_select_initialized: bool,
    bank_data: u8,
    mirroring: u8,
    prg_ram_protect: u8,
    reload_irq_counter_at_zero: u8,
    reload_irq_counter_at_next_edge: Option<u8>,
    irq_enabled: bool,
}

impl MMC3_6 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, variant: MMC3_6Variant) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        let prg_rom_banks_count = mapper_internal.get_prg_rom_bank_count(_8KB);
        Self {
            mapper_internal,
            variant,
            prg_rom_banks: [BankSelect {
                size: _8KB,
                bank: prg_rom_banks_count - 1,
            }; 4],
            chr_rom_banks: [Default::default(); 8],
            prg_rom_banks_count,
            bank_select: 0,
            is_bank_select_initialized: false,
            bank_data: 0,
            mirroring: 0,
            prg_ram_protect: 0,
            reload_irq_counter_at_zero: 0,
            reload_irq_counter_at_next_edge: None,
            irq_enabled: false,
        }
    }

    fn init_all_banks(&mut self) {
        for i in 0..8 {
            self.update_selected_bank(i);
        }
    }

    fn update_selected_bank(&mut self, selected_bank: usize) {
        if selected_bank < 6 {
            let mode = self.bank_select.get_chr_inversion_mode() as usize;
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
            let mode = self.bank_select.get_prg_rom_mode() as usize;
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
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        let bank_select = self.chr_rom_banks[address as usize / _1KB as usize];
        let val = self
            .mapper_internal
            .get_chr_byte(address, bank_select.bank, bank_select.size);

        if false {
            println!(
                "bank size {:?} bank {:?} real {:X}",
                bank_select.size,
                bank_select.bank,
                bank_select.bank as usize * bank_select.size as usize
            );
        }
        val
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        if PRG_RAM_RANGE.contains(&address) {
            self.mapper_internal.get_prg_ram_byte(address, 0, _8KB)
        } else {
            let bank_select =
                self.prg_rom_banks[(address - PRG_RAM_RANGE.end) as usize / _8KB as usize];
            //panic!("get at {:X} index {}", address,bank_select.bank);
            self.mapper_internal
                .get_prg_rom_byte(address, bank_select.bank, bank_select.size)
        }
    }

    fn store_chr_byte(&mut self, _: u16, _: u8) {}

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        if PRG_RAM_RANGE.contains(&address) {
            self.mapper_internal
                .store_prg_ram_byte(address, 0, _8KB, byte)
        } else {
            let is_even = address % 2 == 0;
            match address {
                0x8000..=0x9FFF => {
                    if is_even {
                        self.bank_select = byte;
                        if !self.is_bank_select_initialized {
                            self.init_all_banks();
                            self.is_bank_select_initialized = true;
                        }
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
                        self.reload_irq_counter_at_zero = byte;
                    } else {
                        self.reload_irq_counter_at_next_edge = Some(byte)
                    }
                }
                0xE000..=0xFFFF => {
                    self.irq_enabled = !is_even;
                }
                _ => panic!("Incorrect address {:X}", address),
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

    fn reset(&mut self) {
        self.mapper_internal.reset();
        self.prg_rom_banks = [BankSelect {
            size: _8KB,
            bank: self.prg_rom_banks_count - 1,
        }; 4];
        self.chr_rom_banks = [Default::default(); 8];
        self.bank_select = 0;
        self.is_bank_select_initialized = false;
        self.bank_data = 0;
        self.mirroring = 0;
        self.prg_ram_protect = 0;
        self.reload_irq_counter_at_next_edge = None;
        self.reload_irq_counter_at_zero = 0;
        self.irq_enabled = false;
    }
}
