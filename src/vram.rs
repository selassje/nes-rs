use self::AttributeDataQuadrantMask::*;
use crate::common::{self, Mirroring};
use crate::{mappers::Mapper, memory::VideoMemory};

use std::{cell::RefCell, ops::Range, rc::Rc};

const ADDRESS_SPACE: usize = 0x10000;
const PATTERN_TABLE_SIZE: u16 = 0x1000;
const NAMETABLE_SIZE: u16 = 0x400;
const NAMETABLE_MIRROR_SIZE: u16 = 0x1000;
const NAMETABLES_START: u16 = 0x2000;
const NAMETABLES_END: u16 = 0x3F00;
const NAMETABLES_RANGE: Range<u16> = Range {
    start: NAMETABLES_START,
    end: NAMETABLES_END,
};

const PALETTES_START: u16 = 0x3F00;
const PALETTES_END: u16 = 0x4000;
const PALETTES_RANGE: Range<u16> = Range {
    start: PALETTES_START,
    end: PALETTES_END,
};

enum AttributeDataQuadrantMask {
    TopLeft = 0b00000011,
    TopRight = 0b00001100,
    BottomLeft = 0b00110000,
    BottomRight = 0b11000000,
}

const ATTRIBUTE_DATA_QUADRANT_MASKS: [u8; 4] = [
    TopLeft as u8,
    TopRight as u8,
    BottomLeft as u8,
    BottomRight as u8,
];

pub struct VRAM {
    memory: [u8; ADDRESS_SPACE],
    mapper: Rc<RefCell<dyn Mapper>>,
    read_buffer: u8,
}

impl VRAM {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> Self {
        VRAM {
            memory: [0; ADDRESS_SPACE],
            mapper,
            read_buffer: 0,
        }
    }

    pub fn reset(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
    }

    fn get_attribute_table(&self, table_index: u8) -> [u8; 64] {
        let mut attribute_table = [0; 64];
        let attrib_table_addr = NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE + 960;
        attribute_table.copy_from_slice(
            &self.memory[attrib_table_addr as usize..attrib_table_addr as usize + 64],
        );
        attribute_table
    }

    fn get_byte(&self, address: u16) -> u8 {
        if address < NAMETABLES_START {
            self.mapper.borrow_mut().get_chr_byte(address)
        } else {
            self.memory[address as usize]
        }
    }

    fn get_palette(&self, start_addres: u16) -> [u8; 3] {
        [
            self.get_byte(start_addres),
            self.get_byte(start_addres + 1),
            self.get_byte(start_addres + 2),
        ]
    }

    fn get_nametable_mirrors(&self, addr: u16) -> Vec<u16> {
        let mut mirrors = common::get_mirrors(
            addr,
            NAMETABLE_MIRROR_SIZE,
            Range {
                start: NAMETABLES_START,
                end: PALETTES_END,
            },
        );
        assert!(mirrors.len() == 2);
        let namespace_region_offset = addr % NAMETABLE_MIRROR_SIZE;
        let internal_mirror_offset = match self.mapper.borrow_mut().get_mirroring() {
            Mirroring::VERTICAL => match namespace_region_offset {
                0x0000..=0x03FF => 0x0800,
                0x0400..=0x07FF => 0x0C00,
                0x0800..=0x0BFF => 0x0000,
                0x0C00..=0x0FFF => 0x0400,
                _ => panic!("Unexpected nametable offset {:X}", namespace_region_offset),
            },
            Mirroring::HORIZONTAL => match namespace_region_offset {
                0x0000..=0x03FF => 0x0400,
                0x0400..=0x07FF => 0x0000,
                0x0800..=0x0BFF => 0x0C00,
                0x0C00..=0x0FFF => 0x0800,
                _ => panic!("Unexpected nametable offset {:X}", namespace_region_offset),
            },
        } + namespace_region_offset % NAMETABLE_SIZE;
        for i in 0..2 {
            let m = NAMETABLES_START + i * NAMETABLE_MIRROR_SIZE + internal_mirror_offset;
            if m < NAMETABLES_END {
                mirrors.push(m);
            }
        }
        mirrors
    }

    fn get_pallete_mirrors(&self, addr: u16) -> Vec<u16> {
        const MIRROR_SIZE: u16 = 0x20;
        let mut mirrors = common::get_mirrors(addr, MIRROR_SIZE, PALETTES_RANGE);
        let maybe_offset = match addr % MIRROR_SIZE {
            0x00 => Some(0x10),
            0x10 => Some(0x00),
            0x04 => Some(0x14),
            0x14 => Some(0x04),
            0x08 => Some(0x18),
            0x18 => Some(0x08),
            0x0C => Some(0x1C),
            0x1C => Some(0x0C),
            _ => None,
        };

        if let Some(offset) = maybe_offset {
            for i in 0..8 {
                mirrors.push(PALETTES_START + i * MIRROR_SIZE + offset);
            }
        }
        mirrors
    }
}

impl VideoMemory for VRAM {
    fn get_background_pallete_index(
        &self,
        table_index: u8,
        color_tile_x: u8,
        color_tile_y: u8,
    ) -> u8 {
        let attribute_table = self.get_attribute_table(table_index);
        let attribute_index = (color_tile_y / 2) * 8 + color_tile_x / 2;
        let attribute_data = attribute_table[attribute_index as usize];
        let quadrant: u8 = (color_tile_y % 2) * 2 + (color_tile_x % 2);
        (attribute_data & ATTRIBUTE_DATA_QUADRANT_MASKS[quadrant as usize] as u8) >> (2 * quadrant)
    }

    fn get_nametable_tile_index(&self, table_index: u8, tile_x: u8, tile_y: u8) -> u8 {
        let name_table_addr = NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE;
        let tile_index = 32 * tile_y as u16 + tile_x as u16;
        self.get_byte(name_table_addr + tile_index)
    }

    fn get_pattern_table_tile_data(&self, table_index: u8, tile_index: u8) -> [u8; 16] {
        let mut tile_data = [0; 16];
        let pattern_table_addr = table_index as u16 * PATTERN_TABLE_SIZE;
        for i in 0..16 {
            tile_data[i] = self.get_byte(pattern_table_addr + 16 * tile_index as u16 + i as u16);
        }
        tile_data
    }

    fn get_universal_background_color(&self) -> u8 {
        self.get_byte(PALETTES_START)
    }

    fn get_background_palette(&self, palette_index: u8) -> [u8; 3] {
        self.get_palette(0x3F01 + 4 * palette_index as u16)
    }

    fn get_sprite_palette(&self, palette_index: u8) -> [u8; 3] {
        self.get_palette(0x3F11 + 4 * palette_index as u16)
    }

    fn store_bytes(&mut self, addr: u16, bytes: &Vec<u8>) {
        for (i, b) in bytes.iter().enumerate() {
            self.store_byte(addr + i as u16, *b);
        }
    }

    fn store_byte(&mut self, addr: u16, byte: u8) {
        if addr < NAMETABLES_START {
            self.mapper.borrow_mut().store_chr_byte(addr, byte);
        } else if NAMETABLES_RANGE.contains(&addr) {
            let mirrors = self.get_nametable_mirrors(addr);
            for m in mirrors {
                self.memory[m as usize] = byte;
            }
            self.memory[addr as usize] = byte;
        } else if PALETTES_RANGE.contains(&addr) {
            let mirrors = self.get_pallete_mirrors(addr);
            for m in mirrors {
                self.memory[m as usize] = byte;
            }
            self.memory[addr as usize] = byte;
        }
    }

    fn get_byte(&mut self, addr: u16) -> u8 {
        let byte = (self as &VRAM).get_byte(addr);
        if PALETTES_RANGE.contains(&addr) {
            self.read_buffer = byte;
            byte
        } else {
            let read_buffer = self.read_buffer;
            self.read_buffer = byte;
            read_buffer
        }
    }
}
