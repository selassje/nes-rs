use std::thread;
use std::fs::{File};
use std::env;
use std::io::prelude::*;

mod display;
mod memory;
mod cpu;

fn read_rom(file_name: &String) -> Vec<u8> {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    rom
}


fn cpu_thread(rom : &Vec<u8>)
{
   let ins_count = rom.len() as u16;
   let mut ram = memory::RAM::new();
   ram.store_bytes(0x200, rom);

   let mut cpu = cpu::CPU::new(&ram);

   cpu.run(ins_count);

   println!("{}", rom[0]);
}

fn main() {

   let args: Vec<String> = env::args().collect();
   let filename = &args[1];
   let rom = read_rom(filename);

   let mut display = display::create_display();
   let sink = display.cb_sink().clone();
   thread::spawn(move || {
        cpu_thread(&rom);
        //sink.send(Box::new(|s| display::update_screen_view(s,15, 20, true))).unwrap();
   });
  display.run();
}