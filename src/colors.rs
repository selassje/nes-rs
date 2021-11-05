pub type RgbColor = (u8, u8, u8);

use serde::{Deserialize, Serialize};
pub trait ColorMapper {
    fn map_nes_color(&self, color: u8) -> RgbColor;
}
#[derive(Serialize, Deserialize)]
pub struct DefaultColorMapper {
    #[serde(with = "serde_arrays")]
    color_map: [RgbColor; 64],
}

impl DefaultColorMapper {
    pub fn new() -> Self {
        let map_color = |color| match color {
            0x00 => (124, 124, 124),
            0x01 => (0, 0, 252),
            0x02 => (0, 0, 188),
            0x03 => (68, 40, 188),
            0x04 => (148, 0, 132),
            0x05 => (168, 0, 32),
            0x06 => (168, 16, 0),
            0x07 => (136, 20, 0),
            0x08 => (80, 48, 0),
            0x09 => (0, 120, 0),
            0x0A => (0, 104, 0),
            0x0B => (0, 88, 0),
            0x0C => (0, 64, 88),
            0x0D => (0, 0, 0),
            0x0E => (0, 0, 0),
            0x0F => (0, 0, 0),

            0x10 => (188, 188, 188),
            0x11 => (0, 120, 248),
            0x12 => (0, 88, 248),
            0x13 => (104, 68, 252),
            0x14 => (216, 0, 204),
            0x15 => (180, 30, 123),
            0x16 => (248, 56, 0),
            0x17 => (228, 92, 16),
            0x18 => (172, 124, 0),
            0x19 => (0, 184, 0),
            0x1A => (0, 168, 0),
            0x1B => (0, 168, 68),
            0x1C => (0, 136, 136),
            0x1D => (0, 0, 0),
            0x1E => (0, 0, 0),
            0x1F => (0, 0, 0),

            0x20 => (248, 248, 248),
            0x21 => (60, 188, 252),
            0x22 => (104, 136, 252),
            0x23 => (152, 120, 248),
            0x24 => (248, 120, 248),
            0x25 => (248, 88, 152),
            0x26 => (248, 120, 88),
            0x27 => (252, 160, 68),
            0x28 => (248, 184, 0),
            0x29 => (184, 248, 24),
            0x2A => (88, 216, 84),
            0x2B => (88, 248, 152),
            0x2C => (0, 232, 216),
            0x2D => (120, 120, 120),
            0x2E => (0, 0, 0),
            0x2F => (0, 0, 0),

            0x30 => (252, 252, 252),
            0x31 => (164, 228, 252),
            0x32 => (184, 184, 248),
            0x33 => (216, 184, 248),
            0x34 => (248, 184, 248),
            0x35 => (248, 164, 192),
            0x36 => (240, 208, 176),
            0x37 => (252, 224, 168),
            0x38 => (248, 216, 120),
            0x39 => (216, 248, 120),
            0x3A => (184, 248, 184),
            0x3B => (184, 248, 216),
            0x3C => (0, 252, 252),
            0x3D => (248, 216, 248),
            0x3E => (0, 0, 0),
            0x3F => (0, 0, 0),

            _ => panic!("This shouldn't happen"),
        };
        let mut color_map = [(0, 0, 0); 64];
        for (i, color) in color_map.iter_mut().enumerate() {
            *color = map_color(i);
        }

        DefaultColorMapper { color_map }
    }
}
impl ColorMapper for DefaultColorMapper {
    fn map_nes_color(&self, color: u8) -> RgbColor {
        self.color_map[color as usize]
    }
}
