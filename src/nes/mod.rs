mod apu;
mod colors;
mod common;
mod controllers;
mod cpu;
mod mappers;
mod memory;
mod nes_file;
mod ppu;
mod ram;
mod ram_apu;
mod ram_controllers;
mod ram_ppu;
mod vram;

use apu::Apu;
use controllers::Controllers;
use cpu::Cpu;
use mappers::Mapper;
use mappers::MapperEnum;
use mappers::MapperNull;
use nes_file::NesFile;
use ppu::Ppu;
use ppu::PpuState;
use ram::Ram;

use std::time::Duration;

const SERIALIZATION_VER: &str = "1";

#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ControllerId {
    Controller1,
    Controller2,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ControllerType {
    NullController,
    #[default]
    StdNesController,
    Zapper,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum StdNesControllerButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

pub enum ZapperTarget {
    OffScreen,
    OnScreen(u8, u8),
}

pub trait ControllerCallback {
    fn is_button_pressed(&self, id: ControllerId, button: StdNesControllerButton) -> bool;
    fn is_zapper_trigger_pressed(&self, id: ControllerId) -> Option<ZapperTarget>;
}
struct CpuBus<'a> {
    pub ram: &'a mut Ram,
    pub ppu: &'a mut Ppu,
    pub apu: &'a mut Apu,
    pub mapper: &'a mut MapperEnum,
    pub controllers: &'a mut Controllers,
    pub callback: Option<&'a dyn ControllerCallback>,
}

macro_rules! cpu_bus {
    ($nes:expr, $callback:ident) => {{
        CpuBus {
            ram: &mut $nes.ram,
            ppu: &mut $nes.ppu,
            apu: &mut $nes.apu,
            mapper: &mut $nes.mapper,
            controllers: &mut $nes.controllers,
            callback: $callback,
        }
    }};
}

struct PpuBus<'a> {
    pub mapper: &'a mut MapperEnum,
    pub emulation_frame: &'a mut EmulationFrame,
}
struct RamBus<'a> {
    pub apu: &'a mut Apu,
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut MapperEnum,
    pub controllers: &'a mut Controllers,
    pub callback: Option<&'a dyn ControllerCallback>,
}
pub const DEFAULT_FPS: u16 = 60;
pub const PIXEL_SIZE: usize = 3;
pub const VIDEO_FRAME_WIDTH: usize = 256;
pub const VIDEO_FRAME_HEIGHT: usize = 240;
pub const VIDEO_FRAME_SIZE: usize = VIDEO_FRAME_HEIGHT * VIDEO_FRAME_WIDTH * PIXEL_SIZE;
pub const MAX_AUDIO_FRAME_SIZE: usize = 2048;
pub const SAMPLING_RATE: usize = 44100;

pub struct VideoFrame {
    pixels: Box<[u8; VIDEO_FRAME_SIZE]>,
}

impl VideoFrame {
    pub(crate) fn new() -> Self {
        Self {
            pixels: Box::new([0; VIDEO_FRAME_SIZE]),
        }
    }
    pub fn get_pixels(&self) -> &[u8; VIDEO_FRAME_SIZE] {
        &self.pixels
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> (u8, u8, u8) {
        let index = (y as usize * VIDEO_FRAME_WIDTH + x as usize) * PIXEL_SIZE;
        (
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
        )
    }

    pub(crate) fn set_pixel(&mut self, x: u8, y: u8, color: (u8, u8, u8)) {
        let index = (y as usize * VIDEO_FRAME_WIDTH + x as usize) * PIXEL_SIZE;
        self.pixels[index] = color.0;
        self.pixels[index + 1] = color.1;
        self.pixels[index + 2] = color.2;
    }
}

pub struct AudioFrame {
    samples: Box<[f32; MAX_AUDIO_FRAME_SIZE]>,
    size: usize,
}

impl AudioFrame {
    pub(crate) fn new() -> Self {
        Self {
            samples: Box::new([0.0; MAX_AUDIO_FRAME_SIZE]),
            size: 0,
        }
    }
    pub fn get_samples(&self) -> &[f32] {
        &self.samples[..self.size]
    }

    pub fn get_byte_size(&self) -> usize {
        self.size * std::mem::size_of::<f32>()
    }

    pub(crate) fn reset(&mut self) {
        self.size = 0;
    }

    pub(crate) fn add_sample(&mut self, sample: f32) {
        if self.size < MAX_AUDIO_FRAME_SIZE {
            self.samples[self.size] = sample;
            self.size += 1;
        }
    }
}

pub struct EmulationFrame {
    pub video: VideoFrame,
    pub audio: AudioFrame,
}

impl Default for EmulationFrame {
    fn default() -> Self {
        Self {
            video: VideoFrame::new(),
            audio: AudioFrame::new(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct AudioConfig {
    pub audio_volume: f32,
    pub target_fps: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            audio_volume: 1.0,
            target_fps: DEFAULT_FPS,
        }
    }
}

pub struct Config<'a> {
    config: &'a mut AudioConfig,
    controllers: &'a mut Controllers,
}

impl Config<'_> {
    pub fn set_audio_volume(&mut self, volume: f32) {
        self.config.audio_volume = volume;
    }
    pub fn get_audio_volume(&self) -> f32 {
        self.config.audio_volume
    }
    pub fn set_target_fps(&mut self, fps: u16) {
        self.config.target_fps = fps;
    }
    pub fn get_target_fps(&self) -> u16 {
        self.config.target_fps
    }
    pub fn set_controller(&mut self, id: ControllerId, controller_type: ControllerType) {
        self.controllers.set_controller(id, controller_type);
    }

    pub fn get_controller_type(&self, id: ControllerId) -> ControllerType {
        self.controllers.get_controller_type(id)
    }
}

pub(crate) struct ApuBus<'a> {
    pub ram: &'a mut Ram,
    pub mapper: &'a mut MapperEnum,
    pub emulation_frame: &'a mut EmulationFrame,
    pub config: &'a AudioConfig,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Nes {
    version: String,
    cpu: Cpu,
    ram: Ram,
    ppu: Ppu,
    apu: Apu,
    controllers: Controllers,
    mapper: MapperEnum,
    config: AudioConfig,
    #[serde(skip, default)]
    emulation_frame: EmulationFrame,
}

impl Nes {
    pub fn new() -> Self {
        Nes {
            version: SERIALIZATION_VER.to_string(),
            cpu: Cpu::new(),
            ram: Ram::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            controllers: Controllers::new(),
            mapper: MapperEnum::MapperNull(MapperNull::new()),
            config: AudioConfig::default(),
            emulation_frame: EmulationFrame::default(),
        }
    }

    pub fn save_state(&self) -> Vec<u8> {
        let serialized = serde_json::to_vec(self).unwrap();
        let compressed = yazi::compress(
            serialized.as_slice(),
            yazi::Format::Zlib,
            yazi::CompressionLevel::Default,
        )
        .unwrap();
        compressed
    }

    pub fn get_emulation_frame(&self) -> &EmulationFrame {
        &self.emulation_frame
    }

    pub fn config(&mut self) -> Config<'_> {
        Config {
            config: &mut self.config,
            controllers: &mut self.controllers,
        }
    }

    pub fn load_state(&mut self, state: Vec<u8>) {
        let (decompressed, checksum) =
            yazi::decompress(state.as_slice(), yazi::Format::Zlib).unwrap();

        assert_eq!(
            yazi::Adler32::from_buf(&decompressed).finish(),
            checksum.unwrap()
        );

        let mut deserializer = serde_json::Deserializer::from_slice(&decompressed);
        let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer).unwrap();
        let new_nes: Nes = serde_json::from_value(value).unwrap();
        assert!(new_nes.version.eq(SERIALIZATION_VER));
        *self = new_nes;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let nes_file = NesFile::new(rom);
        self.mapper = nes_file.create_mapper();
        self.power_cycle();
    }

    pub fn power_cycle(&mut self) {
        self.ppu.power_cycle();
        self.apu.power_cycle();
        self.ram.power_cycle();
        self.mapper.power_cycle();
        let mut cpu_bus = cpu_bus!(self, None);
        self.cpu.power_cycle(&mut cpu_bus);
        self.controllers.power_cycle();
    }

    pub fn run_for(&mut self, duration: Duration, callback: Option<&dyn ControllerCallback>) {
        let mut elapsed_frames = 0;
        while elapsed_frames < duration.as_secs() as u128 * DEFAULT_FPS as u128 {
            self.run_single_frame(callback);
            elapsed_frames += 1;
        }
    }

    pub fn run_single_frame(&mut self, callback: Option<&dyn ControllerCallback>) {
        self.emulation_frame.audio.reset();
        let current_frame = self.ppu.get_time().frame;
        while self.ppu.get_time().frame == current_frame {
            self.run_single_cpu_cycle(callback);
        }
        self.controllers
            .update_zappers(&self.emulation_frame, self.ppu.get_time().frame);
        self.apu.reset_audio_buffer();
    }

    fn run_single_cpu_cycle(&mut self, callback: Option<&dyn ControllerCallback>) {
        let mut cpu_bus = cpu_bus!(self, callback);
        self.cpu.maybe_fetch_next_instruction(&mut cpu_bus);
        let mut ppu_bus = PpuBus {
            mapper: &mut self.mapper,
            emulation_frame: &mut self.emulation_frame,
        };
        self.ppu.run_single_cpu_cycle(&mut ppu_bus);
        let mut apu_bus = ApuBus {
            ram: &mut self.ram,
            mapper: &mut self.mapper,
            emulation_frame: &mut self.emulation_frame,
            config: &self.config,
        };
        self.apu.run_single_cpu_cycle(&mut apu_bus);
        let mut cpu_bus = cpu_bus!(self, callback);
        self.cpu.run_single_cycle(&mut cpu_bus);
    }
}

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}
