use crate::common::Mirroring;

mod mapper0;
mod mapper2;
mod mapper227;
mod mapper66;
mod mapper_internal;

pub use self::mapper0::Mapper0;
pub use self::mapper2::Mapper2;
pub use self::mapper227::Mapper227;
pub use self::mapper66::Mapper66;

pub trait Mapper {
    fn get_chr_byte(&mut self, _: u16) -> u8;
    fn store_chr_byte(&mut self, _: u16, _: u8);

    fn get_pgr_byte(&mut self, _: u16) -> u8;
    fn store_pgr_byte(&mut self, _: u16, _: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn reset(&mut self);
}
