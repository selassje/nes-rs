use super::Mapper;
use super::mapper_internal::BankSize;
use super::mapper_internal::MapperInternal;
use crate::nes::common::Mirroring;
use crate::nes::common::NametableSource;
use crate::nes::mappers::PRG_RAM_RANGE;
use crate::nes::mappers::PRG_RANGE;
use crate::nes::ram_ppu::WriteAccessRegister;

use BankSize::*;

use serde::{Deserialize, Serialize};
use serde_arrays;

const PCM_MODE_REGISTER: u16 = 0x5010;
const PRG_MODE_SELECTION_REGISTER: u16 = 0x5100;
const CHR_MODE_SELECTION_REGISTER: u16 = 0x5101;
const PRG_RAM_PROTECT_REGISTER_1: u16 = 0x5102;
const PRG_RAM_PROTECT_REGISTER_2: u16 = 0x5103;
const EXTENDED_RAM_MODE_REGISTER: u16 = 0x5104;
const NAMETABLE_MAPPING_REGISTER: u16 = 0x5105;
const FILL_MODE_TILE_REGISTER: u16 = 0x5106;
const FILL_MODE_COLOR_REGISTER: u16 = 0x5107;
const PRG_BANK_REGISTER_1: u16 = 0x5113;
const PRG_BANK_REGISTER_5: u16 = 0x5117;
const CHR_BANK_REGISTER_1: u16 = 0x5120;
const CHR_BANK_REGISTER_12: u16 = 0x512B;
const UPPER_CHR_BITS_REGISTER: u16 = 0x5130;
const SPLIT_MODE_CONTROL_REGISTER: u16 = 0x5200;
const SPLIT_MODE_SCROLL_REGISTER: u16 = 0x5201;
const SPLIT_MODE_BANK_REGISTER: u16 = 0x5202;
const IRQ_SCANLINE_COMPARE_REGISTER: u16 = 0x5203;
const IRQ_SCANLINE_STATUS_REGISTER: u16 = 0x5204;
const EXPANSION_RAM_START: u16 = 0x5C00;
const EXPANSION_RAM_END: u16 = 0x5FFF;

#[derive(Serialize, Deserialize)]
enum SpriteMode8x16 {
    Default,
    Background,
    Sprites,
}

#[derive(Serialize, Deserialize)]
pub struct Mapper5 {
    mapper_internal: MapperInternal,
    prg_selection_mode: u8,
    chr_selection_mode: u8,
    prg_ram_protect_1: u8,
    prg_ram_protect_2: u8,
    extended_ram_mode: u8,
    prg_bank_registers: [u8; 5],
    chr_bank_registers: [u8; 12],
    chr_bank_upper_bits: u8,
    fill_mode_tile: u8,
    fill_mode_color: u8,
    split_mode_control: u8,
    split_mode_scroll: u8,
    split_mode_bank: u8,
    scanline_compare_value: u8,
    scanline_counter: u8,
    scanline_irq_enabled: bool,
    scanline_irq_pending: bool,
    in_frame: bool,
    nametable_mapping: u8,
    #[serde(with = "serde_arrays")]
    expansion_ram: [u8; 1024],
    is_sprite_mode_8x16: bool,
    are_ext_features_enabled: bool,
    sprite_mode_8x16: SpriteMode8x16,
    use_ext_as_default_for_8x16_sprite_mode: bool,
}

impl Mapper5 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            prg_selection_mode: 3,
            chr_selection_mode: 3,
            prg_ram_protect_1: 0,
            prg_ram_protect_2: 0,
            extended_ram_mode: 0,
            prg_bank_registers: [0x00, 0xFF, 0xFF, 0xFF, 0xFF],
            chr_bank_registers: [0x00; 12],
            chr_bank_upper_bits: 0,
            fill_mode_tile: 0,
            fill_mode_color: 0,
            split_mode_control: 0,
            split_mode_scroll: 0,
            split_mode_bank: 0,
            scanline_compare_value: 0,
            scanline_counter: 0,
            scanline_irq_enabled: false,
            scanline_irq_pending: false,
            in_frame: false,
            nametable_mapping: 0,
            expansion_ram: [0; 1024],
            is_sprite_mode_8x16: false,
            are_ext_features_enabled: false,
            sprite_mode_8x16: SpriteMode8x16::Default,
            use_ext_as_default_for_8x16_sprite_mode: false,
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
    fn get_chr_bank_register_index_and_size(
        &self,
        address: u16,
        use_ext: bool,
    ) -> (usize, BankSize) {
        let index = (address / _1KB as u16) as usize;
        const INDEX_AND_MODE_TO_REGISTER: [[usize; 8]; 4] = [
            [7, 7, 7, 7, 7, 7, 7, 7],
            [3, 3, 3, 3, 7, 7, 7, 7],
            [1, 1, 3, 3, 5, 5, 7, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
        ];
        const INDEX_AND_MODE_TO_REGISTER_EXT: [[usize; 8]; 4] = [
            [11, 11, 11, 11, 11, 11, 11, 11],
            [11, 11, 11, 11, 11, 11, 11, 11],
            [9, 9, 11, 11, 9, 9, 11, 11],
            [8, 9, 10, 11, 8, 9, 10, 11],
        ];
        let mode = self.chr_selection_mode as usize;
        const MODE_TO_SIZE: [BankSize; 4] = [_8KB, _4KB, _2KB, _1KB];
        let register_index = if use_ext {
            INDEX_AND_MODE_TO_REGISTER_EXT[mode][index]
        } else {
            INDEX_AND_MODE_TO_REGISTER[mode][index]
        };
        (register_index, MODE_TO_SIZE[mode])
    }

    fn decode_prg_bank_register(&self, index: u8, bank_size: BankSize) -> (usize, bool) {
        let byte = self.prg_bank_registers[index as usize];
        let mut is_rom = (byte & 0b1000_0000) != 0;
        let mut bank = (byte & 0b0111_1111) as usize;

        if index == 0 {
            bank &= 0b0000_1111;
            is_rom = false;
        }
        if index == 4 {
            is_rom = true;
        }
        if bank_size == _16KB {
            bank = ((byte & 0b0111_1110) >> 1) as usize;
        }
        if bank_size == _32KB {
            bank = ((byte & 0b0111_1100) >> 2) as usize;
        }
        (bank, is_rom)
    }

    fn is_prg_ram_writable(&self) -> bool {
        (self.prg_ram_protect_1 & 0b11) == 0b10 && (self.prg_ram_protect_2 & 0b11) == 0b01
    }
}

impl Mapper for Mapper5 {
    fn get_chr_byte(&self, address: u16) -> u8 {
        let mut use_ext = false;
        let is_sprite_mode_8x16 = self.are_ext_features_enabled && self.is_sprite_mode_8x16;
        if is_sprite_mode_8x16 {
            match self.sprite_mode_8x16 {
                SpriteMode8x16::Background => {
                    use_ext = true;
                }
                SpriteMode8x16::Default => {
                    use_ext = self.use_ext_as_default_for_8x16_sprite_mode;
                }
                _ => {}
            }
        }
        let (register, bank_size) = self.get_chr_bank_register_index_and_size(address, use_ext);
        let bank =
            ((self.chr_bank_upper_bits as usize) << 8) | self.chr_bank_registers[register] as usize;
        self.mapper_internal.get_chr_byte(address, bank, bank_size)
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        let (register, bank_size) = self.get_chr_bank_register_index_and_size(address, false);
        let bank =
            ((self.chr_bank_upper_bits as usize) << 8) | self.chr_bank_registers[register] as usize;
        self.mapper_internal
            .store_chr_byte(address, bank, bank_size, byte);
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        let byte = match address {
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
            EXPANSION_RAM_START..=EXPANSION_RAM_END => {
                if self.extended_ram_mode >= 2 {
                    let index = (address - EXPANSION_RAM_START) as usize;
                    self.expansion_ram[index]
                } else {
                    0
                }
            }
            0x5000..=0x5BFF => 0,
            0x4020..=0x4FFF => 0,
            address if PRG_RANGE.contains(&address) => {
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let (bank, is_rom) = self.decode_prg_bank_register(index as u8, bank_size);
                if is_rom {
                    self.mapper_internal
                        .get_prg_rom_byte(address, bank, bank_size)
                } else {
                    self.mapper_internal
                        .get_prg_ram_byte(address, bank, bank_size)
                }
            }
            _ => {
                println!("Get prg byte : Unknown address ${:04X}", address);
                0
            }
        };
        byte
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        match address {
            PCM_MODE_REGISTER => {}
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
            EXTENDED_RAM_MODE_REGISTER => {
                self.extended_ram_mode = byte & 0b11;
            }
            NAMETABLE_MAPPING_REGISTER => {
                self.nametable_mapping = byte;
            }
            FILL_MODE_TILE_REGISTER => {
                self.fill_mode_tile = byte;
            }
            FILL_MODE_COLOR_REGISTER => {
                self.fill_mode_color = byte;
            }
            UPPER_CHR_BITS_REGISTER => {
                self.chr_bank_upper_bits = byte;
            }
            PRG_BANK_REGISTER_1..=PRG_BANK_REGISTER_5 => {
                let index = (address - PRG_BANK_REGISTER_1) as usize;
                self.prg_bank_registers[index] = byte;
            }
            CHR_BANK_REGISTER_1..=CHR_BANK_REGISTER_12 => {
                let index = (address - CHR_BANK_REGISTER_1) as usize;
                self.use_ext_as_default_for_8x16_sprite_mode = index > 7;
                self.chr_bank_registers[index] = byte;
            }
            SPLIT_MODE_CONTROL_REGISTER => {
                self.split_mode_control = byte;
            }
            SPLIT_MODE_SCROLL_REGISTER => {
                self.split_mode_scroll = byte;
            }
            SPLIT_MODE_BANK_REGISTER => {
                self.split_mode_bank = byte;
            }
            IRQ_SCANLINE_COMPARE_REGISTER => {
                self.scanline_compare_value = byte;
            }
            IRQ_SCANLINE_STATUS_REGISTER => {
                self.scanline_irq_enabled = byte & 0b1000_0000 != 0;
            }
            EXPANSION_RAM_START..=EXPANSION_RAM_END => {
                if self.extended_ram_mode != 3 {
                    let index = (address - EXPANSION_RAM_START) as usize;
                    self.expansion_ram[index] = byte;
                }
            }
            address if PRG_RANGE.contains(&address) => {
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let (bank, is_rom) = self.decode_prg_bank_register(index as u8, bank_size);
                if self.is_prg_ram_writable() && !is_rom {
                    self.mapper_internal
                        .store_prg_ram_byte(address, bank, bank_size, byte);
                } else {
                    println!(
                        "Mapper5: Ignored write to PRG address {:04X} with bank register {:?}, index={} RAMProtected={} ",
                        address,
                        bank,
                        index,
                        self.is_prg_ram_writable()
                    );
                }
            }
            _ => {
                println!("Store prg byte: Unknown address ${:04X}", address)
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        let mut tables = [NametableSource::Vram0; 4];
        for nametable in 0..4 {
            let mask = 0b0000_0011 << nametable * 2;
            let nametable_source = (self.nametable_mapping & mask) >> nametable * 2;
            tables[nametable as usize] = NametableSource::try_from(nametable_source).unwrap();
        }
        Mirroring { tables }
    }

    fn power_cycle(&mut self) {
        self.prg_selection_mode = 3;
        self.chr_selection_mode = 3;
        self.prg_ram_protect_1 = 0;
        self.prg_ram_protect_2 = 0;
        self.extended_ram_mode = 0;
        self.prg_bank_registers = [0x0, 0xFF, 0xFF, 0xFF, 0xFF];
        self.chr_bank_registers = [0xFF; 12];
        self.chr_bank_upper_bits = 0;
        self.fill_mode_tile = 0;
        self.fill_mode_color = 0;
        self.split_mode_control = 0;
        self.split_mode_scroll = 0;
        self.split_mode_bank = 0;
        self.scanline_compare_value = 0;
        self.scanline_counter = 0;
        self.scanline_irq_enabled = false;
        self.scanline_irq_pending = false;
        self.in_frame = false;
        self.nametable_mapping = 0;
        self.expansion_ram = [0; 1024];
        self.is_sprite_mode_8x16 = false;
        self.are_ext_features_enabled = false;
        self.sprite_mode_8x16 = SpriteMode8x16::Default;
        self.use_ext_as_default_for_8x16_sprite_mode = false;
        self.mapper_internal.power_cycle();
    }

    fn notify_scanline(&mut self) {
        if !self.in_frame {
            self.in_frame = true;
            self.scanline_counter = 0;
        } else {
            self.scanline_counter += 1;
        }
        if self.scanline_counter == self.scanline_compare_value {
            self.scanline_irq_pending = true;
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

    fn get_nametable_byte(&self, source: NametableSource, offset: u16) -> Option<u8> {
        match source {
            NametableSource::ExRam => {
                let index = (offset & 0x3FF) as usize;
                Some(self.expansion_ram[index])
            }
            NametableSource::Fill => {
                if offset & 0x3FF < 0x3C0 {
                    Some(self.fill_mode_tile)
                } else {
                    let color = self.fill_mode_color & 0x03;
                    Some(color | (color << 2) | (color << 4) | (color << 6))
                }
            }
            _ => None,
        }
    }

    fn store_nametable_byte(&mut self, source: NametableSource, offset: u16, byte: u8) -> bool {
        match source {
            NametableSource::ExRam => {
                if self.extended_ram_mode <= 1 {
                    let index = (offset & 0x3FF) as usize;
                    self.expansion_ram[index] = byte;
                }
                true
            }
            NametableSource::Fill => true,
            _ => false,
        }
    }

    fn notify_oam_dma_write(&mut self) {
        self.scanline_counter = 0;
        self.sprite_mode_8x16 = SpriteMode8x16::Sprites;
    }

    fn notify_ppu_register_write(&mut self, address: u16, value: u8) {
        if let Ok(register) = WriteAccessRegister::try_from(address) {
            match register {
                WriteAccessRegister::PpuCtrl => {
                    self.is_sprite_mode_8x16 = value & 0b0010_0000 != 0;
                }
                WriteAccessRegister::PpuMask => {
                    self.are_ext_features_enabled = value & 0b0001_1000 != 0;
                }
                WriteAccessRegister::PpuData => {
                    self.sprite_mode_8x16 = SpriteMode8x16::Default;
                }
                WriteAccessRegister::OamData => {
                    self.sprite_mode_8x16 = SpriteMode8x16::Sprites;
                }
                _ => {}
            }
        }
    }

    fn notify_background_tiles_fetch(&mut self) {
        self.sprite_mode_8x16 = SpriteMode8x16::Background;
    }
}
