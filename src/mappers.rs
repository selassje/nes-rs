use crate::common::Mirroring;

mod mapper0;

const CHR_DATA_SIZE: usize = 0x2000;

pub use self::mapper0::Mapper0;

pub trait Mapper {
    fn get_chr_byte(&mut self, _: u16) -> u8;
    fn store_chr_byte(&mut self, _: u16, _: u8);

    fn get_pgr_byte(&mut self, _: u16) -> u8;
    fn store_pgr_byte(&mut self, _: u16, _: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn reset(&mut self);
}
