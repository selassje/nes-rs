use crate::memory::{Memory,RAM};

pub trait Mapper : Memory {

    fn load_rom(&mut self);

    fn get_rom_size(&self) -> u16;

    fn get_rom_start(&self) -> u16;
}

pub struct Mapper0 {
    cpu_ram: RAM,
    prg_rom: Vec::<u8>,
    prg_ram: Vec::<u8>,
}

impl Mapper0 {

    pub fn new(prg_rom: Vec::<u8>, prg_ram_size: u32) -> Mapper0 {
            //assert!(prg_rom.len() == 16 * 1024 || prg_rom.len() == 32 * 1024);
            //assert!(prg_ram_size == 2 * 1024   || prg_ram_size == 4 * 1024);
            Mapper0{
                cpu_ram: RAM::new(),
                prg_rom,
                prg_ram : vec![0; prg_ram_size as usize],
            }
    }

}

impl Mapper for Mapper0 {

    fn load_rom(&mut self) {
            self.cpu_ram.store_bytes(0x8000, &self.prg_rom);
    }

    fn get_rom_size(&self) -> u16 {
        self.prg_rom.len() as u16
    }

    fn get_rom_start(&self) -> u16 {
        0x8000
    }
}

impl Memory for Mapper0 {
    fn get_byte(&self, addr : u16) -> u8 {
       match addr {
             0x6000..=0x7FFF =>  {
                 let prg_ram_addr = ((addr - 0x6000) % (self.prg_ram.len() as u16));
                 self.prg_ram[prg_ram_addr as usize]
             }
             0x8000..=0xFFFF => {
                 let prg_rom_addr = (addr - 0x8000) % (self.prg_ram.len() as u16);
                 self.prg_rom[prg_rom_addr as usize]
             }
             _ => self.cpu_ram.get_byte(addr)  
        }
    }

    fn store_byte(&mut self, addr : u16, byte : u8) {
       match addr {
             0x6000..=0x7FFF =>  {
                 let start_offset = (addr - 0x6000) % (self.prg_ram.len() as u16);
                 for mirror_addr in (0x6000 + start_offset..0x8000).step_by(start_offset as usize) {
                     self.prg_ram[mirror_addr as usize] = byte;
                 }
             }
             0x8000..=0xFFFF => {
                 panic!("Writing to PGR ROM!!");
             }
             _ => self.cpu_ram.store_byte(addr,byte)  
        }
    }

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>){
        for (i, b) in bytes.iter().enumerate()
        {
            self.store_byte(addr + (i as u16),*b);
        }
    }

}

