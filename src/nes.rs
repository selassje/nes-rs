use common::FPS;

use crate::common;
use crate::controllers::Controllers;
use crate::cpu::CPU;
use crate::io::AudioAccess;
use crate::io::KeyboardAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::nes_format_reader::NesFile;
use crate::ppu::PPU;
use crate::ram::RAM;
use crate::vram::VRAM;
use crate::{apu::APU, controllers::Controller};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

pub struct Nes {
    cpu: CPU,
    ram: Rc<RefCell<RAM>>,
    ppu: Rc<RefCell<PPU>>,
    vram: Rc<RefCell<VRAM>>,
    apu: Rc<RefCell<APU>>,
    io: Rc<RefCell<dyn IO>>,
}

impl Nes {
    pub fn new<T>(
        io: Rc<RefCell<T>>,
        controller_1: Rc<dyn Controller>,
        controller_2: Rc<dyn Controller>,
    ) -> Self
    where
        T: IO + VideoAccess + AudioAccess + KeyboardAccess + 'static,
    {
        // let controller_1 = KeyboardController::get_default_keyboard_controller_player1(io.clone());
        //let controller_2 = KeyboardController::get_default_keyboard_controller_player2(io.clone());
        let controllers = Rc::new(RefCell::new(Controllers::new(controller_1, controller_2)));

        let vram = Rc::new(RefCell::new(VRAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new(vram.clone(), io.clone())));
        let apu = Rc::new(RefCell::new(APU::new(io.clone())));
        let ram = Rc::new(RefCell::new(RAM::new(
            ppu.clone(),
            controllers.clone(),
            apu.clone(),
        )));
        let cpu = CPU::new(ram.clone(), ppu.clone());

        Nes {
            cpu,
            ram,
            ppu,
            vram,
            apu,
            io,
        }
    }

    pub fn load(&mut self, nes_file: &NesFile) {
        let mapper = nes_file.create_mapper();
        self.vram.borrow_mut().load_mapper(&mapper);
        self.ppu.borrow_mut().reset();
        self.ram.borrow_mut().load_mapper(mapper);
        self.cpu.reset();
    }

    pub fn run(&mut self, duration: Option<Duration>) {
        const FRAME_DURATION: Duration =
            Duration::from_nanos((Duration::from_secs(1).as_nanos() / FPS as u128) as u64);

        let mut elapsed_frames: u128 = 0;
        let mut frame_start = Instant::now();
        while duration == None || elapsed_frames < duration.unwrap().as_secs() as u128 * FPS as u128
        {
            self.run_single_frame();
            self.io.borrow_mut().present_frame();

            let elapsed_time_since_frame_start = frame_start.elapsed();
            if elapsed_time_since_frame_start < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed_time_since_frame_start);
            }
            frame_start = Instant::now();
            elapsed_frames += 1;
        }
    }

    fn run_single_frame(&mut self) {
        let mut elapsed_cpu_cycles = 0;
        while elapsed_cpu_cycles < common::CPU_CYCLES_PER_FRAME {
            elapsed_cpu_cycles += self.run_single_cpu_instruction() as usize;
        }
    }

    fn run_single_cpu_instruction(&mut self) -> u16 {
        let cpu_cycles_for_next_instruction = self.cpu.fetch_next_instruction();
        if cpu_cycles_for_next_instruction != 0 {
            self.ppu
                .borrow_mut()
                .run_single_cpu_instruction(cpu_cycles_for_next_instruction);

            self.apu
                .borrow_mut()
                .process_cpu_cycles(cpu_cycles_for_next_instruction as u8);

            self.cpu.run_next_instruction();
        }
        cpu_cycles_for_next_instruction
    }
}
