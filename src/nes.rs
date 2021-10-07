use crate::apu::Apu;
use crate::common;
use crate::controllers;
use crate::io::AudioAccess;
use crate::io::ControllerAccess;
use crate::io::VideoAccess;
use crate::io::IO;
use crate::mappers::*;
use crate::nes_file::NesFile;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::vram::VRam;
use crate::{cpu::Cpu, mappers::Mapper, mappers::MapperNull};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use serde::ser::SerializeStruct;
use serde::ser::Serializer;
use serde::Deserialize;

fn serialize_mapper<S>(
    mapper: &Rc<RefCell<dyn Mapper>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("mapper", 2)?;
    state.serialize_field("mapperid", &mapper.borrow().get_mapper_id())?;
    state.serialize_field("concretemapper", &*mapper.borrow())?;
    state.end()
}

fn deserialize_mapper<'de, D>(
    deserializer: D,
) -> std::result::Result<Rc<RefCell<dyn Mapper>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(PartialEq, Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        MapperId,
        ConcreteMapper,
    }

    struct MapperVisitor;
    impl<'de> serde::de::Visitor<'de> for MapperVisitor {
        type Value = Rc<RefCell<dyn Mapper>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Mapper trait")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut mapperfield: Field = map.next_key().unwrap().unwrap();
            assert!(mapperfield == Field::MapperId);
            let mapperid: u8 = map.next_value().unwrap();
            mapperfield = map.next_key().unwrap().unwrap();
            assert!(mapperfield == Field::ConcreteMapper);

            let mapper: Self::Value = match mapperid {
                0 => Rc::new(RefCell::new(map.next_value::<Mapper0>().unwrap())),
                1 => Rc::new(RefCell::new(map.next_value::<Mapper1>().unwrap())),
                2 => Rc::new(RefCell::new(map.next_value::<Mapper2>().unwrap())),
                4 => Rc::new(RefCell::new(map.next_value::<Mapper4>().unwrap())),
                7 => Rc::new(RefCell::new(map.next_value::<Mapper7>().unwrap())),
                66 => Rc::new(RefCell::new(map.next_value::<Mapper66>().unwrap())),
                71 => Rc::new(RefCell::new(map.next_value::<Mapper71>().unwrap())),
                227 => Rc::new(RefCell::new(map.next_value::<Mapper227>().unwrap())),
                255 => Rc::new(RefCell::new(map.next_value::<MapperNull>().unwrap())),
                _ => panic!("Unsupported mapper {}", mapperid),
            };
            Ok(mapper)
        }
    }
    const FIELDS: &[&str] = &["mapperid", "concretemapper"];
    deserializer.deserialize_struct("mapper", FIELDS, MapperVisitor)
}

fn default_video_access() -> Rc<RefCell<dyn VideoAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_audio_access() -> Rc<RefCell<dyn AudioAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}
fn default_controller_access() -> Rc<RefCell<dyn ControllerAccess>> {
    Rc::new(RefCell::new(crate::io::DummyIOImpl::new()))
}

#[derive(serde::Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    ram: Rc<RefCell<Ram>>,
    ppu: Rc<RefCell<Ppu>>,
    vram: Rc<RefCell<VRam>>,
    apu: Rc<RefCell<Apu>>,
    #[serde(
        serialize_with = "serialize_mapper",
        deserialize_with = "deserialize_mapper"
    )]
    mapper: Rc<RefCell<dyn Mapper>>,
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
        let controllers = Rc::new(RefCell::new(controllers::Controllers::new(io.clone())));
        let mapper = Rc::new(RefCell::new(MapperNull::new()));
        let vram = Rc::new(RefCell::new(VRam::new(mapper.clone())));
        let ppu = Rc::new(RefCell::new(Ppu::new(
            vram.clone(),
            io.clone(),
            mapper.clone(),
        )));
        let apu = Rc::new(RefCell::new(Apu::new(io.clone())));
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
            video_access: io.clone(),
            audio_access: io.clone(),
            controller_access: io,
        }
    }

    pub fn serialize(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn deserialize(&mut self, state: String) {
        let mut new_nes: Nes = serde_yaml::from_str(&state).unwrap();
        let mapper = new_nes.mapper.clone();
        let video_access = self.video_access.clone();
        let audio_access = self.audio_access.clone();
        let controller_access = self.controller_access.clone();

        let controllers = Rc::new(RefCell::new(controllers::Controllers::new(
            controller_access.clone(),
        )));
        new_nes.vram.borrow_mut().set_mapper(mapper.clone());
        new_nes.ppu.borrow_mut().set_mapper(mapper.clone());
        new_nes.ppu.borrow_mut().set_vram(new_nes.vram.clone());
        new_nes
            .ppu
            .borrow_mut()
            .set_video_access(video_access.clone());

        new_nes
            .apu
            .borrow_mut()
            .set_audio_access(audio_access.clone());

        new_nes.apu.borrow_mut().set_dmc_memory(new_nes.ram.clone());

        new_nes.ram.borrow_mut().set_apu_access(new_nes.apu.clone());
        new_nes.ram.borrow_mut().set_ppu_access(new_nes.ppu.clone());
        new_nes.ram.borrow_mut().set_controller_access(controllers);
        new_nes.ram.borrow_mut().set_mapper(mapper.clone());

        new_nes.cpu.set_mapper(mapper);
        new_nes.cpu.set_ram(new_nes.ram.clone());
        new_nes.cpu.set_ppu_state(new_nes.ppu.clone());
        new_nes.cpu.set_apu_state(new_nes.apu.clone());

        new_nes.video_access = video_access;
        new_nes.audio_access = audio_access;
        new_nes.controller_access = controller_access;

        *self = new_nes;
    }

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
