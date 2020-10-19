use std::ops::Range;

pub const PRG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHR_ROM_UNIT_SIZE: usize = 0x2000;
pub const PRG_RAM_UNIT_SIZE: usize = 0x2000;

pub const CPU_CYCLES_PER_FRAME: usize = 29780;
pub const FPS: usize = 60;

#[derive(Copy, Clone, Debug)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
}

pub fn convert_2u8_to_u16(b0: u8, b1: u8) -> u16 {
    (b0 as u16) | ((b1 as u16) << 8)
}

pub fn get_mirrors(addr: u16, mirror_size: u16, mirrors_range: Range<u16>) -> Vec<u16> {
    let mut mirrors = Vec::new();
    let mirrors_count = (mirrors_range.end - mirrors_range.start + 1) / mirror_size;
    let offset = addr % mirror_size;
    for i in 0..mirrors_count {
        mirrors.push(mirrors_range.start + i * mirror_size + offset);
    }
    mirrors
}
