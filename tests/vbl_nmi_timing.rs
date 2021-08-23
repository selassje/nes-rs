#[path = "common.rs"]
mod common;
use common::run_simple_short_test;
use common::run_simple_test;
use std::time::Duration;

#[test]
fn frame_basics() {
    run_simple_test(
        "tests/vbl_nmi_timing/1.frame_basics.nes",
        Duration::from_secs(10),
    );
}

#[test]
fn vbl_timing() {
    run_simple_short_test("tests/vbl_nmi_timing/2.vbl_timing.nes");
}

#[test]
fn even_odd_frames() {
    run_simple_short_test("tests/vbl_nmi_timing/3.even_odd_frames.nes");
}
#[test]
fn vbl_clear_timing() {
    run_simple_short_test("tests/vbl_nmi_timing/4.vbl_clear_timing.nes");
}

#[test]
fn nmi_suppression() {
    run_simple_test(
        "tests/vbl_nmi_timing/5.nmi_suppression.nes",
        Duration::from_secs(5),
    );
}

#[test]
fn nmi_disable() {
    run_simple_short_test("tests/vbl_nmi_timing/6.nmi_disable.nes");
}

#[test]
fn nmi_timing() {
    run_simple_short_test("tests/vbl_nmi_timing/7.nmi_timing.nes");
}
