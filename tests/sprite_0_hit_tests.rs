mod common;
use common::run_simple_short_test;

#[test]
fn sprite_0_hit_basics() {
    run_simple_short_test("tests/sprite_0_hit_tests/01.basics.nes");
}
#[test]
fn sprite_0_hit_alignment() {
    run_simple_short_test("tests/sprite_0_hit_tests/02.alignment.nes");
}
#[test]
fn sprite_0_hit_corners() {
    run_simple_short_test("tests/sprite_0_hit_tests/03.corners.nes");
}

#[test]
fn sprite_0_hit_flip() {
    run_simple_short_test("tests/sprite_0_hit_tests/04.flip.nes");
}

#[test]
fn sprite_0_hit_left_clip() {
    run_simple_short_test("tests/sprite_0_hit_tests/05.left_clip.nes");
}
#[test]
fn sprite_0_hit_right_edge() {
    run_simple_short_test("tests/sprite_0_hit_tests/06.right_edge.nes");
}

#[test]
fn sprite_0_hit_screen_bottom() {
    run_simple_short_test("tests/sprite_0_hit_tests/07.screen_bottom.nes");
}

#[test]
fn sprite_0_hit_double_height() {
    run_simple_short_test("tests/sprite_0_hit_tests/08.double_height.nes");
}

#[test]
fn sprite_0_hit_timing_basics() {
    run_simple_short_test("tests/sprite_0_hit_tests/09.timing_basics.nes");
}

#[test]
fn sprite_0_hit_timing_order() {
    run_simple_short_test("tests/sprite_0_hit_tests/10.timing_order.nes");
}
#[test]
fn sprite_0_hit_edge_timing() {
    run_simple_short_test("tests/sprite_0_hit_tests/11.edge_timing.nes");
}
