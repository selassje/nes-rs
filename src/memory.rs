pub struct RAM {
    memory : [u8; 4096]
}

impl RAM 
{
    pub fn get_byte(&self, addr : usize) -> u8 {
            self.memory[addr]
    }

    pub fn store_byte(&mut self, addr : usize, byte : u8){
            self.memory[addr] = byte;
    }

    pub fn store_bytes(&mut self, addr : usize, bytes : &Vec<u8>){
            for (i, b) in bytes.iter().enumerate()
            {
                self.memory[addr + i] = *b;
            }
    }

    pub fn new() -> RAM {
        RAM {
            memory : [0 ; 4096]
        }
    }

}
