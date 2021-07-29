use crate::apu::APU;
use crate::common;
use crate::controllers;
use crate::io::AudioAccess;
use crate::io::ControllerAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::nes_file::NesFile;
use crate::ppu::PPU;
use crate::ram::RAM;
use crate::vram::VRAM;
use crate::{cpu::CPU, mappers::Mapper};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct Nes {
    cpu: CPU,
    ram: Rc<RefCell<RAM>>,
    ppu: Rc<RefCell<PPU>>,
    vram: Rc<RefCell<VRAM>>,
    apu: Rc<RefCell<APU>>,
    mapper: Rc<RefCell<dyn Mapper>>,
}

impl Nes {
    pub fn new<T>(io: Rc<RefCell<T>>) -> Self
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        let controllers = Rc::new(RefCell::new(controllers::Controllers::new(io.clone())));
        let mapper = Rc::new(RefCell::new(crate::mappers::MapperNull::new()));
        let vram = Rc::new(RefCell::new(VRAM::new(mapper.clone())));
        let ppu = Rc::new(RefCell::new(PPU::new(
            vram.clone(),
            io.clone(),
            mapper.clone(),
        )));
        let apu = Rc::new(RefCell::new(APU::new(io.clone())));
        let ram = Rc::new(RefCell::new(RAM::new(
            ppu.clone(),
            controllers.clone(),
            apu.clone(),
            mapper.clone(),
        )));

        apu.borrow_mut().set_dmc_memory(ram.clone());
        let cpu = CPU::new(ram.clone(), ppu.clone(), apu.clone(), mapper.clone());

        Nes {
            cpu,
            ram,
            ppu,
            vram,
            apu,
            mapper,
        }
    }

    pub fn load(&mut self, nes_file: &NesFile) {
        let mapper = nes_file.create_mapper();
        self.vram.borrow_mut().set_mapper(mapper.clone());
        self.ppu.borrow_mut().set_mapper(mapper.clone());
        self.ram.borrow_mut().set_mapper(mapper.clone());
        self.cpu.set_mapper(mapper.clone());
        self.mapper = mapper;
        self.power_cycle();
    }

    pub fn power_cycle(&mut self) {
        self.vram.borrow_mut().power_cycle();
        self.ppu.borrow_mut().power_cycle();
        self.apu.borrow_mut().power_cycle();
        self.ram.borrow_mut().power_cycle();
        self.mapper.borrow_mut().power_cycle();
        self.cpu.power_cycle();
    }

    pub fn run_for(&mut self, duration: Duration) {
        let mut elapsed_frames = 0;
        while elapsed_frames < duration.as_secs() as u128 * common::DEFAULT_FPS as u128 {
            self.run_single_frame();
            elapsed_frames += 1;
        }
    }

    pub fn run_single_frame(&mut self) {
        for _ in 0..common::CPU_CYCLES_PER_FRAME {
            self.run_single_cpu_cycle();
        }
    }

    fn run_single_cpu_cycle(&mut self) {
        self.cpu.maybe_fetch_next_instruction();

        self.ppu.borrow_mut().run_single_cpu_cycle();

        self.apu.borrow_mut().run_single_cpu_cycle();

        self.cpu.run_single_cycle();
    }
}
