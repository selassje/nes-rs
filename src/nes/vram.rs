use self::AttributeDataQuadrantMask::*;
use super::common::Mirroring;
use super::common::NametableSource;
use super::mappers::MapperEnum;

use super::{mappers::Mapper, memory::VideoMemory};

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, ops::Range};

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

#[derive(Serialize, Deserialize, Default)]
pub struct VRam {
    memory: super::memory::MemoryImpl<0x0820>,
    read_buffer: RefCell<u8>,
}

impl VRam {
    pub fn new() -> Self {
        VRam {
            ..Default::default()
        }
    }

    pub fn power_cycle(&mut self) {
        self.memory.clear()
    }

    fn get_target_address(&self, address: u16, mapper: &MapperEnum) -> u16 {
        if NAMETABLES_RANGE.contains(&address) {
            let offset = address & 0x0FFF;
            let table = offset / 0x0400;
            let inner = offset & 0x03FF;

            let vram_page = match mapper.get_mirroring() {
                Mirroring { tables } => match tables[table as usize] {
                    NametableSource::Vram0 => 0,
                    NametableSource::Vram1 => 1,
                    _ => panic!("Fill/ExRam not supported yet"),
                },
            };

            vram_page * 0x0400 + inner
        } else if PALETTES_RANGE.contains(&address) {
            let offset = address & 0x1F;
            let mirrored = match offset {
                0x10 => 0x00,
                0x14 => 0x04,
                0x18 => 0x08,
                0x1C => 0x0C,
                _ => offset,
            };
            0x0800 + mirrored
        } else {
            panic!("Incorrect address! {:X}", address)
        }
    }

    fn get_byte_internal(&self, address: u16, mapper: &MapperEnum) -> u8 {
        if address < NAMETABLES_START {
            mapper.get_chr_byte(address)
        } else {
            self.memory
                .get_byte(self.get_target_address(address, mapper))
        }
    }

    fn get_palette(&self, start_address: u16, mapper: &MapperEnum) -> [u8; 3] {
        [
            self.get_byte_internal(start_address, mapper),
            self.get_byte_internal(start_address + 1, mapper),
            self.get_byte_internal(start_address + 2, mapper),
        ]
    }
}

impl VideoMemory for VRam {
    fn get_byte(&self, addr: u16, mapper: &MapperEnum) -> u8 {
        let addr = addr & 0x3FFF;
        let byte = self.get_byte_internal(addr, mapper);
        if PALETTES_RANGE.contains(&addr) {
            *self.read_buffer.borrow_mut() =
                self.get_byte_internal(NAMETABLES_START + (addr % NAMETABLE_MIRROR_SIZE), mapper);
            byte
        } else {
            let read_buffer = *self.read_buffer.borrow();
            *self.read_buffer.borrow_mut() = byte;
            read_buffer
        }
    }

    fn store_byte(&mut self, address: u16, byte: u8, mapper: &mut MapperEnum) {
        let address = address & 0x3FFF;
        if address < NAMETABLES_START {
            mapper.store_chr_byte(address, byte);
        } else {
            self.memory
                .store_byte(self.get_target_address(address, mapper), byte);
        }
    }
    fn get_nametable_tile_index(
        &self,
        table_index: u8,
        tile_x: u8,
        tile_y: u8,
        mapper: &MapperEnum,
    ) -> u8 {
        let name_table_addr = NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE;
        let tile_index = 32 * tile_y as u16 + tile_x as u16;
        self.get_byte_internal(name_table_addr + tile_index, mapper)
    }

    fn get_pattern_table_tile_data(
        &self,
        table_index: u8,
        tile_index: u8,
        mapper: &MapperEnum,
    ) -> [u8; 16] {
        let mut tile_data = [0; 16];
        let pattern_table_addr = table_index as u16 * PATTERN_TABLE_SIZE;
        for (i, tile_data) in tile_data.iter_mut().enumerate() {
            *tile_data = self.get_byte_internal(
                pattern_table_addr + 16 * tile_index as u16 + i as u16,
                mapper,
            );
        }
        tile_data
    }

    fn get_universal_background_color(&self, mapper: &MapperEnum) -> u8 {
        self.get_byte_internal(PALETTES_START, mapper)
    }

    fn get_background_palette(&self, palette_index: u8, mapper: &MapperEnum) -> [u8; 3] {
        self.get_palette(0x3F01 + 4 * palette_index as u16, mapper)
    }

    fn get_attribute_data(
        &self,
        table_index: u8,
        color_tile_x: u8,
        color_tile_y: u8,
        mapper: &MapperEnum,
    ) -> u8 {
        let attrib_table_addr = self.get_target_address(
            NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE + 960,
            mapper,
        );
        let attribute_index = (color_tile_y / 2) * 8 + color_tile_x / 2;
        let attribute_data = self
            .memory
            .get_byte(attrib_table_addr + attribute_index as u16);
        let quadrant: u8 = (color_tile_y % 2) * 2 + (color_tile_x % 2);
        (attribute_data & ATTRIBUTE_DATA_QUADRANT_MASKS[quadrant as usize]) >> (2 * quadrant)
    }

    fn get_low_pattern_data(
        &self,
        table_index: u8,
        tile_index: u8,
        y: u8,
        mapper: &MapperEnum,
    ) -> u8 {
        let pattern_table_addr = table_index as u16 * PATTERN_TABLE_SIZE;
        self.get_byte_internal(
            pattern_table_addr + 16 * tile_index as u16 + y as u16,
            mapper,
        )
    }

    fn get_high_pattern_data(
        &self,
        table_index: u8,
        tile_index: u8,
        y: u8,
        mapper: &MapperEnum,
    ) -> u8 {
        let pattern_table_addr = table_index as u16 * PATTERN_TABLE_SIZE;
        self.get_byte_internal(
            pattern_table_addr + 16 * tile_index as u16 + 8 + y as u16,
            mapper,
        )
    }

    fn get_sprite_palette(&self, palette_index: u8, mapper: &MapperEnum) -> [u8; 3] {
        self.get_palette(0x3F11 + 4 * palette_index as u16, mapper)
    }
}
