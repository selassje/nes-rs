#[path = "common.rs"]
mod common;
use common::run_simple_test;
use std::time::Duration;

#[test]
fn instr_timing() {
    run_simple_test(
        "tests/instr_timing/1-instr_timing.nes",
        Duration::from_secs(20),
    );
}

#[test]
fn branch_timing() {
    run_simple_test(
        "tests/instr_timing/2-branch_timing.nes",
        Duration::from_secs(5),
    );
}
