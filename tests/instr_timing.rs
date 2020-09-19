mod common;
use std::time::Duration;

use common::run_simple_test;

const PATH: &str = "tests\\instr_timing\\";

#[test]
fn instr_timing() {
    run_simple_test(PATH, "1-instr_timing.nes",Duration::from_secs(20));
}

#[test]
fn branch_timing() {
    run_simple_test(PATH, "2-branch_timing.nes",Duration::from_secs(1));
}