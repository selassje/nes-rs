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
                    ButtonMapping::new(Down),
                    ButtonMapping::new(Left),
                    ButtonMapping::new(Right),
                ],
                _ => panic!("Wrong player!"),
            },
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct IOCommon {
    pub pause: bool,
    pub choose_nes_file: bool,
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
