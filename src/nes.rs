use crate::apu::Apu;
use crate::common;
use crate::common::NonNullPtr;
use crate::controllers::Controllers;
use crate::io::AudioAccess;
use crate::io::ControllerAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::mappers::*;
use crate::nes_file::NesFile;
use crate::vram::VRam;
use crate::{mappers::Mapper, mappers::MapperNull};

use std::cell::RefCell;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

fn default_video_access() -> Rc<RefCell<dyn VideoAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_audio_access() -> Rc<RefCell<dyn AudioAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}

type Ppu = crate::ppu::Ppu<VRam>;
pub type Ram = crate::ram::Ram<Ppu, Apu, Controllers>;
type Cpu = crate::cpu::Cpu<Ram, Ppu, Apu>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NesInternal {
    cpu: Cpu,
    ram: Ram,
    ppu: Ppu,
    vram: VRam,
    apu: Apu,
    controllers: Controllers,
    mapper: MapperEnum,
    #[serde(skip, default = "default_video_access")]
    video_access: Rc<RefCell<dyn VideoAccess>>,
    #[serde(skip, default = "default_audio_access")]
    audio_access: Rc<RefCell<dyn AudioAccess>>,
    #[serde(skip, default = "default_controller_access")]
    controller_access: Rc<RefCell<dyn ControllerAccess>>,
    #[serde(skip)]
    _pin: PhantomPinned,
}

impl NesInternal {
    fn new<T>(io: Rc<RefCell<T>>) -> Pin<Box<Self>>
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        let controllers = Controllers::new(io.clone());
        let mapper = MapperEnum::MapperNull(MapperNull::new());
        let vram = VRam::new();
        let ppu = Ppu::new(io.clone());
        let apu = Apu::new(io.clone());
        let ram = Ram::new();
        let cpu = Cpu::new();

        unsafe {
            let mut pinned_nes = std::pin::Pin::new_unchecked(Box::new(NesInternal {
                cpu,
                ram,
                ppu,
                vram,
                apu,
                controllers,
                mapper,
                video_access: io.clone(),
                audio_access: io.clone(),
                controller_access: io,
                _pin: PhantomPinned,
            }));

            let ram = NonNullPtr::from(&pinned_nes.ram);
            let ppu = NonNullPtr::from(&pinned_nes.ppu);
            let apu = NonNullPtr::from(&pinned_nes.apu);
            let vram = NonNullPtr::from(&pinned_nes.vram);
            let controllers = NonNullPtr::from(&pinned_nes.controllers);
            let mapper = NonNullPtr::from(&pinned_nes.mapper);

            let pin_ref: Pin<&mut Self> = Pin::as_mut(&mut pinned_nes);
            let nes = Pin::get_unchecked_mut(pin_ref);
            nes.cpu.set_ram(ram);
            nes.cpu.set_ppu_state(ppu);
            nes.cpu.set_apu_state(apu);
            nes.cpu.set_mapper(mapper);
            nes.ram.set_controller_access(controllers);
            nes.ram.set_ppu_access(ppu);
            nes.ram.set_apu_access(apu);
            nes.ram.set_mapper(mapper);
            nes.vram.set_mapper(mapper);
            nes.apu.set_dmc_memory(ram);
            nes.ppu.set_vram(vram);
            nes.ppu.set_mapper(mapper);
            pinned_nes
        }
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize(&mut self, state: String) {
        let mut deserializer = serde_json::Deserializer::from_str(&state);
        let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer).unwrap();
        let new_nes: NesInternal = serde_json::from_value(value).unwrap();
        let video_access = self.video_access.clone();
        let audio_access = self.audio_access.clone();
        let controller_access = self.controller_access.clone();

        *self = new_nes;

        let mapper = NonNullPtr::from(&self.mapper);

        self.vram.set_mapper(mapper);
        self.ppu.set_mapper(mapper);
        self.ram.set_mapper(mapper);
        self.cpu.set_mapper(mapper);

        self.ppu.set_vram(NonNullPtr::from(&self.vram));
        self.ppu.set_video_access(video_access.clone());

        self.apu.set_audio_access(audio_access.clone());

        self.apu.set_dmc_memory(NonNullPtr::from(&self.ram));

        self.ram.set_apu_access(NonNullPtr::from(&self.apu));
        self.ram.set_ppu_access(NonNullPtr::from(&self.ppu));
        self.ram
            .set_controller_access(NonNullPtr::from(&self.controllers));
        self.cpu.set_ram(NonNullPtr::from(&self.ram));
        self.cpu.set_ppu_state(NonNullPtr::from(&self.ppu));
        self.cpu.set_apu_state(NonNullPtr::from(&self.apu));

        self.controllers
            .set_controller_access(controller_access.clone());

        self.video_access = video_access;
        self.audio_access = audio_access;
        self.controller_access = controller_access;
    }

    fn load(&mut self, nes_file: &NesFile) {
        self.mapper = nes_file.create_mapper();
        self.power_cycle();
    }

    fn power_cycle(&mut self) {
        self.vram.power_cycle();
        self.ppu.power_cycle();
        self.apu.power_cycle();
        self.ram.power_cycle();
        self.mapper.power_cycle();
        self.cpu.power_cycle();
    }

    fn run_for(&mut self, duration: Duration) {
        let mut elapsed_frames = 0;
        while elapsed_frames < duration.as_secs() as u128 * common::DEFAULT_FPS as u128 {
            self.run_single_frame();
            elapsed_frames += 1;
        }
    }

    fn run_single_frame(&mut self) {
        for _ in 0..common::CPU_CYCLES_PER_FRAME {
            self.run_single_cpu_cycle();
        }
    }

    fn run_single_cpu_cycle(&mut self) {
        self.cpu.maybe_fetch_next_instruction();

        self.ppu.run_single_cpu_cycle();

        self.apu.run_single_cpu_cycle();

        self.cpu.run_single_cycle();
    }
}

pub struct Nes {
    nes: Pin<Box<NesInternal>>,
}

impl Nes {
    pub fn new<T>(io: Rc<RefCell<T>>) -> Self
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        Self {
            nes: NesInternal::new(io),
        }
    }

    fn as_mut(&mut self) -> &mut NesInternal {
        let pin_ref = Pin::as_mut(&mut self.nes);
        unsafe { Pin::get_unchecked_mut(pin_ref) }
    }

    pub fn serialize(&self) -> String {
        self.nes.serialize()
    }

    pub fn deserialize(&mut self, state: String) {
        self.as_mut().deserialize(state);
    }

    pub fn load(&mut self, nes_file: &NesFile) {
        self.as_mut().load(nes_file);
    }

    pub fn power_cycle(&mut self) {
        self.as_mut().power_cycle();
    }

    pub fn run_for(&mut self, duration: Duration) {
        self.as_mut().run_for(duration);
    }

    pub fn run_single_frame(&mut self) {
        self.as_mut().run_single_frame();
    }
}
