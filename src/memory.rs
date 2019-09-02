pub struct RAM {
    memory : [u8; 65536]
}

impl RAM 
{
    pub fn get_byte(&self, addr : u16) -> u8 {
            self.memory[addr as usize]
    }

    pub fn store_byte(&mut self, addr : u16, byte : u8){
            self.memory[addr as usize] = byte;
    }

    pub fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>){
            for (i, b) in bytes.iter().enumerate()
            {
                self.memory[(addr as usize) + i] = *b;
            }
    }

    pub fn new() -> RAM {
        RAM {
            memory : [0 ; 65536]
        }
    }
}
