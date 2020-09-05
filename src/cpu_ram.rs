use crate::common;
use crate::cpu_controllers::*;
use crate::cpu_ppu::*;
use crate::cpu_ram_apu;
use crate::memory::*;
use std::cell::RefCell;
use std::collections::HashMap;

type CpuMemPpuWriteAccessRegisterMapping = HashMap<u16, WriteAccessRegister>;
type CpuMemPpuReadAccessRegisterMapping = HashMap<u16, ReadAccessRegister>;
type CpuMemControllerOutputPortsMapping = HashMap<u16, OutputPort>;
type CpuMemControllerInputPortsMapping = HashMap<u16, InputPort>;
type CpuMemApuWriteAccessRegisterMapping = HashMap<u16, cpu_ram_apu::WriteAccessRegister>;
type CpuMemApuReadAccessRegisterMapping = HashMap<u16, cpu_ram_apu::ReadAccessRegister>;

pub struct CpuRAM<'a> {
    memory: [u8; 65536],
    ppu_access: &'a RefCell<dyn PpuRegisterAccess>,
    controller_access: &'a mut dyn ControllerPortsAccess,
    apu_access: &'a RefCell<dyn cpu_ram_apu::ApuRegisterAccess>,
    ppu_read_reg_map: CpuMemPpuReadAccessRegisterMapping,
    ppu_write_reg_map: CpuMemPpuWriteAccessRegisterMapping,
    controller_output_ports: CpuMemControllerOutputPortsMapping,
    controller_input_ports: CpuMemControllerInputPortsMapping,
    apu_read_reg_map: CpuMemApuReadAccessRegisterMapping,
    apu_write_reg_map: CpuMemApuWriteAccessRegisterMapping,
}

impl<'a> CpuRAM<'a> {
    pub fn new(
        ppu_access: &'a RefCell<dyn PpuRegisterAccess>,
        controller_access: &'a mut dyn ControllerPortsAccess,
        apu_access: &'a RefCell<dyn cpu_ram_apu::ApuRegisterAccess>,
    ) -> CpuRAM<'a> {
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
        for read_reg in cpu_ram_apu::ReadAccessRegister::iterator() {
            apu_read_reg_map.insert(*read_reg as u16, *read_reg);
        }
        let mut apu_write_reg_map: CpuMemApuWriteAccessRegisterMapping = HashMap::new();
        for write_reg in cpu_ram_apu::WriteAccessRegister::iterator() {
            apu_write_reg_map.insert(*write_reg as u16, *write_reg);
        }

        CpuRAM {
            memory: [0; 65536],
            ppu_access: ppu_access,
            controller_access: controller_access,
            apu_access,
            ppu_read_reg_map: ppu_read_reg_map,
            ppu_write_reg_map: ppu_write_reg_map,
            controller_output_ports,
            controller_input_ports,
            apu_read_reg_map,
            apu_write_reg_map,
        }
    }
}

impl<'a> Memory for CpuRAM<'a> {
    fn get_byte(&self, addr: u16) -> u8 {
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
            self.controller_access.read(*port)
        } else if self.ppu_write_reg_map.contains_key(&addr) {
            panic!(
                "Attempting to read from a Ppu write access register {:#X}",
                addr
            );
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

    fn get_2_bytes_as_u16(&self, addr: u16) -> u16 {
        common::convert_2u8_to_u16(self.memory[addr as usize], self.memory[addr as usize + 1])
    }

    fn store_byte(&mut self, addr: u16, byte: u8) {
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
            self.controller_access.write(*port, byte);
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
        } else {
            self.memory[addr as usize] = byte;
        }
    }

    fn store_bytes(&mut self, addr: u16, bytes: &Vec<u8>) {
        for (i, b) in bytes.iter().enumerate() {
            self.memory[(addr as usize) + i] = *b;
        }
    }

    fn store_2_bytes_as_u16(&mut self, addr: u16, bytes: u16) {
        self.memory[addr as usize] = (bytes & 0x00FF) as u8;
        self.memory[addr as usize + 1] = ((bytes & 0xFF00) >> 8) as u8;
    }
}
