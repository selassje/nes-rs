use crate::apu::Apu;
use crate::common;
use crate::common::*;
use crate::controllers::Controllers;
use crate::mappers::Mapper;
use crate::mappers::MapperEnum;
use crate::mappers::MapperNull;
use crate::nes_file::NesFile;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const SERIALIZATION_VER: &str = "1";

type Ppu = crate::ppu::Ppu;
pub type Ram = crate::ram::Ram;
type Cpu = crate::cpu::Cpu;

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

#[derive(Clone)]
pub struct MouseClick {
    pub left_button: bool,
    pub right_button: bool,
    pub x: usize,
    pub y: usize,
}

pub trait ControllerAccess {
    fn is_button_pressed(&self, controller_id: crate::nes::ControllerId, button: StdNesControllerButton) -> bool;
    fn get_mouse_click(&self) -> MouseClick;
    fn get_current_frame(&self) -> u128;
}
pub struct CpuBus<'a> {
    pub ram: &'a mut Ram,
    pub ppu: &'a mut Ppu,
    pub apu: &'a mut Apu,
    pub mapper: &'a mut MapperEnum,
    pub controllers: &'a mut Controllers,
}

macro_rules! cpu_bus {
    ($nes:expr) => {{
        CpuBus {
            ram: &mut $nes.ram,
            ppu: &mut $nes.ppu,
            apu: &mut $nes.apu,
            mapper: &mut $nes.mapper,
            controllers: &mut $nes.controllers,
        }
    }};
}

pub struct PpuBus<'a> {
    pub mapper: &'a mut MapperEnum,
    pub emulation_frame: &'a mut EmulationFrame,
}
pub struct RamBus<'a> {
    pub apu: &'a mut Apu,
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut MapperEnum,
    pub controllers: &'a mut Controllers,
}

pub const VIDEO_FRAME_SIZE: usize = FRAME_HEIGHT * FRAME_WIDTH * PIXEL_SIZE;
pub const AUDIO_FRAME_SIZE: usize = 2048;
pub const SAMPLING_RATE: usize = 44100;

pub struct EmulationFrame {
    pub video: Box<[u8; VIDEO_FRAME_SIZE]>,
    pub audio: Box<[f32; AUDIO_FRAME_SIZE]>,
    pub audio_size: usize,
}

impl Default for EmulationFrame {
    fn default() -> Self {
        Self {
            video: Box::new([0; VIDEO_FRAME_SIZE]),
            audio: Box::new([0.0; AUDIO_FRAME_SIZE]),
            audio_size: 0,
        }
    }
}

impl EmulationFrame {
    pub fn get_audio_samples(&self) -> &[f32] {
        &self.audio[..self.audio_size]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ConfigImpl {
    pub audio_volume: f32,
    pub target_fps: u16,
}

impl Default for ConfigImpl {
    fn default() -> Self {
        ConfigImpl {
            audio_volume: 1.0,
            target_fps: common::DEFAULT_FPS,
        }
    }
}

pub struct Config<'a> {
    config: &'a mut ConfigImpl,
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

    pub fn set_controller_access(&mut self, controller_access: Rc<RefCell<dyn ControllerAccess>>) {
        self.controllers.set_controller_access(controller_access);
    }
}

pub struct ApuBus<'a> {
    pub ram: &'a mut Ram,
    pub mapper: &'a mut MapperEnum,
    pub emulation_frame: &'a mut EmulationFrame,
    pub config: &'a ConfigImpl,
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
    config: ConfigImpl,
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
            config: ConfigImpl::default(),
            emulation_frame: EmulationFrame::default(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
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

    pub fn config(&mut self) -> Config {
        Config {
            config: &mut self.config,
            controllers: &mut self.controllers,
        }
    }

    pub fn deserialize(&mut self, state: Vec<u8>) {
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
        let controller_access = self.controllers.get_controller_access();
        *self = new_nes;
        self.controllers.set_controller_access(controller_access);
    }

    pub fn load(&mut self, nes_file: &NesFile) {
        self.mapper = nes_file.create_mapper();
        self.power_cycle();
    }

    pub fn power_cycle(&mut self) {
        self.ppu.power_cycle();
        self.apu.power_cycle();
        self.ram.power_cycle();
        self.mapper.power_cycle();
        let mut cpu_bus = cpu_bus!(self);
        self.cpu.power_cycle(&mut cpu_bus);
    }

    pub fn run_for(&mut self, duration: Duration) {
        let mut elapsed_frames = 0;
        while elapsed_frames < duration.as_secs() as u128 * common::DEFAULT_FPS as u128 {
            self.run_single_frame();
            elapsed_frames += 1;
        }
    }

    pub fn run_single_frame(&mut self) {
        use crate::ppu::PpuState;
        self.emulation_frame.audio_size = 0;
        let current_frame = self.ppu.get_time().frame;
        while self.ppu.get_time().frame == current_frame {
            self.run_single_cpu_cycle();
        }
        self.controllers
            .update_luminance_for_zappers(&self.emulation_frame);
        self.apu.reset_audio_buffer();
    }

    fn run_single_cpu_cycle(&mut self) {
        let mut cpu_bus = cpu_bus!(self);
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
        let mut cpu_bus = cpu_bus!(self);
        self.cpu.run_single_cycle(&mut cpu_bus);
    }
}
