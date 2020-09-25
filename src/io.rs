pub type SampleFormat = u8;
pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;

pub type RgbColor = (u8, u8, u8);

pub enum KeyCode {
    Q,
    E,
    C,
    Space,
    W,
    S,
    A,
    D,
}

pub trait AudioAccess {
    fn add_sample(&mut self, sample: SampleFormat);
}

pub trait VideoAccess {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor);
}

pub trait IO {
    fn present_frame(&mut self);
}

pub trait KeyboardAccess {
    fn is_key_pressed(&mut self, key: KeyCode) -> bool;
}
