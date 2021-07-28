use std::{cell::RefCell, env, fs::File, io::Read, rc::Rc};

use io::{VideoSizeControl, IO};

mod apu;
mod colors;
mod common;
mod controllers;
mod cpu;
mod cpu_apu;
mod cpu_ppu;
mod io;
mod keyboard;
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

pub fn run() {
    let io = Rc::new(RefCell::new(
        io::io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl::new(),
    ));
    let controller_1 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player1(io.clone()));
    let controller_2 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player2(io.clone()));

    let mut nes = nes::Nes::new(io.clone(), controller_1.clone(), controller_2.clone());
    let mut initial_title: Option<String> = None;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        load(&mut nes, &path);
        initial_title = Some(path.clone());
    };

    let frame_duration: std::time::Duration = std::time::Duration::from_nanos(
        (std::time::Duration::from_secs(1).as_nanos() / (common::DEFAULT_FPS) as u128) as u64,
    );

    let mut io_state: io::IOState = Default::default();

    let mut frame_start = std::time::Instant::now();

    let mut fps = 0;
    let mut one_second_timer = std::time::Instant::now();

    let mut io_control = io::IOControl {
        target_fps: common::DEFAULT_FPS as u16,
        current_fps: 0,
        title: initial_title,
        common: io::IOCommon {
            pause: false,
            audio_enabled: true,
            choose_nes_file: false,
            controllers_setup: false,
            volume: 100,
            video_size: VideoSizeControl::Double,
            controller_configs: Default::default(),
        },
    };

    let is_audio_available = io.borrow().is_audio_available();

    while !io_state.quit {
        if !io_state.common.pause {
            nes.run_single_frame();
            if one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
                fps += 1;
            } else {
                one_second_timer = std::time::Instant::now();
                io_control.current_fps = fps;
                fps = 1;
            }
        }
        io_state = io.borrow_mut().present_frame(io_control.clone());

        handle_io_state(&mut nes, &io_state, &mut io_control);

        if !io_state.common.pause {
            let elapsed_time_since_frame_start = frame_start.elapsed();
            if !is_audio_available {
                if elapsed_time_since_frame_start < frame_duration {
                    std::thread::sleep(frame_duration - elapsed_time_since_frame_start);
                }
            }
            frame_start = std::time::Instant::now();
        }
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
