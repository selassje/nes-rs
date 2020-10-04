mod common;
use common::run_simple_short_test;

#[test]
fn len_ctr() {
    run_simple_short_test("tests\\apu_tests\\1-len_ctr.nes");
}

#[test]
fn len_table() {
    run_simple_short_test("tests\\apu_tests\\2-len_table.nes");
}
#[test]
fn irq_flag() {
    run_simple_short_test("tests\\apu_tests\\3-irq_flag.nes");
}
#[test]
fn jitter() {
    run_simple_short_test("tests\\apu_tests\\4-jitter.nes");
}

#[test]
fn irq_flag_timing() {
    run_simple_short_test("tests\\apu_tests\\6-irq_flag_timing.nes");
}
