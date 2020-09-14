use crate::apu::APU;
use crate::controllers::Controllers;
use crate::cpu::CPU;
use crate::keyboard::KeyboardController;
use crate::nes_format_reader::NesFile;
use crate::ppu::PPU;
use crate::{ram::RAM, vram::VRAM};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    cpu: CPU,
    ram: Rc<RefCell<RAM>>,
    ppu: Rc<RefCell<PPU>>,
    vram: Rc<RefCell<VRAM>>,
    apu: Rc<RefCell<APU>>,
}

impl Nes {
    pub fn new() -> Self {
        let controller_1 = KeyboardController::get_default_keyboard_controller_player1();
        let controller_2 = KeyboardController::get_default_keyboard_controller_player2();
        let controllers = Rc::new(RefCell::new(Controllers::new(
            Box::new(controller_1),
            Box::new(controller_2),
        )));

        let vram = Rc::new(RefCell::new(VRAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new(vram.clone())));
        let apu = Rc::new(RefCell::new(APU::new()));
        let ram = Rc::new(RefCell::new(RAM::new(
            ppu.clone(),
            controllers.clone(),
            apu.clone(),
        )));
        let cpu = CPU::new(ram.clone());

        Nes {
            cpu,
            ram,
            ppu,
            vram,
            apu,
        }
    }

    pub fn load(&mut self, nes_file: &NesFile) {
        let mapper = nes_file.create_mapper();
        self.vram.borrow_mut().load_mapper(&mapper);
        self.ppu.borrow_mut().reset();
        self.ram.borrow_mut().load_mapper(mapper);
        self.cpu.reset();
    }


    pub fn run(&mut self) {
        const PPU_CYCLES_PER_CPU_CYCLE : u16 = 3;
        let mut cpu_cyles_for_next_instruction = self.cpu.fetch_next_instruction();
        while cpu_cyles_for_next_instruction != 0 {
            let mut ppu_cycles = PPU_CYCLES_PER_CPU_CYCLE * cpu_cyles_for_next_instruction;
            let mut elapsed_ppu_cycles = 0;
            while elapsed_ppu_cycles < ppu_cycles {
                if self.ppu.borrow_mut().run_single_ppu_cycle() {
                    let nmi_cpu_cycles = self.cpu.nmi() as u16;
                    ppu_cycles  = PPU_CYCLES_PER_CPU_CYCLE * (self.cpu.fetch_next_instruction() + nmi_cpu_cycles);
                } 
                elapsed_ppu_cycles +=1;
            }
            self.cpu.run_next_instruction();
            cpu_cyles_for_next_instruction = self.cpu.fetch_next_instruction();
        }
    }
}
