pub mod io_test;
pub mod nes_test;

use nes_test::NesTest;
use std::time::Duration;

pub fn run_simple_test(rom_path: &str, duration: Duration) {
    let test_fn = move |nes_test: &mut NesTest| {
        nes_test.run_for(duration);
    };

    let mut nes_test = NesTest::new(rom_path, None, test_fn);
    assert!(nes_test.run());
}

#[allow(dead_code)]
pub fn run_simple_short_test(rom_path: &str) {
    run_simple_test(rom_path, Duration::from_secs(3));
}
