use crate::common;

pub trait Memory {
    fn get_byte(&self, addr : u16) -> u8;

    fn get_2_bytes(&self, addr : u16) -> u16;

    fn store_byte(&mut self, addr : u16, byte : u8);

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>);
}


pub struct RAM {
    memory : [u8; 65536]
}

impl RAM {
    pub fn new() -> RAM {
        RAM {
            memory : [0 ; 65536]
        }
    }
}

impl Memory for RAM 
{
    fn get_byte(&self, addr : u16) -> u8 {
        self.memory[addr as usize]
    }

    fn get_2_bytes(&self, addr : u16) -> u16 {     
        common::convert_2u8_to_u16(self.memory[addr as usize], self.memory[addr as usize + 1])
    }

    fn store_byte(&mut self, addr : u16, byte : u8){
        self.memory[addr as usize] = byte;
    }

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>){
        for (i, b) in bytes.iter().enumerate()
        {
            self.memory[(addr as usize) + i] = *b;
        }
    }
}
