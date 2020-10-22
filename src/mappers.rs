use crate::common::Mirroring;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper227;
mod mapper4;
mod mapper66;
mod mapper7;
mod mmc3_6;

mod mapper_internal;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper227::Mapper227;
pub use self::mapper4::Mapper4;
pub use self::mapper66::Mapper66;
pub use self::mapper7::Mapper7;

pub trait Mapper {
    fn get_chr_byte(&mut self, _: u16) -> u8;
    fn store_chr_byte(&mut self, _: u16, _: u8);

    fn get_prg_byte(&mut self, _: u16) -> u8;
    fn store_prg_byte(&mut self, _: u16, _: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn reset(&mut self);

    fn irq_pending(&mut self) -> bool {
        false
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {}
}
