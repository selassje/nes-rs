use nes_rs::nes_test::NesTest;
use std::time::Duration;

pub fn run_simple_test(test_dir: &str, rom_name: &str, duration: Duration) {
    let rom_path = test_dir.to_owned() + rom_name;

    let test_fn = move |nes_test: &mut NesTest| {
        nes_test.run_for(duration);
    };

    let mut nes_test = NesTest::new(&rom_path, None, test_fn);
    assert!(nes_test.run());
}

#[allow(dead_code)]
pub fn run_simple_short_test(test_dir: &str, rom_name: &str) {
    run_simple_test(test_dir, rom_name, Duration::from_secs(3));
}
