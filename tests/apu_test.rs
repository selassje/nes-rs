mod common;
use common::run_simple_short_test;

#[test]
fn len_ctr() {
    run_simple_short_test("tests\\apu_test\\1-len_ctr.nes");
}

#[test]
fn len_table() {
    run_simple_short_test("tests\\apu_test\\2-len_table.nes");
}
#[test]
fn irq_flag() {
    run_simple_short_test("tests\\apu_test\\3-irq_flag.nes");
}
