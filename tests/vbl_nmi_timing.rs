mod common;
use common::run_simple_short_test;
use std::time::Duration;

use common::run_simple_test;

const PATH: &str = "tests\\vbl_nmi_timing\\";

#[test]
fn frame_basics() {
    run_simple_test(PATH, "1.frame_basics.nes", Duration::from_secs(10));
}

#[test]
fn vbl_timing() {
    run_simple_short_test(PATH, "2.vbl_timing.nes");
}

#[test]
fn even_odd_frames() {
    run_simple_short_test(PATH, "3.even_odd_frames.nes");
}
#[test]
fn vbl_clear_timing() {
    run_simple_short_test(PATH, "4.vbl_clear_timing.nes");
}

#[test]
fn nmi_suppression() {
    run_simple_short_test(PATH, "5.nmi_suppression.nes");
}

#[test]
fn nmi_disable() {
    run_simple_short_test(PATH, "6.nmi_disable.nes");
}

#[test]
fn nmi_timing() {
    run_simple_short_test(PATH, "7.nmi_timing.nes");
}