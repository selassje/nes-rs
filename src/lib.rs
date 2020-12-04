use std::{cell::RefCell, env, fs::File, io::Read, rc::Rc};

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

fn run_rom(path: &str) {
    let nes_file = read_rom(path);
    let io = Rc::new(RefCell::new(
        io::io_sdl2_imgui_opengl::IOSdl2ImGuiOpenGl::new(path),
    ));
    let controller_1 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player1(io.clone()));
    let controller_2 =
        Rc::new(keyboard::KeyboardController::get_default_keyboard_controller_player2(io.clone()));
    let mut nes = nes::Nes::new(io, &nes_file, controller_1, controller_2);
    nes.power_cycle();
    nes.run(None);
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    run_rom(filename);
}
