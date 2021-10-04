use std::fmt::Display;

use crate::controllers;

mod io_internal;
pub mod io_sdl2_imgui_opengl;
pub mod io_test;

pub type AudioSampleFormat = f32;
pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub type RgbColor = (u8, u8, u8);
const PIXEL_SIZE: usize = std::mem::size_of::<RgbColor>();

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
        use self::Button::*;
        match value {
            0 => A,
            1 => B,
            2 => Select,
            3 => Start,
            4 => Up,
            5 => Down,
            6 => Left,
            7 => Right,
            _ => panic!("Can't cast {} to Button", value),
        }
    }
}

impl Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Button::A => "A",
            Button::B => "B",
            Button::Select => "Select",
            Button::Start => "Start",
            Button::Up => "Up",
            Button::Down => "Down",
            Button::Left => "Left",
            Button::Right => "Right",
        })
    }
}
#[derive(Clone)]
pub enum Speed {
    Half,
    Normal,
    Double,
    Increase,
    Decrease,
}

#[derive(Clone, Default)]
pub struct IOState {
    pub quit: bool,
    pub power_cycle: bool,
    pub load_nes_file: Option<String>,
    pub speed: Option<Speed>,
    pub pause: bool,
}

#[derive(Clone, Default)]
pub struct IOControl {
    pub target_fps: u16,
    pub current_fps: u16,
    pub title: Option<String>,
}

pub trait AudioAccess {
    fn add_sample(&mut self, sample: AudioSampleFormat);
}

pub trait VideoAccess {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor);
}

pub struct DummyVideoAccessImpl {}

impl DummyVideoAccessImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl VideoAccess for DummyVideoAccessImpl {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        todo!()
    }
}

pub struct DummyAudioAccessImpl {}

impl DummyAudioAccessImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioAccess for DummyAudioAccessImpl {
    fn add_sample(&mut self, sample: AudioSampleFormat) {
        todo!()
    }
}

pub trait ControllerAccess {
    fn is_button_pressed(&self, controller_id: controllers::ControllerId, button: Button) -> bool;
}

pub trait IO: VideoAccess + AudioAccess + ControllerAccess {
    fn present_frame(&mut self, control: IOControl) -> IOState;
    fn is_audio_available(&self) -> bool;
}

pub struct DummyIOImpl {}

impl DummyIOImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl IO for DummyIOImpl {
    fn present_frame(&mut self, control: IOControl) -> IOState {
        todo!()
    }

    fn is_audio_available(&self) -> bool {
        todo!()
    }
}

impl VideoAccess for DummyIOImpl {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        todo!()
    }
}
impl AudioAccess for DummyIOImpl {
    fn add_sample(&mut self, sample: AudioSampleFormat) {
        todo!()
    }
}
impl ControllerAccess for DummyIOImpl {
    fn is_button_pressed(&self, controller_id: controllers::ControllerId, button: Button) -> bool {
        todo!()
    }
}
