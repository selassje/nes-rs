use crate::common;

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

    fn store_word(&mut self, addr: u16, bytes: u16) {
        self.store_byte(addr, (bytes & 0x00FF) as u8);
        self.store_byte(addr + 1, ((bytes & 0xFF00) >> 8) as u8);
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

pub trait DmcMemory {
    fn set_sample_address(&mut self, address: u8);
    fn get_next_sample_byte(&mut self) -> u8;
}
