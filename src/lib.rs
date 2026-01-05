use std::{
    cell::RefCell,
    env,
    fs::File,
    io::{Read, Write},
    rc::Rc,
};

use emscripten_main_loop::MainLoop;
use io::{io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl, IOControl, IOState, IO};
use nes::Nes;

mod apu;
mod colors;
mod common;
mod controllers;
mod cpu;
mod io;
mod mappers;
mod memory;
pub mod nes;
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

const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(
    (std::time::Duration::from_secs(1).as_nanos() / (common::DEFAULT_FPS) as u128) as u64,
);

#[cfg(target_os = "emscripten")]
extern "C" {
    fn emscripten_run_script(s: *const std::os::raw::c_char);
}
pub struct Emulation {
    nes: Nes,
    io: Rc<RefCell<IOSdl2ImGuiOpenGl>>,
    io_control: IOControl,
    io_state: IOState,
    fps: u16,
    one_second_timer: std::time::Instant,
    frame_start: std::time::Instant,
    is_audio_available: bool,
}
#[allow(clippy::new_without_default)]
impl Emulation {
    pub fn new() -> Self {
        let io = Rc::new(RefCell::new(
            io::io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl::new(),
        ));

        let mut nes: Nes = nes::Nes::new(io.clone());
        let mut initial_title: Option<String> = None;
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let path = &args[1];
            load(&mut nes, path);
            initial_title = Some(path.clone());
        } else {
            load_demo(&mut nes);
        }

        let io_state: io::IOState = Default::default();
        let frame_start = std::time::Instant::now();
        let fps = 0;
        let one_second_timer = std::time::Instant::now();

        let io_control = io::IOControl {
            target_fps: common::DEFAULT_FPS,
            current_fps: 0,
            title: initial_title,
            controller_type: [controllers::ControllerType::NullController; 2],
        };
        let is_audio_available = io.borrow().is_audio_available();
        Self {
            nes,
            io,
            io_control,
            io_state,
            fps,
            one_second_timer,
            frame_start,
            is_audio_available,
        }
    }
}

impl emscripten_main_loop::MainLoop for Emulation {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        if !self.io_state.pause {
            self.nes.run_single_frame();
            if self.one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
                self.fps += 1;
            } else {
                self.one_second_timer = std::time::Instant::now();
                self.io_control.current_fps = self.fps;
                self.fps = 1;
            }
        }
        self.io_control.controller_type = [
            self.nes
                .get_controller_type(controllers::ControllerId::Controller1),
            self.nes
                .get_controller_type(controllers::ControllerId::Controller2),
        ];

        self.io_state = self
            .io
            .borrow_mut()
            .present_frame(self.io_control.clone(), self.nes.get_emulation_frame());

        handle_io_state(&mut self.nes, &self.io_state, &mut self.io_control);

        if !self.io_state.pause {
            let elapsed_time_since_frame_start = self.frame_start.elapsed();
            if !self.is_audio_available && elapsed_time_since_frame_start < FRAME_DURATION {
          //  if elapsed_time_since_frame_start < FRAME_DURATION {
                #[cfg(not(target_os = "emscripten"))]
                std::thread::sleep(FRAME_DURATION - elapsed_time_since_frame_start);
            }
            self.frame_start = std::time::Instant::now();
        } else {
            #[cfg(not(target_os = "emscripten"))]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn read_nes_file(file_name: &str) -> nes_file::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(file_name).unwrap_or_else(|_| {
        panic!(
            "Unable to open ROM {} current dir {}",
            file_name,
            std::env::current_dir().unwrap().display()
        )
    });
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_file::NesFile::new(&rom)
}

fn read_demo() -> nes_file::NesFile {
    let mut demo_rom = include_bytes!("../res/nes-rs-demo.nes").to_vec();
    let pattern = b"xxxxxx";
    const GIT_HASH: &str = git_version::git_version!();
    let replacement = &GIT_HASH.as_bytes()[..6];
    if let Some(pos) = demo_rom
        .windows(pattern.len())
        .position(|window| window == pattern)
    {
        demo_rom[pos..pos + 6].copy_from_slice(replacement);
    } else {
        println!("Pattern not found!");
    }
    nes_file::NesFile::new(&demo_rom)
}

pub fn run(mut emulation: Emulation) {
    while !emulation.io_state.quit {
        emulation.main_loop();
    }
}

fn handle_io_state(nes: &mut nes::Nes, io_state: &io::IOState, io_control: &mut io::IOControl) {
    if io_state.power_cycle {
        nes.power_cycle();
    }

    if let Some(ref nes_file_path) = io_state.load_nes_file {
        load(nes, nes_file_path.as_str());
        io_control.title = Some(nes_file_path.clone());
    }

    if let Some(ref save_state_path) = io_state.save_state {
        let serialized = nes.serialize();
        let file_name = save_state_path.as_str();
        let mut file = File::create(file_name).unwrap_or_else(|_| {
            panic!(
                "Unable to create save file {} current dir {}",
                file_name,
                std::env::current_dir().unwrap().display()
            )
        });
        file.write_all(serialized.as_slice()).unwrap();
        #[cfg(target_os = "emscripten")]
        unsafe {
            let script = std::ffi::CString::new("refreshDownloadList();").unwrap();
            emscripten_run_script(script.as_ptr());
        };
    }

    if let Some(ref load_state_path) = io_state.load_state {
        let file_name = load_state_path.as_str();
        let save = std::fs::read(file_name).unwrap_or_else(|_| {
            panic!(
                "Unable to open save file {} current dir {}",
                file_name,
                std::env::current_dir().unwrap().display()
            )
        });
        nes.deserialize(save);
        io_control.title = Some(load_state_path.clone());
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
        nes.config().set_target_fps(io_control.target_fps);
    }

    for (i, controller_type) in io_state.switch_controller_type.iter().enumerate() {
        if let Some(controller_type) = controller_type {
            if let Some(id) = controllers::ControllerId::from_index(i) {
                nes.set_controller(id, *controller_type);
            }
        }
    }
    nes.config().set_audio_volume(io_state.audio_volume);
}

fn load(nes: &mut nes::Nes, path: &str) {
    let nes_file = read_nes_file(path);
    nes.load(&nes_file);
}

fn load_demo(nes: &mut nes::Nes) {
    let nes_file = read_demo();
    nes.load(&nes_file);
}
