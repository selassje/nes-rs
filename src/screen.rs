pub const DISPLAY_WIDTH: usize  = 256;
pub const DISPLAY_HEIGHT: usize = 240;

//pub const DISPLAY_WIDTH: usize  = 128;
//pub const DISPLAY_HEIGHT: usize = 128;

pub type RGB = (u8,u8,u8);

pub type Screen = [[RGB; DISPLAY_HEIGHT]; DISPLAY_WIDTH];

pub trait ScreenController
{
    fn clear(&self);

    fn draw(&self, screen: Screen);
}
