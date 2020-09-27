use nes_rs::nes_test::NesTest;
use std::{
    fs::{self, File},
    io::Read,
    time::Duration,
};

pub fn frames_are_the_same(path_1: &str, path_2: &str) -> bool {
    let mut file_1 = File::open(path_1.clone()).expect(&format!("Unable to open {}", path_1));
    let mut file_2 = File::open(path_2.clone()).expect(&format!("Unable to open {}", path_2));
    let mut buffer_1 = Vec::new();
    let mut buffer_2 = Vec::new();
    let _ = file_1.read_to_end(&mut buffer_1);
    let _ = file_2.read_to_end(&mut buffer_2);
    buffer_1 == buffer_2
}



pub fn delete_frame(rom_path: &str) {
    let path = rom_path.to_owned() + ".bmp";
    fs::remove_file(path.clone()).unwrap_or_default();
}

pub fn run_simple_test(test_dir: &str, rom_name: &str, duration: Duration) {
    let rom_path = test_dir.to_owned() + rom_name;
    let path_1 = rom_path.to_owned() + ".bmp";
    let path_2 = rom_path.to_owned() + ".expected.bmp";
    delete_frame(&path_1);
    let mut nes_test = NesTest::new(&rom_path);
    nes_test.run_for(duration);
    nes_test.dump_frame(&path_1);
    assert!(frames_are_the_same(&path_1,&path_2));
}

#[allow(dead_code)]
pub fn run_simple_short_test(test_dir: &str, rom_name: &str) {
    run_simple_test(test_dir, rom_name, Duration::from_secs(5));
}
