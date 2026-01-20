use serde::{Deserialize, Serialize};

pub const PRG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHR_ROM_UNIT_SIZE: usize = 0x2000;
pub const PRG_RAM_UNIT_SIZE: usize = 0x2000;

pub const CPU_CYCLES_PER_FRAME: usize = 29780;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NametableSource {
    Vram0,
    Vram1,
    ExRam,
    Fill,
}

impl TryFrom<u8> for NametableSource {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NametableSource::Vram0),
            1 => Ok(NametableSource::Vram1),
            2 => Ok(NametableSource::ExRam),
            3 => Ok(NametableSource::Fill),
            _ => Err("Invalid nametable source value"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mirroring {
    pub tables: [NametableSource; 4],
}

impl Mirroring {
    pub const VERTICAL: Self = Self {
        tables: [
            NametableSource::Vram0,
            NametableSource::Vram1,
            NametableSource::Vram0,
            NametableSource::Vram1,
        ],
    };

    pub const HORIZONTAL: Self = Self {
        tables: [
            NametableSource::Vram0,
            NametableSource::Vram0,
            NametableSource::Vram1,
            NametableSource::Vram1,
        ],
    };

    pub const SINGLE_SCREEN_0: Self = Self {
        tables: [
            NametableSource::Vram0,
            NametableSource::Vram0,
            NametableSource::Vram0,
            NametableSource::Vram0,
        ],
    };

    pub const SINGLE_SCREEN_1: Self = Self {
        tables: [
            NametableSource::Vram1,
            NametableSource::Vram1,
            NametableSource::Vram1,
            NametableSource::Vram1,
        ],
    };
}

pub fn convert_2u8_to_u16(b0: u8, b1: u8) -> u16 {
    (b0 as u16) | ((b1 as u16) << 8)
}
