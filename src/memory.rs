use crate::common;
use crate::ppu;
use crate::ppu::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator; 

type CpuMemPpuWriteAccessRegisterMapping =  HashMap<u16, WriteAccessRegister>;
type CpuMemPpuReadAccessRegisterMapping  =  HashMap<u16, ReadAccessRegister>;


pub trait Memory {
    fn get_byte(&self, addr : u16) -> u8;

    fn get_2_bytes_as_u16(&self, addr : u16) -> u16;

    fn store_byte(&mut self, addr : u16, byte : u8);

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>);

    fn store_2_bytes_as_u16(&mut self, addr : u16, bytes : u16);
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

    fn get_2_bytes_as_u16(&self, addr : u16) -> u16 {     
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
    fn store_2_bytes_as_u16(&mut self, addr : u16, bytes : u16 ) {
        self.memory[addr as usize]     = (bytes & 0x00FF) as u8;
        self.memory[addr as usize + 1] = (bytes & 0xFF00) as u8;
    }
}


pub struct CpuRAM<'a> {
    memory            : [u8; 65536],
    ppu_access        : &'a RefCell::<dyn PpuRegisterAccess>,
    ppu_read_reg_map  : CpuMemPpuReadAccessRegisterMapping,
    ppu_write_reg_map : CpuMemPpuWriteAccessRegisterMapping,
}

impl<'a> CpuRAM<'a>{
    pub fn new(ppu_access : &RefCell::<dyn PpuRegisterAccess>) -> CpuRAM {
        let mut ppu_read_reg_map : CpuMemPpuReadAccessRegisterMapping = HashMap::new();
        for read_reg in ReadAccessRegister::iterator() {
            ppu_read_reg_map.insert(*read_reg as u16, *read_reg);
        }
        let mut ppu_write_reg_map : CpuMemPpuWriteAccessRegisterMapping = HashMap::new();
        for write_reg in WriteAccessRegister::iterator() {
            ppu_write_reg_map.insert(*write_reg as u16, *write_reg);
        }

        CpuRAM {
            memory            : [0 ; 65536],
            ppu_access        : ppu_access,
            ppu_read_reg_map  : ppu_read_reg_map,                           
            ppu_write_reg_map : ppu_write_reg_map
        }
    }
}

impl<'a> Memory for CpuRAM<'a> 
{
    fn get_byte(&self, addr : u16) -> u8 {
        if self.ppu_read_reg_map.contains_key(&addr) {
            let reg = self.ppu_read_reg_map.get(&addr).expect("store_byte: missing read register entry");
            //panic!("Reading from PPU!");
            self.ppu_access.borrow_mut().read(*reg)
        } else if self.ppu_write_reg_map.contains_key(&addr) {
            panic!("Attempting to read from a Ppu write access register {:#X}",addr);   
        } else { 
            self.memory[addr as usize]
        }
    }

    fn get_2_bytes_as_u16(&self, addr : u16) -> u16 {     
        common::convert_2u8_to_u16(self.memory[addr as usize], self.memory[addr as usize + 1])
    }

    fn store_byte(&mut self, addr : u16, byte : u8) {
       if self.ppu_write_reg_map.contains_key(&addr) {
            let reg = self.ppu_write_reg_map.get(&addr).expect("store_byte: missing write register entry");
            //panic!("writing to PPU {:?}",*reg);
            self.ppu_access.borrow_mut().write(*reg, byte);
       } else if addr == ppu::DmaWriteAccessRegister::OamDma as u16 {
            let mut dma_data = [0;256];
            for (i, e) in dma_data.iter_mut().enumerate() {
                let page_adress = (byte as u16)<< 8; 
                *e = self.get_byte(page_adress + i as u16);
            }
            self.memory[ppu::DmaWriteAccessRegister::OamDma as usize] = byte;
            self.ppu_access.borrow_mut().writeOamDma(dma_data);
       } else if self.ppu_read_reg_map.contains_key(&addr) {
            panic!("Attempting to write to a read Ppu register");   
       } else {
            self.memory[addr as usize] = byte;
       }
    }

    fn store_bytes(&mut self, addr : u16, bytes : &Vec<u8>){
        for (i, b) in bytes.iter().enumerate()
        {
            self.memory[(addr as usize) + i] = *b;
        }
    }

    fn store_2_bytes_as_u16(&mut self, addr : u16, bytes : u16 ) {
        self.memory[addr as usize]     = (bytes & 0x00FF) as u8;
        self.memory[addr as usize + 1] = ((bytes & 0xFF00) >>8) as u8;
    }


}