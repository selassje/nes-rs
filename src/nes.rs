use common::FPS;

use crate::cpu::CPU;
use crate::io::AudioAccess;
use crate::io::KeyboardAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::nes_file::NesFile;
use crate::ppu::PPU;
use crate::ram::RAM;
use crate::vram::VRAM;
use crate::{apu::APU, controllers::Controller};
use crate::{common, io::IOState};
use crate::{controllers::Controllers, io::IOControl};

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
        let controllers = Rc::new(RefCell::new(Controllers::new(controller_1, controller_2)));

        let vram = Rc::new(RefCell::new(VRAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new(vram.clone(), io.clone())));
        let apu = Rc::new(RefCell::new(APU::new(io.clone())));
        let ram = Rc::new(RefCell::new(RAM::new(
            ppu.clone(),
            controllers.clone(),
            apu.clone(),
        )));
        apu.borrow_mut().set_dmc_memory(ram.clone());
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
        let mut frame_duration: Duration =
            Duration::from_nanos((Duration::from_secs(1).as_nanos() / (FPS) as u128) as u64);

        let mut frame_duration_adjustment: i32 = 0;
        let mut io_state: IOState = Default::default();
        let mut elapsed_frames: u128 = 0;

        let mut frame_start = Instant::now();

        let mut fps = 0;
        let mut one_second_timer = Instant::now();

        let mut io_control = IOControl { fps: 0 };

        while (duration == None
            || elapsed_frames < duration.unwrap().as_secs() as u128 * FPS as u128)
            && !io_state.quit
        {
            self.run_single_frame();
            if duration == None {
                if one_second_timer.elapsed() < Duration::from_secs(1) {
                    fps += 1;
                } else {
                    one_second_timer = Instant::now();
                    if fps != FPS {
                        frame_duration_adjustment += FPS as i32 - fps as i32;
                        frame_duration = Duration::from_nanos(
                            (Duration::from_secs(1).as_nanos()
                                / ((FPS as i32 + frame_duration_adjustment) as u128))
                                as u64,
                        );
                    }
                    io_control.fps = fps as u8;
                    fps = 1;
                }
            }
            io_state = self.io.borrow_mut().present_frame(io_control);
            let elapsed_time_since_frame_start = frame_start.elapsed();
            if duration.is_none() && elapsed_time_since_frame_start < frame_duration {
                std::thread::sleep(frame_duration - elapsed_time_since_frame_start);
            }
            frame_start = Instant::now();
            elapsed_frames += 1;
        }
    }

    fn run_single_frame(&mut self) {
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
