use super::common::Mirroring;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper227;
mod mapper4;
mod mapper5;
mod mapper66;
mod mapper7;
mod mapper71;
mod mapper_null;
mod mmc3_6;

mod mapper_internal;

pub use self::mapper_null::MapperNull;
pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper4::Mapper4;
pub use self::mapper5::Mapper5;
pub use self::mapper7::Mapper7;
pub use self::mapper66::Mapper66;
pub use self::mapper71::Mapper71;
pub use self::mapper227::Mapper227;

const PRG_RAM_RANGE: std::ops::Range<u16> = std::ops::Range {
    start: 0x6000,
    end: 0x8000,
};

const PRG_RANGE: std::ops::RangeInclusive<u16> = 0x6000..=0xFFFF;

use super::common::NametableSource;

#[enum_dispatch::enum_dispatch(MapperEnum)]
pub trait Mapper {
    fn get_chr_byte(&self, address: u16) -> u8;
    fn store_chr_byte(&mut self, address: u16, byte: u8);

    fn get_prg_byte(&mut self, address: u16) -> u8;
    fn store_prg_byte(&mut self, address: u16, byte: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn power_cycle(&mut self);

    fn is_irq_pending(&self) -> bool {
        false
    }

    fn notify_scanline(&mut self) {}
    fn notify_vblank(&mut self) {}

    fn ppu_a12_rising_edge_triggered(&mut self) {}

    fn get_nametable_byte(&self, _source: NametableSource, _offset: u16) -> Option<u8> {
        None
    }

    fn store_nametable_byte(&mut self, _source: NametableSource, _offset: u16, _byte: u8) -> bool {
        false
    }

    fn notify_ppu_register_write(&mut self, _address: u16, _value: u8) {}

    fn notify_oam_dma_write(&mut self) {}

    fn notify_background_tiles_fetch(&mut self) {}

    fn notify_sprite_tiles_fetch(&mut self) {}

    fn get_attribute_data(&mut self, _tile_x: u8, _tile_y: u8) -> Option<u8> {
        None
    }

    fn get_low_pattern_data(
        &self,
        _table_index: u8,
        _pattern_tile_index: u8,
        _y: u8,
    ) -> Option<u8> {
        None
    }
    fn get_high_pattern_data(
        &self,
        _table_index: u8,
        _pattern_tile_index: u8,
        _y: u8,
    ) -> Option<u8> {
        None
    }
}

#[enum_dispatch::enum_dispatch]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum MapperEnum {
    MapperNull(self::mapper_null::MapperNull),
    Mapper0(self::mapper0::Mapper0),
    Mapper1(self::mapper1::Mapper1),
    Mapper2(self::mapper2::Mapper2),
    Mapper4(self::mapper4::Mapper4),
    Mapper5(self::mapper5::Mapper5),
    Mapper7(self::mapper7::Mapper7),
    Mapper66(self::mapper66::Mapper66),
    Mapper71(self::mapper71::Mapper71),
    Mapper227(self::mapper227::Mapper227),
}
