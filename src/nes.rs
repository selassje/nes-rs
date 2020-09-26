use common::FPS;
use io_sdl2::IOSdl2;

use crate::keyboard::KeyboardController;
use crate::nes_format_reader::NesFile;
use crate::ppu::PPU;
use crate::{apu::APU, NesSettings};
use crate::{
    common, controllers::Controllers, io::KeyboardAccess, io::IO, io_dummy::IODummy, io_sdl2,
};
use crate::{cpu::CPU, io::AudioAccess, io::VideoAccess};
use crate::{ram::RAM, vram::VRAM};
use std::rc::Rc;
use std::time::Instant;
use std::{cell::RefCell, time::Duration};

pub struct Nes {
    cpu: CPU,
    ram: Rc<RefCell<RAM>>,
    ppu: Rc<RefCell<PPU>>,
    vram: Rc<RefCell<VRAM>>,
    apu: Rc<RefCell<APU>>,
    settings: NesSettings,
    io: Rc<RefCell<dyn IO>>,
}

impl Nes {
    fn create_io(
        title: &str,
        settings: NesSettings,
    ) -> (
        Rc<RefCell<dyn IO>>,
        Rc<RefCell<dyn VideoAccess>>,
        Rc<RefCell<dyn AudioAccess>>,
        Rc<RefCell<dyn KeyboardAccess>>,
    ) {
        if settings.test_mode {
            let io = Rc::new(RefCell::new(IODummy::new("")));
            (io.clone(), io.clone(), io.clone(), io.clone())
        } else {
            let io = Rc::new(RefCell::new(IOSdl2::new(title)));
            (io.clone(), io.clone(), io.clone(), io.clone())
        }
    }

    pub fn new(title: &str, settings: NesSettings) -> Self {
        let (io, video, audio, keyboard) = Self::create_io(title, settings);
        let controller_1 =
            KeyboardController::get_default_keyboard_controller_player1(keyboard.clone());
        let controller_2 =
            KeyboardController::get_default_keyboard_controller_player2(keyboard.clone());
        let controllers = Rc::new(RefCell::new(Controllers::new(
            Box::new(controller_1),
            Box::new(controller_2),
        )));

        let vram = Rc::new(RefCell::new(VRAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new(vram.clone(), video.clone())));
        let apu = Rc::new(RefCell::new(APU::new(audio.clone())));
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
            settings,
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

    pub fn dump_frame(&mut self, path: &str) {
        self.io.borrow().dump_frame(path);
    }
    pub fn run(&mut self) {
        const FRAME_DURATION: Duration =
            Duration::from_nanos((Duration::from_secs(1).as_nanos() / FPS as u128) as u64);

        let run_start = Instant::now();
        let mut frame_start = Instant::now();
        while self.settings.duration == None
            || run_start.elapsed() < self.settings.duration.unwrap()
        {
            self.run_single_frame();
            self.io.borrow_mut().present_frame();

            let elapsed_time_since_frame_start = frame_start.elapsed();
            if elapsed_time_since_frame_start < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed_time_since_frame_start);
            }
            frame_start = Instant::now();
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

            self.apu.borrow_mut().process_cpu_cycles(
                cpu_cycles_for_next_instruction as u8,
                !self.settings.test_mode,
            );

            self.cpu.run_next_instruction();
        }
        cpu_cycles_for_next_instruction
    }
}
