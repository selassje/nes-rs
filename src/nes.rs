use crate::apu::Apu;
use crate::common;
use crate::controllers;
use crate::io::AudioAccess;
use crate::io::ControllerAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::nes_file::NesFile;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::vram::VRam;
use crate::{cpu::Cpu, mappers::Mapper, mappers::MapperNull};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;
//#[derive(serde::Serialize)]
pub struct Nes {
    cpu: Cpu,
    ram: Rc<RefCell<Ram>>,
    ppu: Rc<RefCell<Ppu>>,
    vram: Rc<RefCell<VRam>>,
    apu: Rc<RefCell<Apu>>,
    mapper: Rc<RefCell<dyn Mapper>>,
}

impl Serialize for Nes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Nes", 6)?;
        state.serialize_field("cpu", &self.cpu)?;
        state.serialize_field("ram", &*self.ram.borrow())?;
        state.serialize_field("ppu", &*self.ppu.borrow())?;
        state.serialize_field("vram", &*self.vram.borrow())?;
        state.serialize_field("apu", &*self.apu.borrow())?;
        state.serialize_field("mapper", &*self.mapper.borrow())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Nes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl Nes {
    pub fn new<T>(io: Rc<RefCell<T>>) -> Self
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        let controllers = Rc::new(RefCell::new(controllers::Controllers::new(io.clone())));
        let mapper = Rc::new(RefCell::new(MapperNull::new()));
        let vram = Rc::new(RefCell::new(VRam::new(mapper.clone())));
        let ppu = Rc::new(RefCell::new(Ppu::new(
            vram.clone(),
            io.clone(),
            mapper.clone(),
        )));
        let apu = Rc::new(RefCell::new(Apu::new(io)));
        let ram = Rc::new(RefCell::new(Ram::new(
            ppu.clone(),
            controllers,
            apu.clone(),
            mapper.clone(),
        )));

        apu.borrow_mut().set_dmc_memory(ram.clone());
        let cpu = Cpu::new(ram.clone(), ppu.clone(), apu.clone(), mapper.clone());

        Nes {
            cpu,
            ram,
            ppu,
            vram,
            apu,
            mapper,
        }
    }

    pub fn serialize(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn deserialize(&mut self, state: String) {}

    pub fn load(&mut self, nes_file: &NesFile) {
        let mapper = nes_file.create_mapper();
        self.vram.borrow_mut().set_mapper(mapper.clone());
        self.ppu.borrow_mut().set_mapper(mapper.clone());
        self.ram.borrow_mut().set_mapper(mapper.clone());
        self.cpu.set_mapper(mapper.clone());
        self.mapper = mapper;
        self.power_cycle();
    }

    pub fn power_cycle(&mut self) {
        self.vram.borrow_mut().power_cycle();
        self.ppu.borrow_mut().power_cycle();
        self.apu.borrow_mut().power_cycle();
        self.ram.borrow_mut().power_cycle();
        self.mapper.borrow_mut().power_cycle();
        self.cpu.power_cycle();
    }

    pub fn run_for(&mut self, duration: Duration) {
        let mut elapsed_frames = 0;
        while elapsed_frames < duration.as_secs() as u128 * common::DEFAULT_FPS as u128 {
            self.run_single_frame();
            elapsed_frames += 1;
        }
    }

    pub fn run_single_frame(&mut self) {
        for _ in 0..common::CPU_CYCLES_PER_FRAME {
            self.run_single_cpu_cycle();
        }
    }

    fn run_single_cpu_cycle(&mut self) {
        self.cpu.maybe_fetch_next_instruction();

        self.ppu.borrow_mut().run_single_cpu_cycle();

        self.apu.borrow_mut().run_single_cpu_cycle();

        self.cpu.run_single_cycle();
    }
}
