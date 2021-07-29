use crate::controllers;

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
    _Normal = 1,
    Double = 2,
    Triple = 3,
    Quadrupal = 4,
    FullScreen = 5,
}

impl Into<[u32; 2]> for VideoSizeControl {
    fn into(self) -> [u32; 2] {
        if self == VideoSizeControl::FullScreen {
            panic!("VideoSizeControl::FullScreen size can't be converted to [u32;2]")
        }

        let scaling = self as u32;
        [scaling * FRAME_WIDTH as u32, scaling * FRAME_HEIGHT as u32]
    }
}

impl Into<[f32; 2]> for VideoSizeControl {
    fn into(self) -> [f32; 2] {
        let [width, height]: [u32; 2] = self.into();
        [width as _, height as _]
    }
}

impl Default for VideoSizeControl {
    fn default() -> Self {
        Self::Double
    }
}

#[derive(Clone, Copy)]
pub struct ButtonMapping {
    pub waiting_for_input: bool,
    pub key: sdl2::keyboard::Scancode,
}

impl Default for ButtonMapping {
    fn default() -> Self {
        Self {
            waiting_for_input: false,
            key: sdl2::keyboard::Scancode::A,
        }
    }
}

impl ButtonMapping {
    pub fn new(key: sdl2::keyboard::Scancode) -> Self {
        Self {
            waiting_for_input: false,
            key,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ControllerConfig {
    pub use_zapper: bool,
    pub mapping: [ButtonMapping; controllers::Button::Right as usize + 1],
    pub pending_key_select: Option<u8>,
}

impl ControllerConfig {
    pub fn new(player: u8) -> Self {
        use sdl2::keyboard::Scancode::*;
        Self {
            use_zapper: false,
            pending_key_select: None,
            mapping: match player {
                0 => [
                    ButtonMapping::new(Q),
                    ButtonMapping::new(E),
                    ButtonMapping::new(C),
                    ButtonMapping::new(Space),
                    ButtonMapping::new(W),
                    ButtonMapping::new(S),
                    ButtonMapping::new(A),
                    ButtonMapping::new(D),
                ],
                1 => [
                    ButtonMapping::new(Kp4),
                    ButtonMapping::new(Kp5),
                    ButtonMapping::new(Kp6),
                    ButtonMapping::new(KpPlus),
                    ButtonMapping::new(Up),
                    ButtonMapping::new(Left),
                    ButtonMapping::new(Right),
                    ButtonMapping::new(Down),
                ],
                _ => panic!("Wrong player"),
            },
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct IOCommon {
    pub pause: bool,
    pub choose_nes_file: bool,
    pub volume: u8,
    pub video_size: VideoSizeControl,
    pub controllers_setup: bool,
    pub controller_configs: [ControllerConfig; 2],
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

#[derive(Clone, Default)]
pub struct IOControl {
    pub common: IOCommon,
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

pub trait IO {
    fn present_frame(&mut self, control: IOControl) -> IOState;
    fn is_audio_available(&self) -> bool;
}

pub trait ControllerAccess {
    fn is_button_pressed(
        &self,
        controller_id: controllers::ControllerId,
        button: controllers::Button,
    ) -> bool;
}
