#[path = "common.rs"]
mod common;
use nes_rs::nes_test::NesTest;
use std::time::Duration;

const PATH: &str = "tests/nestest/";

#[test]
fn nestest_official() {
    let rom_path = PATH.to_owned() + "nestest.nes";
    let test_fn = |nes_test: &mut NesTest| {
        nes_test.run_for(Duration::from_secs(1));
        nes_test.press_player_1_start();
        nes_test.run_for(Duration::from_secs(3));
    };

    let mut nes_test = NesTest::new(&rom_path, Some("official"), test_fn);
    assert!(nes_test.run());
}

#[test]
fn nestest_unofficial() {
    let rom_path = PATH.to_owned() + "nestest.nes";

    let test_fn = |nes_test: &mut NesTest| {
        nes_test.press_player_1_select();
        nes_test.run_for(Duration::from_secs(1));
        nes_test.release_player_1_select();
        nes_test.press_player_1_start();
        nes_test.run_for(Duration::from_secs(3));
    };

    let mut nes_test = NesTest::new(&rom_path, Some("unofficial"), test_fn);
    assert!(nes_test.run());
}
