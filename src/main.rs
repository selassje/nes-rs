use std::thread;
use std::fs::{File};
use std::env;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver, channel};
use crate::ppu::*;
use std::cell::RefCell;
use std::rc::Rc;

mod memory;
mod cpu_ppu;
mod cpu_controllers;
mod cpu;
mod ppu;
mod apu;
mod cpu_ram_apu;
mod controllers;
mod screen;
mod common;
mod io_sdl;
mod keyboard;
mod audio;
mod nes_format_reader;
mod mapper;
mod vram;
mod cpu_ram;
mod colors;
mod nes;

fn read_rom(file_name: &String) -> nes_format_reader::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_format_reader::NesFile::new(&rom)
}


fn cpu_thread(nes_file : &nes_format_reader::NesFile )
{
   let mut nes = nes::Nes::new(nes_file);

   let mapper = nes_file.create_mapper();
   let controller_1 = keyboard::KeyboardController::get_default_keyboard_controller_player1();
   let controller_2 = keyboard::KeyboardController::get_default_keyboard_controller_player2();

   let controllers = Rc::new(RefCell::new(controllers::Controllers::new(Box::new(controller_1), Box::new(controller_2))));
                                                
   let ppu = RefCell::new(PPU::new(mapper.get_chr_rom().to_vec(),nes_file.get_mirroring()));
   let apu = Rc::new(RefCell::new(apu::APU::new()));
   let mut cpu = cpu::CPU::new(mapper, &ppu, apu, controllers);

   //cpu.run2();
   nes.run();

}

fn main() {

   let u1 : i8 = (80u16 + 80u16) as i8;
   
   println!("{}", u1);

   let args: Vec<String> = env::args().collect();
   let filename = &args[1];
   let rom = read_rom(filename);
   let rom2 = read_rom(filename);
   let nes = nes::Nes::new(&rom);

   let mut io =                         io_sdl::IOSdl::new(filename.clone(),None);
  
   
   thread::spawn(move || {
      cpu_thread(&rom);
   });
unsafe {
   io.run(&rom2);
}
}