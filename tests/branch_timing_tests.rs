mod common;
use common::run_simple_short_test;

const PATH: &str = "tests\\branch_timing_tests\\";

#[test]
fn branch_basics() {
    run_simple_short_test(PATH, "1.Branch_Basics.nes");
}

#[test]
fn backward_branch() {
    run_simple_short_test(PATH, "2.Backward_Branch.nes");
}

#[test]
fn forward_branch() {
    run_simple_short_test(PATH, "3.Forward_Branch.nes");
}
