mod common;
use common::run_simple_short_test;

#[test]
fn clocking() {
    run_simple_short_test("tests/mmc3_irq_tests/1.Clocking.nes");
}
