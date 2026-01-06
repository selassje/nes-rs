use std::fmt::Display;

use crate::nes::EmulationFrame;
use crate::nes::ControllerAccess;
use crate::nes::StdNesControllerButton;
use crate::nes::MouseClick;
use crate::nes::ControllerType;
use crate::nes::ControllerId;

pub mod io_sdl2_imgui_opengl;
pub mod io_test;

pub type AudioSampleFormat = f32;
pub type RgbColor = (u8, u8, u8);


impl From<u8> for StdNesControllerButton {
    fn from(value: u8) -> Self {
        use StdNesControllerButton::*;
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

impl Display for StdNesControllerButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StdNesControllerButton::A => "A",
            StdNesControllerButton::B => "B",
            StdNesControllerButton::Select => "Select",
            StdNesControllerButton::Start => "Start",
            StdNesControllerButton::Up => "Up",
            StdNesControllerButton::Down => "Down",
            StdNesControllerButton::Left => "Left",
            StdNesControllerButton::Right => "Right",
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
    pub switch_controller_type: [Option<ControllerType>; 2],
    pub audio_volume: f32,
}

#[derive(Clone, Default)]
pub struct IOControl {
    pub target_fps: u16,
    pub current_fps: u16,
    pub title: Option<String>,
    pub controller_type: [ControllerType; 2],
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
        _controller_id: ControllerId,
        _button: StdNesControllerButton,
    ) -> bool {
        todo!()
    }
    fn get_mouse_click(&self) -> MouseClick {
        todo!()
    }
    fn get_current_frame(&self) -> u128 {
        todo!()
    }
}

pub trait IO: ControllerAccess {
    fn present_frame(&mut self, control: IOControl, emulation_frame: &EmulationFrame) -> IOState;
    fn is_audio_available(&self) -> bool;
}
