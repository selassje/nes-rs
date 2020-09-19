mod common;
use common::run_simple_short_test;

const PATH: &str = "tests\\blargg_ppu_tests\\";

#[test]
fn palette_ram() {
    run_simple_short_test(PATH, "palette_ram.nes");
}

#[test]
fn sprite_ram() {
    run_simple_short_test(PATH, "sprite_ram.nes");
}

#[test]
fn vram_access() {
    run_simple_short_test(PATH, "vram_access.nes");
}
#[test]
#[ignore]
fn vbl_clear_time() {
    run_simple_short_test(PATH, "vbl_clear_time.nes");
}





