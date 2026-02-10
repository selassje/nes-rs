mod common;
use common::run_simple_short_test;

#[test]
fn mmc5_exram() {
    run_simple_short_test("tests/mmc5_tests/mmc5exram.nes");
}

#[test]
fn mmc5_test() {
    run_simple_short_test("tests/mmc5_tests/mmc5test.nes");
}
