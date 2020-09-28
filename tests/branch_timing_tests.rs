mod common;
use common::run_simple_short_test;

#[test]
fn branch_basics() {
    run_simple_short_test("tests\\branch_timing_tests\\1.Branch_Basics.nes");
}

#[test]
fn backward_branch() {
    run_simple_short_test("tests\\branch_timing_tests\\2.Backward_Branch.nes");
}

#[test]
fn forward_branch() {
    run_simple_short_test("tests\\branch_timing_tests\\3.Forward_Branch.nes");
}
