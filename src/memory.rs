pub struct RAM {
    memory : [u8; 4096]
}

impl RAM 
{
    pub fn get_byte(&self, addr : u16) -> u8 {
            self.memory[addr as usize]
    }

    pub fn get_byte_from_3_nibbles(&self, n1 :u8 , n2: u8, n3: u8) -> u8 {
            let addres_u16 =  crate::utils::convert_3n_to_u16(n1, n2, n3);
            self.memory[addres_u16 as usize]
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
            memory : [0 ; 4096]
        }
    }

}
