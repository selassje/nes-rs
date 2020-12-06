use std::{cell::RefCell, env, fs::File, io::Read, rc::Rc};

use io::IO;

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

fn read_rom(file_name: &str) -> nes_file::NesFile {
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
        io::io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl::new("nes-rs"),
    ));
    let controller_1 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player1(io.clone()));
    let controller_2 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player2(io.clone()));

    let mut nes = nes::Nes::new(io.clone(), controller_1.clone(), controller_2.clone());

    let mut load_nes_file = |path| {
        let nes_file = read_rom(path);
        nes.load(&nes_file);
    };

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        load_nes_file(path);
    };

    let mut frame_duration: std::time::Duration = std::time::Duration::from_nanos(
        (std::time::Duration::from_secs(1).as_nanos() / (common::FPS) as u128) as u64,
    );

    let mut frame_duration_adjustment: i32 = 0;
    let mut io_state: io::IOState = Default::default();

    let mut frame_start = std::time::Instant::now();

    let mut fps = 0;
    let mut one_second_timer = std::time::Instant::now();

    let mut io_control = io::IOControl {
        fps: common::FPS as u16,
    };

    while !io_state.quit {
        nes.run_single_frame();

        if one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
            fps += 1;
        } else {
            one_second_timer = std::time::Instant::now();
            if fps != common::FPS {
                frame_duration_adjustment += common::FPS as i32 - fps as i32;
                frame_duration = std::time::Duration::from_nanos(
                    (std::time::Duration::from_secs(1).as_nanos()
                        / ((common::FPS as i32 + frame_duration_adjustment) as u128))
                        as u64,
                );
            }
            io_control.fps = fps as u16;
            fps = 1;
        }

        io_state = io.borrow_mut().present_frame(io_control);
        if io_state.power_cycle {
            nes.power_cycle();
        }

        if io_state.load_rom.is_some() {
            let s = io_state.load_rom.clone().unwrap();
            let x = s.as_str();
            let nes_file = read_rom(x);
            nes.load(&nes_file);
        }

        let elapsed_time_since_frame_start = frame_start.elapsed();
        if elapsed_time_since_frame_start < frame_duration {
            std::thread::sleep(frame_duration - elapsed_time_since_frame_start);
        }

        frame_start = std::time::Instant::now();
    }
}
