mod io_internal;
pub mod io_sdl2_imgui_opengl;
pub mod io_test;

pub type AudioSampleFormat = f32;
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

pub enum AudioVolumeControl {
    Increase,
    Decrease,
}
#[derive(Clone, Copy, PartialEq)]
pub enum VideoSizeControl {
    Normal,
    Double,
    Triple,
    Quadrupal,
    FullScreen,
}

impl Default for VideoSizeControl {
    fn default() -> Self {
        Self::Double
    }
}

#[derive(Copy, Clone, Default)]
pub struct IOCommon {
    pub pause: bool,
    pub audio_enabled: bool,
    pub choose_nes_file: bool,
    pub volume: u8,
    pub video_size: VideoSizeControl,
}
#[derive(Default)]
pub struct IOState {
    pub common: IOCommon,
    pub quit: bool,
    pub power_cycle: bool,
    pub load_nes_file: Option<String>,
    pub speed: Option<Speed>,
    pub audio_volume_control: Option<AudioVolumeControl>,
}

#[derive(Copy, Clone, Default)]
pub struct IOControl {
    pub common: IOCommon,
    pub target_fps: u16,
    pub current_fps: u16,
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
    fn add_sample(&mut self, sample: AudioSampleFormat);
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
