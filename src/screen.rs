pub const DISPLAY_WIDTH: usize  = 256;
pub const DISPLAY_HEIGHT: usize = 240;


pub type RGB = (u8,u8,u8);

pub type Screen = [[RGB; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
