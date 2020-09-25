use std::{env, fs::File, io::Read, thread, time::Duration};

mod apu;
mod colors;
mod common;
mod controllers;
mod cpu;
mod cpu_ppu;
mod io_sdl;
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
mod screen;
mod vram;

#[derive(Copy, Clone, Debug)]
pub struct NesSettings {
    test_mode: bool,
    duration: Option<Duration>,
}

impl Default for NesSettings {
    fn default() -> Self {
        Self {
            test_mode: true,
            duration: None,
        }
    }
}

fn read_rom(file_name: &str) -> nes_format_reader::NesFile {
    let mut rom = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open ROM");
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    nes_format_reader::NesFile::new(&rom)
}

fn nes_thread(nes_file: &nes_format_reader::NesFile, settings: NesSettings) {
    let mut nes = nes::Nes::new(settings);
    nes.load(nes_file);
    nes.run();
}

fn run_rom(path: &str, settings: NesSettings) {
    let rom = read_rom(path);
    let io = io_sdl::IOSdl::new(String::from(path));
    let nes_handle = thread::spawn(move || {
        nes_thread(&rom, settings);
    });
    if !settings.test_mode {
        io.run(settings);
    }
    let _ = nes_handle.join();
    if settings.test_mode {
        io_sdl::IOSdl::dump_frame(&(path.to_owned() + ".bmp"));
    }
}

pub fn run_test_rom(path: &str, duration: Duration) {
    run_rom(
        path,
        NesSettings {
            test_mode: true,
            duration: Some(duration),
        },
    );
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    run_rom(filename, Default::default());
}
