use nes_rs::run_test_rom;
use std::{
    fs::{self, File},
    io::Read,
    time::Duration,
};

fn frames_are_the_same(rom_path: &str) -> bool {
    let path_1 = rom_path.to_owned() + ".bmp";
    let path_2 = rom_path.to_owned() + ".expected.bmp";
    let mut file_1 = File::open(path_1.clone()).expect(&format!("Unable to open {}", path_1));
    let mut file_2 = File::open(path_2.clone()).expect(&format!("Unable to open {}", path_2));
    let mut buffer_1 = Vec::new();
    let mut buffer_2 = Vec::new();
    let _ = file_1.read_to_end(&mut buffer_1);
    let _ = file_2.read_to_end(&mut buffer_2);
    buffer_1 == buffer_2
}

fn delete_frame(rom_path: &str) {
    let path = rom_path.to_owned() + ".bmp";
    fs::remove_file(path.clone()).unwrap_or_default();
}

pub fn run_simple_test(test_dir: &str, rom_name: &str, duration: Duration) {
    let rom_path = test_dir.to_owned() + rom_name;
    delete_frame(&rom_path);
    run_test_rom(&rom_path, duration);
    assert!(frames_are_the_same(&rom_path));
}

#[allow(dead_code)]
pub fn run_simple_short_test(test_dir: &str, rom_name: &str) {
    run_simple_test(test_dir, rom_name, Duration::from_secs(5));
}
