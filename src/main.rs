use std::thread;
use std::fs::{File};
use std::env;
use std::io::prelude::*;


mod display;
mod memory;
mod cpu;
mod screen;
mod cursive_screen;
mod utils;

fn read_rom(file_name: &String) -> Vec<u8> {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    rom
}

fn load_sprites(ram: & mut memory::RAM) {
    let sprites : [Vec<u8>;16] = [vec!(0xF0, 0x90, 0x90, 0x90, 0xF0), // 0
                                  vec!(0x20, 0x60, 0x20, 0x20, 0x70), // 1
                                  vec!(0xF0, 0x10, 0xF0, 0x80, 0xF0), // 2
                                  vec!(0xF0, 0x10, 0xF0, 0x10, 0xF0), // 3
                                  vec!(0x90, 0x90, 0xF0, 0x10, 0x10), // 4
                                  vec!(0xF0, 0x80, 0xF0, 0x10, 0xF0), // 5
                                  vec!(0xF0, 0x80, 0xF0, 0x90, 0xF0), // 6
                                  vec!(0xF0, 0x10, 0x20, 0x40, 0x40), // 7
                                  vec!(0xF0, 0x90, 0xF0, 0x90, 0xF0), // 8
                                  vec!(0xF0, 0x90, 0xF0, 0x10, 0xF0), // 9
                                  vec!(0xF0, 0x90, 0xF0, 0x90, 0x90), // A
                                  vec!(0xE0, 0x90, 0xE0, 0x90, 0xE0), // B 
                                  vec!(0xF0, 0x80, 0x80, 0x80, 0xF0), // C 
                                  vec!(0xE0, 0x90, 0x90, 0x90, 0xE0), // D
                                  vec!(0xF0, 0x80, 0xF0, 0x80, 0xF0), // E 
                                  vec!(0xF0, 0x80, 0xF0, 0x80, 0x80), // F 
    ];

    for (i, sprite) in sprites.iter().enumerate() {
        println!("Loading  sprite {}",i);
        ram.store_bytes( (5*i) as u16, sprite);
    }

}


fn cpu_thread(rom : &Vec<u8>, screen: & dyn screen::ScreenController)
{
   let ins_count = rom.len() as u16;
   let mut ram = memory::RAM::new();
   ram.store_bytes(0x200, rom);
   load_sprites(& mut ram);

   let mut cpu = cpu::CPU::new(&mut ram, screen);

   cpu.run(ins_count);

   println!("{}", rom[0]);
}

fn main() {

   let args: Vec<String> = env::args().collect();
   let filename = &args[1];
   let rom = read_rom(filename);

   let mut display = display::create_display();
   let screen_controller = cursive_screen::CursiveScreenController::new(display.cb_sink().clone());
   thread::spawn(move || {
        cpu_thread(&rom, &screen_controller);
   });
  display.run();
}