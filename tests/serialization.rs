#[path = "common.rs"]
mod common;
mod nes_test;
use std::time::Duration;
use nes_test::NesTest;

#[test]
fn serialization_test() {
    let rom_path = "tests/nestest/nestest.nes";
    let test_fn = |nes_test: &mut NesTest| {
        nes_test.run_for(Duration::from_secs(1));
        nes_test.press_player_1_start();
        nes_test.run_for(Duration::from_secs(3));
        let serialized = nes_test.serialize_and_reset();
        nes_test.deserialize(serialized);
        nes_test.release_player_1_start();
        nes_test.run_for(Duration::from_secs(1));
    };

    let mut nes_test = NesTest::new(rom_path, Some("official"), test_fn);
    assert!(nes_test.run());
}
