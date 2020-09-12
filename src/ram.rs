use crate::common;
use crate::ram_apu;
use crate::ram_controllers::*;
use crate::ram_ppu::*;
use crate::{mapper::Mapper, memory::*};
use std::collections::HashMap;
use std::rc::Rc;
use std::{cell::RefCell, ops::Range};

const INTERNAL_START: u16 = 0x0000;
const INTERNAL_END: u16 = 0x2000;
const INTERNAL_MIRROR_SIZE: u16 = 0x0800;

const INTERNAL_RAM_RANGE: Range<u16> = Range {
    start: INTERNAL_START,
    end: INTERNAL_END,
};

const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_END: u16 = 0x4000;
const PPU_REGISTERS_MIRROR_SIZE: u16 = 0x0008;

const PPU_REGIGERS_RANGE: Range<u16> = Range {
    start: PPU_REGISTERS_START,
    end: PPU_REGISTERS_END,
};

type CpuMemPpuWriteAccessRegisterMapping = HashMap<u16, WriteAccessRegister>;
type CpuMemPpuReadAccessRegisterMapping = HashMap<u16, ReadAccessRegister>;
type CpuMemControllerOutputPortsMapping = HashMap<u16, OutputPort>;
type CpuMemControllerInputPortsMapping = HashMap<u16, InputPort>;
type CpuMemApuWriteAccessRegisterMapping = HashMap<u16, ram_apu::WriteAccessRegister>;
type CpuMemApuReadAccessRegisterMapping = HashMap<u16, ram_apu::ReadAccessRegister>;

pub struct RAM {
    memory: [u8; 65536],
    mapper: Option<Box<dyn Mapper>>,
    ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
    controller_access: Rc<RefCell<dyn ControllerPortsAccess>>,
    apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
    ppu_read_reg_map: CpuMemPpuReadAccessRegisterMapping,
    ppu_write_reg_map: CpuMemPpuWriteAccessRegisterMapping,
    controller_output_ports: CpuMemControllerOutputPortsMapping,
    controller_input_ports: CpuMemControllerInputPortsMapping,
    apu_read_reg_map: CpuMemApuReadAccessRegisterMapping,
    apu_write_reg_map: CpuMemApuWriteAccessRegisterMapping,
}

impl RAM {
    pub fn new(
        ppu_access: Rc<RefCell<dyn PpuRegisterAccess>>,
        controller_access: Rc<RefCell<dyn ControllerPortsAccess>>,
        apu_access: Rc<RefCell<dyn ram_apu::ApuRegisterAccess>>,
    ) -> RAM {
        let mut ppu_read_reg_map: CpuMemPpuReadAccessRegisterMapping = HashMap::new();
        for read_reg in ReadAccessRegister::iterator() {
            ppu_read_reg_map.insert(*read_reg as u16, *read_reg);
        }
        let mut ppu_write_reg_map: CpuMemPpuWriteAccessRegisterMapping = HashMap::new();
        for write_reg in WriteAccessRegister::iterator() {
            ppu_write_reg_map.insert(*write_reg as u16, *write_reg);
        }
        let mut controller_output_ports: CpuMemControllerOutputPortsMapping = HashMap::new();
        for output_port in OutputPort::iterator() {
            controller_output_ports.insert(*output_port as u16, *output_port);
        }
        let mut controller_input_ports: CpuMemControllerInputPortsMapping = HashMap::new();
        for input_port in InputPort::iterator() {
            controller_input_ports.insert(*input_port as u16, *input_port);
        }

        let mut apu_read_reg_map: CpuMemApuReadAccessRegisterMapping = HashMap::new();
        for read_reg in ram_apu::ReadAccessRegister::iterator() {
            apu_read_reg_map.insert(*read_reg as u16, *read_reg);
        }
        let mut apu_write_reg_map: CpuMemApuWriteAccessRegisterMapping = HashMap::new();
        for write_reg in ram_apu::WriteAccessRegister::iterator() {
            apu_write_reg_map.insert(*write_reg as u16, *write_reg);
        }

        RAM {
            memory: [0; 65536],
            mapper: None,
            ppu_access: ppu_access,
            controller_access: controller_access,
            apu_access: apu_access,
            ppu_read_reg_map: ppu_read_reg_map,
            ppu_write_reg_map: ppu_write_reg_map,
            controller_output_ports,
            controller_input_ports,
            apu_read_reg_map,
            apu_write_reg_map,
        }
    }

    pub fn load_mapper(&mut self, mapper: Box<dyn Mapper>) {
        self.memory.iter_mut().for_each(|m| *m = 0);
        self.store_bytes(mapper.get_rom_start(), &mapper.get_pgr_rom().to_vec());
        self.mapper = Some(mapper);
    }
}

impl Memory for RAM {
    fn get_byte(&self, addr: u16) -> u8 {
        let addr = if PPU_REGIGERS_RANGE.contains(&addr) {
            PPU_REGISTERS_START + addr % PPU_REGISTERS_MIRROR_SIZE
        } else {
            addr
        };

        if self.ppu_read_reg_map.contains_key(&addr) {
            let reg = self
                .ppu_read_reg_map
                .get(&addr)
                .expect("store_byte: missing read register entry");
            self.ppu_access.borrow_mut().read(*reg)
        } else if self.apu_read_reg_map.contains_key(&addr) {
            let reg = self
                .apu_read_reg_map
                .get(&addr)
                .expect("store_byte: missing read register entry");
            self.apu_access.borrow_mut().read(*reg)
        } else if self.controller_input_ports.contains_key(&addr) {
            let port = self
                .controller_input_ports
                .get(&addr)
                .expect("store_byte: missing input port entry");
            self.controller_access.borrow_mut().read(*port)
        } else if self.ppu_write_reg_map.contains_key(&addr) {
            println!(
                "Attempting to read from a Ppu write access register {:#X}",
                addr
            );
            0
        } else if self.controller_output_ports.contains_key(&addr) {
            panic!(
                "Attempting to read from the controller output port {:#X}",
                addr
            );
        } else if self.apu_write_reg_map.contains_key(&addr) {
            panic!(
                "Attempting to read from a Apu write access register {:#X}",
                addr
            );
        } else {
            self.memory[addr as usize]
        }
    }

    fn get_word(&self, addr: u16) -> u16 {
        common::convert_2u8_to_u16(self.memory[addr as usize], self.memory[addr as usize + 1])
    }

    fn store_byte(&mut self, addr: u16, byte: u8) {
        let addr = if PPU_REGIGERS_RANGE.contains(&addr) {
            PPU_REGISTERS_START + addr % PPU_REGISTERS_MIRROR_SIZE
        } else {
            addr
        };
        if self.ppu_write_reg_map.contains_key(&addr) {
            let reg = self
                .ppu_write_reg_map
                .get(&addr)
                .expect("store_byte: missing write register entry");
            self.ppu_access.borrow_mut().write(*reg, byte);
        } else if addr == DmaWriteAccessRegister::OamDma as u16 {
            let mut dma_data = [0; 256];
            for (i, e) in dma_data.iter_mut().enumerate() {
                let page_adress = (byte as u16) << 8;
                *e = self.get_byte(page_adress + i as u16);
            }
            self.memory[DmaWriteAccessRegister::OamDma as usize] = byte;
            self.ppu_access.borrow_mut().write_oam_dma(dma_data);
        } else if self.controller_output_ports.contains_key(&addr) {
            let port = self
                .controller_output_ports
                .get(&addr)
                .expect("store_byte: missing output port entry");
            self.controller_access.borrow_mut().write(*port, byte);
        } else if self.apu_write_reg_map.contains_key(&addr) {
            let reg = self
                .apu_write_reg_map
                .get(&addr)
                .expect("store_byte: missing apu write register entry");
            self.apu_access.borrow_mut().write(*reg, byte);
        } else if self.controller_input_ports.contains_key(&addr) {
        } else if self.ppu_read_reg_map.contains_key(&addr) {
            panic!("Attempting to write to a read Ppu register");
        } else if self.apu_read_reg_map.contains_key(&addr) {
            panic!("Attempting to write to a read Apu register");
        } else if INTERNAL_RAM_RANGE.contains(&addr) {
            let mirrors = common::get_mirrors(addr, INTERNAL_MIRROR_SIZE, INTERNAL_RAM_RANGE);
            for m in mirrors {
                self.memory[m as usize] = byte;
            }
        } else {
            self.memory[addr as usize] = byte;
        }
    }

    fn store_bytes(&mut self, addr: u16, bytes: &Vec<u8>) {
        for (i, b) in bytes.iter().enumerate() {
            self.memory[(addr as usize) + i] = *b;
        }
    }

    fn store_word(&mut self, addr: u16, bytes: u16) {
        self.memory[addr as usize] = (bytes & 0x00FF) as u8;
        self.memory[addr as usize + 1] = ((bytes & 0xFF00) >> 8) as u8;
    }
}

impl CpuMemory for RAM {
    fn get_code_segment(&self) -> (u16, u16) {
        if let Some(ref mapper) = self.mapper {
            return (
                mapper.get_rom_start(),
                mapper.get_rom_start() - 1 + mapper.get_pgr_rom().len() as u16,
            );
        } else {
            (0, 0)
        }
    }
}
