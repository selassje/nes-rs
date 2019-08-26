pub fn convert_3n_to_u16(n1 :u8 ,n2: u8, n3: u8) -> u16 {
    let hi_nibble  = (n1 as u16) << 8;
    let mid_nibble = (n2 as u16) << 4;
    let low_nibble = n3 as u16;
    hi_nibble + mid_nibble + low_nibble
}  

pub fn convert_2n_to_u8(n1 :u8 ,n2: u8) -> u8 {
    convert_3n_to_u16(0, n1, n2) as u8
}  

pub fn get_bit(byte: u8, bit: u8) -> bool {
        byte & (1 << bit) != 0
}  