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

type Ppu = crate::ppu::Ppu<VRam>;
pub type Ram = crate::ram::Ram<Ppu, Apu, Controllers>;
type Cpu = crate::cpu::Cpu<Ram, Ppu, Apu>;

#[derive(serde::Serialize, Deserialize)]
pub struct NesInternal {
    cpu: Cpu,
    ram: Ram,
    ppu: Ppu,
    vram: VRam,
    apu: Apu,
    controllers: Controllers,
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
    #[serde(skip)]
    _pin: PhantomPinned,
}

impl NesInternal {
    fn new<T>(io: Rc<RefCell<T>>) -> Pin<Box<Self>>
    where
        T: IO + VideoAccess + AudioAccess + ControllerAccess + 'static,
    {
        let controllers = Controllers::new(io.clone());
        let mapper = Rc::new(RefCell::new(MapperNull::new()));
        let vram = VRam::new(mapper.clone());
        let ppu = Ppu::new(io.clone(), mapper.clone());
        let apu = Apu::new(io.clone());
        let ram = Ram::new(mapper.clone());
        let cpu = Cpu::new(mapper.clone());

        let mut nes = Box::pin(NesInternal {
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
        });

        let ram = NonNullPtr::from(&nes.ram);
        let ppu = NonNullPtr::from(&nes.ppu);
        let apu = NonNullPtr::from(&nes.apu);
        let vram = NonNullPtr::from(&nes.vram);
        let controllers = NonNullPtr::from(&nes.controllers);

        unsafe {
            let pin_ref: Pin<&mut Self> = Pin::as_mut(&mut nes);
            let nes = Pin::get_unchecked_mut(pin_ref);
            nes.cpu.set_ram(ram);
            nes.cpu.set_ppu_state(ppu);
            nes.cpu.set_apu_state(apu);
            nes.ram.set_controller_access(controllers);
            nes.ram.set_ppu_access(ppu);
            nes.ram.set_apu_access(apu);
            nes.apu.set_dmc_memory(ram);
            nes.ppu.set_vram(vram);
        }
        nes
    }

    fn serialize(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    fn deserialize(&mut self, state: String) {
        let mut new_nes: NesInternal = serde_yaml::from_str(&state).unwrap();
        let mapper = new_nes.mapper.clone();
        let video_access = self.video_access.clone();
        let audio_access = self.audio_access.clone();
        let controller_access = self.controller_access.clone();

        new_nes.vram.set_mapper(mapper.clone());
        new_nes.ppu.set_mapper(mapper.clone());
        new_nes.ppu.set_vram(NonNullPtr::from(&new_nes.vram));
        new_nes.ppu.set_video_access(video_access.clone());

        new_nes.apu.set_audio_access(audio_access.clone());

        new_nes.apu.set_dmc_memory(NonNullPtr::from(&new_nes.ram));

        new_nes.ram.set_apu_access(NonNullPtr::from(&new_nes.apu));
        new_nes.ram.set_ppu_access(NonNullPtr::from(&new_nes.ppu));
        new_nes
            .ram
            .set_controller_access(NonNullPtr::from(&new_nes.controllers));
        new_nes.ram.set_mapper(mapper.clone());

        new_nes.cpu.set_mapper(mapper);
        new_nes.cpu.set_ram(NonNullPtr::from(&new_nes.ram));
        new_nes.cpu.set_ppu_state(NonNullPtr::from(&new_nes.ppu));
        new_nes.cpu.set_apu_state(NonNullPtr::from(&new_nes.apu));

        new_nes.video_access = video_access;
        new_nes.audio_access = audio_access;
        new_nes.controller_access = controller_access;

        *self = new_nes;
    }

    fn load(&mut self, nes_file: &NesFile) {
        let mapper = nes_file.create_mapper();
        self.vram.set_mapper(mapper.clone());
        self.ppu.set_mapper(mapper.clone());
        self.ram.set_mapper(mapper.clone());
        self.cpu.set_mapper(mapper.clone());
        self.mapper = mapper;
        self.power_cycle();
    }

    fn power_cycle(&mut self) {
        self.vram.power_cycle();
        self.ppu.power_cycle();
        self.apu.power_cycle();
        self.ram.power_cycle();
        self.mapper.borrow_mut().power_cycle();
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
