use crate::common::Mirroring;

mod mapper0;

pub use self::mapper0::Mapper0;

pub trait Mapper {
    fn get_rom_start(&self) -> u16;

    fn get_pgr_rom(&self) -> &[u8];

    fn get_chr_rom(&self) -> &[u8];

    fn get_mirroring(&self) -> Mirroring;

    fn get_byte(&mut self, _: u16) -> u8 {
        unimplemented!("get byte")
    }

    fn store_byte(&mut self, _: u16, _: u8) {
        unimplemented!("store byte")
    }
}
