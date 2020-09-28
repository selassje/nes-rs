mod common;
use common::run_simple_short_test;

#[test]
fn palette_ram() {
    run_simple_short_test("tests\\blargg_ppu_tests\\palette_ram.nes");
}

#[test]
fn sprite_ram() {
    run_simple_short_test("tests\\blargg_ppu_tests\\sprite_ram.nes");
}

#[test]
fn vram_access() {
    run_simple_short_test("tests\\blargg_ppu_tests\\vram_access.nes");
}
#[test]
fn vbl_clear_time() {
    run_simple_short_test("tests\\blargg_ppu_tests\\vbl_clear_time.nes");
}





