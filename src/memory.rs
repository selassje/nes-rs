pub trait Memory {
    fn get_byte(&self, addr: u16) -> u8;

    fn get_word(&self, addr: u16) -> u16;

    fn store_byte(&mut self, addr: u16, byte: u8);

    fn store_bytes(&mut self, addr: u16, bytes: &Vec<u8>);

    fn store_word(&mut self, addr: u16, bytes: u16);
}

pub trait CpuMemory: Memory {
    fn get_code_segment(&self) -> (u16, u16);
}

pub trait VideoMemory {
    fn store_byte(&mut self, addr: u16, byte: u8);

    fn store_bytes(&mut self, addr: u16, bytes: &Vec<u8>);

    fn get_byte(&mut self, addr: u16) -> u8;

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

    fn get_sprite_palette(&self, palette_index: u8) -> [u8; 3];
}
