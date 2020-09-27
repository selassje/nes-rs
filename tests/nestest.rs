mod common;
use std::time::Duration;
use common::delete_frame;
use common::frames_are_the_same;
use nes_rs::{controllers::Button, nes_test::NesTest};

const PATH: &str = "tests\\nestest\\";

#[test]
fn nestest_official() {
    let rom_path = PATH.to_owned() + "nestest.nes";
    let path_1 = rom_path.to_owned() + "_official.bmp";
    let path_2 = rom_path.to_owned() + "_official.expected.bmp";
    delete_frame(&path_1);
    let mut nes_test = NesTest::new(&rom_path);
    nes_test.run_for(Duration::from_secs(1));
    nes_test.io_test.borrow_mut().press_button_player_1(Button::Start);
    nes_test.run_for(Duration::from_secs(3));
    nes_test.dump_frame(&path_1);
    assert!(frames_are_the_same(&path_1,&path_2));
}

#[test]
fn nestest_unofficial() {
    let rom_path = PATH.to_owned() + "nestest.nes";
    let path_1 = rom_path.to_owned() + "_unofficial.bmp";
    let path_2 = rom_path.to_owned() + "_unofficial.expected.bmp";
    delete_frame(&path_1);
    let mut nes_test = NesTest::new(&rom_path);
    nes_test.io_test.borrow_mut().press_button_player_1(Button::Select);
    nes_test.run_for(Duration::from_secs(1));
    nes_test.io_test.borrow_mut().release_button_player_1(Button::Select);
    nes_test.io_test.borrow_mut().press_button_player_1(Button::Start);
    nes_test.run_for(Duration::from_secs(3));
    nes_test.dump_frame(&path_1);
    assert!(frames_are_the_same(&path_1,&path_2));
}
