use std::{cell::RefCell, env, fs::File, io::Read, rc::Rc};

mod apu;
mod colors;
mod common;
pub mod controllers;
mod cpu;
mod cpu_ppu;
mod io;
mod keyboard;
mod mapper;
mod memory;
mod nes;
mod nes_format_reader;
mod ppu;
mod ram;
mod ram_apu;
mod ram_controllers;
mod ram_ppu;
mod vram;

pub mod nes_test;

fn read_rom(file_name: &str) -> nes_format_reader::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_format_reader::NesFile::new(&rom)
}

fn run_rom(path: &str) {
    let nes_file = read_rom(path);
    let io = Rc::new(RefCell::new(io::io_sdl2::IOSdl2::new(path)));
    let mut nes = nes::Nes::new(io);
    nes.load(&nes_file);
    nes.run(None);
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    run_rom(filename);
}
