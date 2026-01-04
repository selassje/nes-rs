use std::fmt::Display;

use crate::controllers;
use crate::nes::EmulationFrame;

mod io_internal;
pub mod io_sdl2_imgui_opengl;
pub mod io_test;

pub type AudioSampleFormat = f32;
pub type RgbColor = (u8, u8, u8);

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
    pub save_state: Option<String>,
    pub load_state: Option<String>,
    pub speed: Option<Speed>,
    pub pause: bool,
    pub switch_controller_type: [Option<controllers::ControllerType>; 2],
}

#[derive(Clone, Default)]
pub struct IOControl {
    pub target_fps: u16,
    pub current_fps: u16,
    pub title: Option<String>,
    pub controller_type: [controllers::ControllerType; 2],
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
    fn set_pixel(&mut self, _x: usize, _y: usize, _color: RgbColor) {
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
    fn add_sample(&mut self, _sample: AudioSampleFormat) {
        todo!()
    }
}

#[derive(Clone)]
pub struct MouseClick {
    pub left_button: bool,
    pub right_button: bool,
    pub x: usize,
    pub y: usize,
}

pub trait ControllerAccess {
    fn is_button_pressed(&self, controller_id: controllers::ControllerId, button: Button) -> bool;
    fn get_mouse_click(&self) -> MouseClick;
    fn get_current_frame(&self) -> u128;
    fn get_luminance(&self, x: usize, y: usize) -> f32;
}

pub struct DummyControllerAccessImplementation {}
impl DummyControllerAccessImplementation {
    pub fn new() -> Self {
        Self {}
    }
}
impl ControllerAccess for DummyControllerAccessImplementation {
    fn is_button_pressed(
        &self,
        _controller_id: controllers::ControllerId,
        _button: Button,
    ) -> bool {
        todo!()
    }
    fn get_mouse_click(&self) -> MouseClick {
        todo!()
    }
    fn get_current_frame(&self) -> u128 {
        todo!()
    }
    fn get_luminance(&self, _x: usize, _y: usize) -> f32 {
        todo!()
    }
}

pub trait IO: VideoAccess + AudioAccess + ControllerAccess {
    fn present_frame(&mut self, control: IOControl, emulation_frame: &EmulationFrame) -> IOState;
    fn is_audio_available(&self) -> bool;
}

pub struct DummyIOImpl {}

impl DummyIOImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl IO for DummyIOImpl {
    fn present_frame(&mut self, _control: IOControl, _emulation_frame: &EmulationFrame) -> IOState {
        todo!()
    }

    fn is_audio_available(&self) -> bool {
        todo!()
    }
}

impl VideoAccess for DummyIOImpl {
    fn set_pixel(&mut self, _x: usize, _y: usize, _color: RgbColor) {
        todo!()
    }
}
impl AudioAccess for DummyIOImpl {
    fn add_sample(&mut self, _sample: AudioSampleFormat) {
        todo!()
    }
}
impl ControllerAccess for DummyIOImpl {
    fn is_button_pressed(
        &self,
        _controller_id: controllers::ControllerId,
        _button: Button,
    ) -> bool {
        todo!()
    }
    fn get_mouse_click(&self) -> MouseClick {
        todo!()
    }
    fn get_current_frame(&self) -> u128 {
        todo!()
    }
    fn get_luminance(&self, _x: usize, _y: usize) -> f32 {
        todo!()
    }
}
