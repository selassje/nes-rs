use crate::nes::common::Mirroring;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct MapperNull {}

impl MapperNull {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Mapper for MapperNull {
    fn get_chr_byte(&self, _address: u16) -> u8 {
        0
    }

    fn get_prg_byte(&mut self, _address: u16) -> u8 {
        0
    }

    fn store_chr_byte(&mut self, _address: u16, _byte: u8) {}

    fn store_prg_byte(&mut self, _: u16, _: u8) {}

    fn get_mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn power_cycle(&mut self) {}
}
