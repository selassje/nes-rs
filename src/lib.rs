use std::{cell::RefCell, env, fs::File, io::Read, rc::Rc};

use emscripten_main_loop::MainLoop;
use io::{
    io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl, ControllerConfig, IOControl, IOState,
    VideoSizeControl, IO,
};
use nes::Nes;

mod apu;
mod colors;
mod common;
mod controllers;
mod cpu;
mod io;
mod mappers;
mod memory;
mod nes;
mod nes_file;
mod ppu;
mod ram;
mod ram_apu;
mod ram_controllers;
mod ram_ppu;
mod vram;

pub mod nes_test;

extern crate enum_tryfrom;

#[macro_use]
extern crate enum_tryfrom_derive;
extern crate cfg_if;

#[cfg(not(target_os = "emscripten"))]
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(
    (std::time::Duration::from_secs(1).as_nanos() / (common::DEFAULT_FPS) as u128) as u64,
);
pub struct Emulation {
    nes: Nes,
    io: Rc<RefCell<IOSdl2ImGuiOpenGl>>,
    io_control: IOControl,
    io_state: IOState,
    fps: u16,
    one_second_timer: std::time::Instant,
    frame_start: std::time::Instant,
    #[cfg(not(target_os = "emscripten"))]
    is_audio_available: bool,
}

impl Emulation {
    pub fn new() -> Self {
        let io = Rc::new(RefCell::new(
            io::io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl::new(),
        ));

        let mut nes = nes::Nes::new(io.clone());
        let mut initial_title: Option<String> = None;
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let path = &args[1];
            load(&mut nes, &path);
            initial_title = Some(path.clone());
        };

        // #[cfg(target_os = "emscripten")]
        // {
        //     let game = include_bytes!("../games/Balloon Fight (E).nes");
        //     let v: Vec<u8> = game.to_vec();
        //     let nes_file = nes_file::NesFile::new(&v);
        //     nes.load(&nes_file);
        // }

        let io_state: io::IOState = Default::default();
        let frame_start = std::time::Instant::now();
        let fps = 0;
        let one_second_timer = std::time::Instant::now();

        let io_control = io::IOControl {
            target_fps: common::DEFAULT_FPS as u16,
            current_fps: 0,
            title: initial_title,
            common: io::IOCommon {
                pause: false,
                choose_nes_file: false,
                controllers_setup: false,
                volume: 100,
                video_size: VideoSizeControl::Double,
                controller_configs: [ControllerConfig::new(0), ControllerConfig::new(1)],
            },
        };
        #[cfg(not(target_os = "emscripten"))]
        let is_audio_available = io.borrow().is_audio_available();
        Self {
            nes,
            io,
            io_control,
            io_state,
            fps,
            one_second_timer,
            frame_start,
            #[cfg(not(target_os = "emscripten"))]
            is_audio_available,
        }
    }
}

impl emscripten_main_loop::MainLoop for Emulation {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        if !self.io_state.common.pause {
            self.nes.run_single_frame();
            if self.one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
                self.fps += 1;
            } else {
                self.one_second_timer = std::time::Instant::now();
                self.io_control.current_fps = self.fps;
                self.fps = 1;
            }
        }
        self.io_state = self.io.borrow_mut().present_frame(self.io_control.clone());

        handle_io_state(&mut self.nes, &self.io_state, &mut self.io_control);

        if !self.io_state.common.pause {
            #[cfg(not(target_os = "emscripten"))]
            {
                let elapsed_time_since_frame_start = self.frame_start.elapsed();
                if !self.is_audio_available {
                    if elapsed_time_since_frame_start < FRAME_DURATION {
                        std::thread::sleep(FRAME_DURATION - elapsed_time_since_frame_start);
                    }
                }
            }
            self.frame_start = std::time::Instant::now();
        }
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn read_nes_file(file_name: &str) -> nes_file::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect(&format!(
        "Unable to open ROM {} current dir {}",
        file_name,
        std::env::current_dir().unwrap().display()
    ));
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_file::NesFile::new(&rom)
}

pub fn run(mut emulation: Emulation) {
    while !emulation.io_state.quit {
        emulation.main_loop();
    }
}

fn handle_io_state(nes: &mut nes::Nes, io_state: &io::IOState, io_control: &mut io::IOControl) {
    io_control.common = io_state.common;

    if io_state.power_cycle {
        nes.power_cycle();
    }

    if let Some(ref nes_file_path) = io_state.load_nes_file {
        load(nes, nes_file_path.as_str());
        io_control.title = Some(nes_file_path.clone());
    }

    if let Some(ref speed) = io_state.speed {
        match speed {
            io::Speed::Half => io_control.target_fps = common::HALF_FPS,
            io::Speed::Normal => io_control.target_fps = common::DEFAULT_FPS,
            io::Speed::Double => io_control.target_fps = common::DOUBLE_FPS,
            io::Speed::Increase => io_control.target_fps += 5,
            io::Speed::Decrease => {
                io_control.target_fps = std::cmp::max(0, io_control.target_fps as i32 - 5) as u16
            }
        }
    }
    if let Some(ref volume_control) = io_state.audio_volume_control {
        io_control.common.volume = match volume_control {
            io::AudioVolumeControl::Increase => std::cmp::min(100, io_control.common.volume + 5),
            io::AudioVolumeControl::Decrease => {
                std::cmp::max(0, io_control.common.volume as i32 - 5) as u8
            }
        }
    }
}

fn load(nes: &mut nes::Nes, path: &str) {
    let nes_file = read_nes_file(path);
    nes.load(&nes_file);
}
