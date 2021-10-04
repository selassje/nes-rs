use crate::common;

use serde::{Deserialize, Serialize};
pub trait Memory {
    fn get_byte(&self, addr: u16) -> u8;

    fn store_byte(&mut self, addr: u16, byte: u8);

    fn get_word(&self, addr: u16) -> u16 {
        common::convert_2u8_to_u16(self.get_byte(addr), self.get_byte(addr + 1))
    }

    fn store_bytes(&mut self, addr: u16, bytes: &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            self.store_byte(addr + i as u16, *b);
        }
    }

    fn store_word(&mut self, addr: u16, word: u16) {
        self.store_byte(addr, (word & 0x00FF) as u8);
        self.store_byte(addr + 1, ((word & 0xFF00) >> 8) as u8);
    }
}

pub struct DummyMemoryImpl {}

impl DummyMemoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Memory for DummyMemoryImpl {
    fn get_byte(&self, _addr: u16) -> u8 {
        todo!()
    }

    fn store_byte(&mut self, _addr: u16, _byte: u8) {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MemoryImpl<const N: usize> {
    #[serde(with = "serde_arrays")]
    memory: [u8; N],
}

impl<const N: usize> MemoryImpl<N> {
    pub fn new() -> Self {
        Self { memory: [0; N] }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn store_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    pub fn clear(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
    }
}

pub trait VideoMemory: Memory {
    fn get_background_pallete_index(
        &self,
        table_index: u8,
        color_tile_x: u8,
        color_tile_y: u8,
    ) -> u8;

    fn get_nametable_tile_index(&self, table_index: u8, tile_x: u8, tile_y: u8) -> u8;

    fn get_pattern_table_tile_data(&self, table_index: u8, tile_index: u8) -> [u8; 16];

    fn get_universal_background_color(&self) -> u8;

    fn get_background_palette(&self, palette_index: u8) -> [u8; 3];

    fn get_attribute_data(&self, table_index: u8, color_tile_x: u8, color_tile_y: u8) -> u8;

    fn get_low_pattern_data(&self, table_index: u8, tile_index: u8, y: u8) -> u8;

    fn get_high_pattern_data(&self, table_index: u8, tile_index: u8, y: u8) -> u8;

    fn get_sprite_palette(&self, palette_index: u8) -> [u8; 3];
}

pub struct DummyVideoMemoryImpl {}

impl DummyVideoMemoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl VideoMemory for DummyVideoMemoryImpl {
    fn get_background_pallete_index(
        &self,
        _table_index: u8,
        _color_tile_x: u8,
        _color_tile_y: u8,
    ) -> u8 {
        todo!()
    }

    fn get_nametable_tile_index(&self, _table_index: u8, _tile_x: u8, _tile_y: u8) -> u8 {
        todo!()
    }

    fn get_pattern_table_tile_data(&self, _table_index: u8, _tile_index: u8) -> [u8; 16] {
        todo!()
    }

    fn get_universal_background_color(&self) -> u8 {
        todo!()
    }

    fn get_background_palette(&self, _palette_index: u8) -> [u8; 3] {
        todo!()
    }

    fn get_attribute_data(&self, _table_index: u8, _color_tile_x: u8, _color_tile_y: u8) -> u8 {
        todo!()
    }

    fn get_low_pattern_data(&self, _table_index: u8, _tile_index: u8, _y: u8) -> u8 {
        todo!()
    }

    fn get_high_pattern_data(&self, _table_index: u8, _tile_index: u8, _y: u8) -> u8 {
        todo!()
    }

    fn get_sprite_palette(&self, _palette_index: u8) -> [u8; 3] {
        todo!()
    }
}

impl Memory for DummyVideoMemoryImpl {
    fn get_byte(&self, _addr: u16) -> u8 {
        todo!()
    }

    fn store_byte(&mut self, _addr: u16, _byte: u8) {
        todo!()
    }
}

pub trait DmcMemory {
    fn set_sample_address(&mut self, address: u8);
    fn get_next_sample_byte(&mut self) -> u8;
}
