use super::mmc3_6::MMC3_6Variant;
use super::mmc3_6::MMC3_6;
use super::Mapper;
use crate::common::Mirroring;
pub struct Mapper4 {
    mmc3: MMC3_6,
}

impl Mapper4 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            mmc3: MMC3_6::new(prg_rom, chr_rom, MMC3_6Variant::MMC3HkROM),
        }
    }
}

impl Mapper for Mapper4 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        self.mmc3.get_chr_byte(address)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mmc3.get_mirroring()
    }
    fn get_prg_byte(&mut self, address: u16) -> u8 {
        self.mmc3.get_prg_byte(address)
    }

    fn reset(&mut self) {
        self.mmc3.reset();
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        self.mmc3.store_chr_byte(address, byte)
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        self.mmc3.store_prg_byte(address, byte)
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {
        self.mmc3.ppu_a12_rising_edge_triggered()
    }

    fn irq_pending(&mut self) -> bool {
        self.mmc3.irq_pending()
    }
}
