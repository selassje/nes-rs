use crate::common::{Mirroring, convert_2u8_to_u16};
use crate::{mapper::Mapper, memory::{Memory, VideoMemory}};
use self::AttributeDataQuadrantMask::*;

use std::ops::{Range};

const     ADDRESS_SPACE       : usize      = 0x10000;
const     PATTERN_TABLE_SIZE  : u16        = 0x1000;
const     NAMETABLE_SIZE      : u16        = 0x400;
const     NAMETABLES_START    : u16        = 0x2000;
const     NAMETABLES_END      : u16        = 0x3000;
const     NAMETABLES_RANGE    : Range<u16> = Range{start: NAMETABLES_START, end: NAMETABLES_END};

const     PALETTES_START      : u16        = 0x3F00;
const     PALETTES_END        : u16        = 0x3F1F;
const     PALETTES_RANGE      : Range<u16> = Range{start: PALETTES_START, end: PALETTES_END};

enum AttributeDataQuadrantMask {
    TopLeft     = 0b00000011,
    TopRight    = 0b00001100,
    BottomLeft  = 0b00110000,
    BottomRight = 0b11000000,
}

const ATTRIBUTE_DATA_QUADRANT_MASKS : [u8; 4] = [TopLeft as u8, TopRight as u8, BottomLeft as u8, BottomRight as u8];

pub struct VRAM {
    memory    : [u8; ADDRESS_SPACE],
    mirroring : Mirroring
}

impl VRAM {
    pub fn new() -> Self {
        VRAM {
            memory    : [0 ; ADDRESS_SPACE],
            mirroring : Mirroring::VERTICAL,
        }
    }

    pub fn load_mapper(&mut self, mapper: &Box<dyn Mapper> ) {
        self.store_bytes(0, &mapper.get_chr_rom().to_vec());
        self.mirroring = mapper.get_mirroring();
    }

    fn get_attribute_table(&self, table_index: u8) -> [u8; 64] {
        let mut attribute_table = [0;64];
        let attrib_table_addr = NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE + 960;
        attribute_table.copy_from_slice(&self.memory[attrib_table_addr as usize ..  attrib_table_addr as usize + 64]);
        attribute_table
    }

    fn get_palette(&self, start_addres: u16) -> [u8;3] {
        [self.get_byte(start_addres),
         self.get_byte(start_addres + 1),
         self.get_byte(start_addres + 2)]
    }
}

impl Memory for VRAM 
{
    fn get_byte(&self, addr : u16) -> u8 {
        self.memory[addr as usize]
    }

    fn get_word(&self, addr : u16) -> u16 {     
        convert_2u8_to_u16(self.memory[addr as usize], self.memory[addr as usize + 1])
    }

    fn store_byte(&mut self, addr : u16, byte : u8){

        if NAMETABLES_RANGE.contains(&addr) {
            let mirror_address : u16 =
            match self.mirroring {
                 Mirroring::VERTICAL => match addr {
                    0x2800 ..= 0x2BFF  => addr - 0x2800 + 0x2000,
                    0x2C00 ..= 0x2FFF  => addr - 0x2C00 + 0x2400,
                    _                  => addr + 2 * NAMETABLE_SIZE
                 } 
                 Mirroring::HORIZONTAL => match addr {
                    0x2400 ..= 0x27FF  => addr - NAMETABLE_SIZE,
                    0x2C00 ..= 0x2FFF  => addr - NAMETABLE_SIZE,
                    _                  => addr + NAMETABLE_SIZE
                 } 
            };
            self.memory[mirror_address as usize] = byte;
        } else if PALETTES_RANGE.contains(&addr) {
            let maybe_mirror_address = match addr {
                0x3F00 => Some(0x3F10),
                0x3F10 => Some(0x3F00),
                0x3F04 => Some(0x3F14),
                0x3F14 => Some(0x3F04),
                0x3F18 => Some(0x3F08),
                0x3F08 => Some(0x3F18),
                0x3F0C => Some(0x3F1C),
                0x3F1C => Some(0x3F0C),
                _      => None
            };

            if let Some(mirror_address) = maybe_mirror_address {
                self.memory[mirror_address as usize] = byte;
            }
        }
        self.memory[addr as usize] = byte;
    }

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>){
        for (i, b) in bytes.iter().enumerate()
        {
            self.store_byte(addr + i as u16,*b);
        }
    }
    fn store_word(&mut self, addr : u16, bytes : u16 ) {
        self.memory[addr as usize]     = (bytes & 0x00FF) as u8;
        self.memory[addr as usize + 1] = ((bytes & 0xFF00)>>8) as u8;
    }

}

impl VideoMemory for VRAM {
    fn get_background_pallete_index(&self, table_index: u8, color_tile_x: u8, color_tile_y: u8) -> u8 {
        let attribute_table = self.get_attribute_table(table_index);
        let attribute_index = (color_tile_y / 2) * 8 + color_tile_x / 2;
        let attribute_data  = attribute_table[attribute_index as usize];
        let quadrant : u8   = (color_tile_y % 2) * 2 + (color_tile_x % 2);
        (attribute_data & ATTRIBUTE_DATA_QUADRANT_MASKS[quadrant as usize] as u8) >> (2 * quadrant)
    }

    fn get_nametable_tile_index(&self, table_index: u8, tile_x: u8, tile_y: u8) -> u8 {
        let name_table_addr = NAMETABLES_START + table_index as u16 * NAMETABLE_SIZE;
        let tile_index = 32 * tile_y as u16 + tile_x as u16;
        self.get_byte(name_table_addr + tile_index)
    }

    fn get_pattern_table_tile_data(&self, table_index: u8, tile_index: u8) -> [u8;16] {
        let mut tile_data = [0;16];
        let pattern_table_addr = table_index as u16 * PATTERN_TABLE_SIZE;
        for i in 0..16 {
            tile_data[i] = self.get_byte(pattern_table_addr + 16 * tile_index as u16 + i as u16);
        }
        tile_data
    }

    fn get_universal_background_color(&self) -> u8 {
        self.get_byte(PALETTES_START)
    }

    fn get_background_palette(&self, palette_index: u8) -> [u8;3] {
        self.get_palette(0x3F01 + 4 * palette_index as u16)
    }

    fn get_sprite_palette(&self, palette_index: u8) -> [u8;3] {
        self.get_palette(0x3F11 + 4 * palette_index as u16)
    }

}