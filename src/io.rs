mod io_internal;
pub mod io_sdl2_imgui_opengl;
pub mod io_test;

pub type SampleFormat = f32;
pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub type RgbColor = (u8, u8, u8);
const PIXEL_SIZE: usize = std::mem::size_of::<RgbColor>();

pub enum Speed {
    Half,
    Normal,
    Double,
    Increase,
    Decrease,
}

#[derive(Default)]
pub struct IOState {
    pub quit: bool,
    pub power_cycle: bool,
    pub load_nes_file: Option<String>,
    pub pause: bool,
    pub speed: Option<Speed>,
}

#[derive(Copy, Clone)]
pub struct IOControl {
    pub target_fps: u16,
    pub current_fps: u16,
    pub pause: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Q,
    E,
    C,
    Space,
    W,
    S,
    A,
    D,
    Kp4,
    Kp5,
    Kp6,
    KpPlus,
    Up,
    Down,
    Left,
    Right,
}

pub trait AudioAccess {
    fn add_sample(&mut self, sample: SampleFormat);
}

pub trait VideoAccess {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor);
}

pub trait IO {
    fn present_frame(&mut self, control: IOControl) -> IOState;
    fn is_audio_available(&self) -> bool;
}

pub trait KeyboardAccess {
    fn is_key_pressed(&self, key: KeyCode) -> bool;
}
