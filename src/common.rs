pub const PRG_ROM_UNIT_SIZE : usize = 16384;
pub const CHR_ROM_UNIT_SIZE : usize = 8192;
pub const PRG_RAM_UNIT_SIZE : usize = 8192;

pub fn convert_2u8_to_u16(b0 :u8 ,b1: u8) -> u16 {
     (b0 as u16)  |  ((b1 as u16)<<8)
}  

#[derive(Copy,Clone,Debug)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
}

