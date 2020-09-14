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

    fn run_cpu_instruction(&mut self) -> bool {
        if let Some(mut cpu_cycles) = self.cpu.run_single_instruction() {
            let loops = cpu_cycles * 3;
            //let loops = 1;
            for _ in 0..loops {
                let nmi = self.ppu.borrow_mut().run_single_ppu_cycle();
                //let nmi = self.ppu.borrow_mut().process_cpu_cycles(cpu_cycles);
                if nmi {
                    self.cpu.nmi();
                    cpu_cycles += 7;
                }
            }
            self.apu.borrow_mut().process_cpu_cycles(cpu_cycles);
            true
        } else {
            false
        }
    }

    pub fn run(&mut self) {
        while self.run_cpu_instruction() {}
    }
}
