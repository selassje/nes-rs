mod common;
use common::run_simple_short_test;

#[test]
fn clocking() {
    run_simple_short_test("tests/mmc3_irq_tests/1.Clocking.nes");
}

#[test]
fn details() {
    run_simple_short_test("tests/mmc3_irq_tests/2.Details.nes");
}
#[test]
fn a12_clocking() {
    run_simple_short_test("tests/mmc3_irq_tests/3.A12_clocking.nes");
}

#[test]
fn scanline_timing() {
    run_simple_short_test("tests/mmc3_irq_tests/4.Scanline_timing.nes");
}
#[ignore]
#[test]
fn mmc3_rev_a() {
    run_simple_short_test("tests/mmc3_irq_tests/5.MMC3_rev_A.nes");
}
#[test]
fn mmc3_rev_b() {
    run_simple_short_test("tests/mmc3_irq_tests/6.MMC3_rev_B.nes");
}
