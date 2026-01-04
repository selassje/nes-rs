use crate::apu::Apu;
use crate::common;
use crate::common::CPU_CYCLES_PER_FRAME;
use crate::controllers::ControllerId;
use crate::controllers::ControllerType;
use crate::controllers::Controllers;
use crate::io::AudioAccess;
use crate::io::ControllerAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::mappers::Mapper;
use crate::mappers::MapperEnum;
use crate::mappers::MapperNull;
use crate::nes_file::NesFile;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const SERIALIZATION_VER: &str = "1";

fn default_video_access() -> Rc<RefCell<dyn VideoAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_audio_access() -> Rc<RefCell<dyn AudioAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}

type Ppu = crate::ppu::Ppu;
pub type Ram = crate::ram::Ram;
type Cpu = crate::cpu::Cpu;

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
pub struct ApuBus<'a> {
    pub ram: &'a mut Ram,
    pub mapper: &'a mut MapperEnum,
    pub emulation_frame: &'a mut EmulationFrame,
}
pub struct RamBus<'a> {
    pub apu: &'a mut Apu,
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut MapperEnum,
    pub controllers: &'a mut Controllers,
}


pub struct EmulationFrame {
    pub video: [(u8,u8,u8); common::FRAME_WIDTH as usize * common::FRAME_HEIGHT as usize],
    pub audio: [f32; CPU_CYCLES_PER_FRAME as usize],
}

impl Default for EmulationFrame {
    fn default() -> Self {
        Self {
            video: [(0,0,0); common::FRAME_WIDTH as usize * common::FRAME_HEIGHT as usize],
            audio: [0.0; CPU_CYCLES_PER_FRAME as usize],
        }
    }
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
    #[serde(skip,default)]
    emulation_frame: EmulationFrame,
    #[serde(skip, default = "default_video_access")]
    video_access: Rc<RefCell<dyn VideoAccess>>,
    #[serde(skip, default = "default_audio_access")]
    audio_access: Rc<RefCell<dyn AudioAccess>>,
    #[serde(skip, default = "default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
}

impl Nes {
    pub fn new<T>(io: Rc<RefCell<T>>) -> Self
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        let controllers = Controllers::new();
        let mapper = MapperEnum::MapperNull(MapperNull::new());
        let ppu = Ppu::new(io.clone());
        let apu = Apu::new(io.clone());
        let ram = Ram::new();
        let cpu = Cpu::new();
        let mut nes = Nes {
            version: SERIALIZATION_VER.to_string(),
            cpu,
            ram,
            ppu,
            apu,
            controllers,
            mapper,
            video_access: io.clone(),
            audio_access: io.clone(),
            controller_access: io.clone(),
            emulation_frame: EmulationFrame::default(),
        };
        nes.set_controller(ControllerId::Controller1, ControllerType::StdNesController);
        nes.set_controller(ControllerId::Controller2, ControllerType::StdNesController);

        nes
    }

    pub fn set_controller(&mut self, id: ControllerId, controller_type: ControllerType) {
        self.controllers
            .set_controller(id, controller_type, self.controller_access.clone());
    }
    pub fn get_controller_type(&self, id: ControllerId) -> ControllerType {
        self.controllers.get_controller_type(id)
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
        let video_access = self.video_access.clone();
        let audio_access = self.audio_access.clone();
        let controller_access = self.controller_access.clone();

        *self = new_nes;

        self.ppu.set_video_access(video_access.clone());

        self.apu.set_audio_access(audio_access.clone());

        self.controllers
            .set_controller_access(controller_access.clone());

        self.video_access = video_access;
        self.audio_access = audio_access;
        self.controller_access = controller_access;
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

        let current_frame = self.ppu.get_time().frame;

        while self.ppu.get_time().frame == current_frame {
            self.run_single_cpu_cycle();
        }
    }

    pub fn run_single_cpu_cycle(&mut self) {
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
        };
        self.apu.run_single_cpu_cycle(&mut apu_bus);
        let mut cpu_bus = cpu_bus!(self);
        self.cpu.run_single_cycle(&mut cpu_bus);
    }
}
