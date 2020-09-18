use std::{fs::File, io::Read};

use nes_rs::run_test_rom;

const PATH : &str = "tests\\branch_timing_tests\\";

fn frames_are_the_same(rom_path : &str) -> bool {
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


#[test]
fn branch_basics() {
    let rom_path = PATH.to_owned() + "1.Branch_Basics.nes";
    run_test_rom(&rom_path);
    assert!(frames_are_the_same(&rom_path));
}

#[test]
fn backward_branch() {
    let rom_path = PATH.to_owned() + "2.Backward_Branch.nes";
    run_test_rom(&rom_path);
    assert!(frames_are_the_same(&rom_path));
}


#[test]
fn forward_branch() {
    let rom_path = PATH.to_owned() + "3.Forward_Branch.nes";
    run_test_rom(&rom_path);
    assert!(frames_are_the_same(&rom_path));
}