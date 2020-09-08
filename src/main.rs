use std::thread;
use std::fs::{File};
use std::env;
use std::io::prelude::*;

mod memory;
mod ram_ppu;
mod ram_controllers;
mod cpu;
mod ppu;
mod apu;
mod ram_apu;
mod controllers;
mod screen;
mod common;
mod io_sdl;
mod keyboard;
mod nes_format_reader;
mod mapper;
mod vram;
mod ram;
mod colors;
mod nes;

fn read_rom(file_name: &String) -> nes_format_reader::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_format_reader::NesFile::new(&rom)
}


fn nes_thread(nes_file : &nes_format_reader::NesFile)
{
   let mut nes = nes::Nes::new();
   nes.load(nes_file);
   nes.run();
}

fn main() {
   let args: Vec<String> = env::args().collect();
   let filename = &args[1];
   let rom = read_rom(filename);

   let io = io_sdl::IOSdl::new(filename.clone());

   thread::spawn(move || {
      nes_thread(&rom);
   });
   io.run();
}