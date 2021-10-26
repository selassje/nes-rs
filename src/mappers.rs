use crate::common::Mirroring;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper227;
mod mapper4;
mod mapper66;
mod mapper7;
mod mapper71;
mod mapper_null;
mod mmc3_6;

mod mapper_internal;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper227::Mapper227;
pub use self::mapper4::Mapper4;
pub use self::mapper66::Mapper66;
pub use self::mapper7::Mapper7;
pub use self::mapper71::Mapper71;
pub use self::mapper_null::MapperNull;

pub trait Mapper {
    fn get_chr_byte(&mut self, address: u16) -> u8;
    fn store_chr_byte(&mut self, address: u16, byte: u8);

    fn get_prg_byte(&mut self, address: u16) -> u8;
    fn store_prg_byte(&mut self, address: u16, byte: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn power_cycle(&mut self);

    fn is_irq_pending(&mut self) -> bool {
        false
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {}
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum MapperEnum {
    MapperNull(self::mapper_null::MapperNull),
    Mapper0(self::mapper0::Mapper0),
    Mapper1(self::mapper1::Mapper1),
    Mapper2(self::mapper2::Mapper2),
    Mapper4(self::mapper4::Mapper4),
    Mapper7(self::mapper7::Mapper7),
    Mapper66(self::mapper66::Mapper66),
    Mapper71(self::mapper71::Mapper71),
    Mapper227(self::mapper227::Mapper227),
}

impl Mapper for MapperEnum {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper0(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper1(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper2(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper4(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper7(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper66(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper71(mapper) => mapper.get_chr_byte(address),
            MapperEnum::Mapper227(mapper) => mapper.get_chr_byte(address),
        }
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper0(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper1(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper2(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper4(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper7(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper66(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper71(mapper) => mapper.store_chr_byte(address, byte),
            MapperEnum::Mapper227(mapper) => mapper.store_chr_byte(address, byte),
        }
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper0(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper1(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper2(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper4(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper7(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper66(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper71(mapper) => mapper.get_prg_byte(address),
            MapperEnum::Mapper227(mapper) => mapper.get_prg_byte(address),
        }
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper0(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper1(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper2(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper4(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper7(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper66(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper71(mapper) => mapper.store_prg_byte(address, byte),
            MapperEnum::Mapper227(mapper) => mapper.store_prg_byte(address, byte),
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper0(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper1(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper2(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper4(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper7(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper66(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper71(mapper) => mapper.get_mirroring(),
            MapperEnum::Mapper227(mapper) => mapper.get_mirroring(),
        }
    }

    fn power_cycle(&mut self) {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper0(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper1(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper2(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper4(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper7(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper66(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper71(mapper) => mapper.power_cycle(),
            MapperEnum::Mapper227(mapper) => mapper.power_cycle(),
        }
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper0(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper1(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper2(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper4(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper7(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper66(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper71(mapper) => mapper.ppu_a12_rising_edge_triggered(),
            MapperEnum::Mapper227(mapper) => mapper.ppu_a12_rising_edge_triggered(),
        }
    }

    fn is_irq_pending(&mut self) -> bool {
        match self {
            MapperEnum::MapperNull(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper0(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper1(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper2(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper4(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper7(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper66(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper71(mapper) => mapper.is_irq_pending(),
            MapperEnum::Mapper227(mapper) => mapper.is_irq_pending(),
        }
    }
}
