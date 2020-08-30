use std::thread;
use std::fs::{File};
use std::env;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver, channel};
use crate::ppu::*;
use std::cell::RefCell;

mod memory;
mod cpu_ppu;
mod cpu;
mod ppu;
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

fn read_rom(file_name: &String) -> nes_format_reader::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_format_reader::NesFile::new(&rom)
}


fn cpu_thread(nes_file : &nes_format_reader::NesFile, screen_tx: Sender<screen::Screen>, keyboard_rx :Receiver::<keyboard::KeyEvent>, audio_tx: Sender::<bool> )
{
   let mut mapper = nes_file.create_mapper();
   let ppu = RefCell::new(PPU::new(mapper.get_chr_rom().to_vec(),nes_file.get_mirroring()));
   let mut cpu = cpu::CPU::new(&mut mapper, &ppu, screen_tx, keyboard_rx, audio_tx);

   cpu.run();
}

fn main() {

   let u1 : i8 = (80u16 + 80u16) as i8;
   
   println!("{}", u1);

   let args: Vec<String> = env::args().collect();
   let filename = &args[1];
   let rom = read_rom(filename);

   let (keyboard_tx, keyboard_rx) : (Sender<keyboard::KeyEvent>, Receiver<keyboard::KeyEvent>) = channel();
   let (screen_tx,   screen_rx)   : (Sender<screen::Screen>, Receiver<screen::Screen>) = channel();
   let (audio_tx,   audio_rx)     : (Sender<bool>, Receiver<bool>) = channel();
   let io =                         io_sdl::IOSdl::new(filename.clone(), screen_rx, keyboard_tx, audio_rx);

   thread::spawn(move || {
        cpu_thread(&rom, screen_tx, keyboard_rx, audio_tx);
   });
   io.run();
}