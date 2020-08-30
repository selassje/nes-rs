pub trait Memory {
    fn get_byte(&self, addr : u16) -> u8;

    fn get_2_bytes_as_u16(&self, addr : u16) -> u16;

    fn store_byte(&mut self, addr : u16, byte : u8);

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>);

    fn store_2_bytes_as_u16(&mut self, addr : u16, bytes : u16);
}
