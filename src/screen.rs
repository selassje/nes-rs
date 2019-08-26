pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub type Screen = [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH];

pub trait ScreenController
{
    fn clear(&self);

    fn draw(&self, screen: Screen);
}
