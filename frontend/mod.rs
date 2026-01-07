use crate::ControllerCallback;
use crate::ControllerType;
use crate::EmulationFrame;
use crate::StdNesControllerButton;

pub mod io_sdl2_imgui_opengl;

pub type AudioSampleFormat = f32;
#[derive(Clone)]
pub struct MouseClick {
    pub left_button: bool,
    pub right_button: bool,
    pub x: usize,
    pub y: usize,
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
pub struct FrontendState {
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
pub struct FrontendControl {
    pub target_fps: u16,
    pub current_fps: u16,
    pub title: Option<String>,
    pub controller_type: [ControllerType; 2],
}

pub trait Frontend: ControllerCallback {
    fn present_frame(&mut self, control: FrontendControl, emulation_frame: &EmulationFrame) -> FrontendState;
    fn is_audio_available(&self) -> bool;
}