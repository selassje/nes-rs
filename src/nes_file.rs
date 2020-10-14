use crate::common;
use crate::mapper::{Mapper, Mapper0};

#[derive(PartialEq, Debug)]
enum NesFormat {
    Nes2_0,
    INes,
}
#[allow(dead_code)]
enum HeaderFlag6 {
    MirroringVertical = 0b00000001,
    PrgRAMPresent = 0b00000010,
    TrainerPresent = 0b00000100,
    IgnoreMirroring = 0b00001000,
}

#[allow(dead_code)]
enum HeaderFlag7 {
    VsSystem = 0b00000001,
    PlayChoice10 = 0b00000010,
    Flags8_15InNes2 = 0b00001100,
}

#[allow(dead_code)]
enum HeaderFlag9 {
    TvSystem = 0b00000001,
}

#[allow(dead_code)]
enum HeaderFlag10 {
    TvSystem = 0b00000011,
    PrgRAMPresent = 0b00010000,
    BusConflictPresent = 0b00100000,
}

#[derive(Debug)]
struct NesHeader {
    prg_rom_units: u8,
    chr_rom_units: u8,
    flag_6: u8,
    lo_n_mapper_number: u8,
    flag_7: u8,
    ho_n_mapper_number: u8,
    prg_ram_units: u8,
    flag_9: u8,
    flag_10: u8,
}

type Trainer = [u8; 512];
type PrgRomUnit = [u8; common::PRG_ROM_UNIT_SIZE];
type ChrRomUnit = [u8; common::CHR_ROM_UNIT_SIZE];

type PlayChoiceInstRom = [u8; 8192];
type PlayChoiceDecryptData = [u8; 16];

#[allow(dead_code)]
struct PlayChoiceRom {
    inst_rom: PlayChoiceInstRom,
    data_output: PlayChoiceDecryptData,
    counter_output: PlayChoiceDecryptData,
}

#[allow(dead_code)]
pub struct NesFile {
    trainer: Option<Trainer>,
    prg_rom: Vec<PrgRomUnit>,
    chr_rom: Vec<ChrRomUnit>,
    play_choice_rom: Option<PlayChoiceRom>,
    prg_ram_size: u32,
    mapper_number: u32,
    mirroring: common::Mirroring,
}

fn read_to_array(array: &mut [u8], in_bytes: &[u8]) -> usize {
    let unit_size = array.len();
    array.copy_from_slice(&in_bytes[0..unit_size]);
    unit_size
}

impl NesFile {
    pub fn create_mapper(&self) -> Box<dyn Mapper> {
        let mut prg_rom = Vec::<u8>::new();
        for prg_rom_chunk in &self.prg_rom {
            prg_rom.extend_from_slice(prg_rom_chunk);
        }

        let mut chr_rom = Vec::<u8>::new();
        for chr_rom_chunk in &self.chr_rom {
            chr_rom.extend_from_slice(chr_rom_chunk);
        }

        match self.mapper_number {
            0 => Box::new(Mapper0::new(prg_rom, chr_rom, self.mirroring)),
            _ => panic!("Unsupported mapper {}", self.mapper_number),
        }
    }

    fn get_format(header: &Vec<u8>) -> NesFormat {
        let mut is_ines_format = false;
        let mut is_nes2_format = false;
        if header[0] == 'N' as u8
            && header[1] == 'E' as u8
            && header[2] == 'S' as u8
            && header[3] == 0x1A
        {
            is_ines_format = true;
        }

        if is_ines_format && (header[7] & 0x0C == 0x08) {
            is_nes2_format = true;
        }

        if is_nes2_format {
            NesFormat::Nes2_0
        } else if is_ines_format {
            NesFormat::INes
        } else {
            panic!("Unknown format file")
        }
    }

    pub fn new(in_bytes: &Vec<u8>) -> NesFile {
        let format = Self::get_format(in_bytes);
        assert!(format == NesFormat::INes);

        let mut read_index = 4;

        let header = NesHeader {
            prg_rom_units: in_bytes[read_index],
            chr_rom_units: in_bytes[read_index + 1],
            flag_6: in_bytes[read_index + 2],
            lo_n_mapper_number: (in_bytes[read_index + 2] & 0xF0) >> 4,
            flag_7: in_bytes[read_index + 3],
            ho_n_mapper_number: (in_bytes[read_index + 3] & 0xF0) >> 4,
            prg_ram_units: in_bytes[read_index + 4],
            flag_9: in_bytes[read_index + 5],
            flag_10: in_bytes[read_index + 6],
        };

        let mirroring = if header.flag_6 & HeaderFlag6::MirroringVertical as u8 != 0 {
            common::Mirroring::VERTICAL
        } else {
            common::Mirroring::HORIZONTAL
        };
        read_index = 16;
        let mut trainer = Option::None;
        if header.flag_6 & (HeaderFlag6::TrainerPresent as u8) == 1 {
            let mut trainer_data: Trainer =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            read_index += read_to_array(&mut trainer_data, &in_bytes[read_index..]);
            trainer = Option::Some(trainer_data);
        }

        let mut prg_rom = Vec::<PrgRomUnit>::new();
        for _ in 0..header.prg_rom_units {
            let mut prg_rom_unit: PrgRomUnit = [0; 16384];
            read_index += read_to_array(&mut prg_rom_unit, &in_bytes[read_index..]);
            prg_rom.push(prg_rom_unit);
        }

        let mut chr_rom = Vec::<ChrRomUnit>::new();

        for _ in 0..header.chr_rom_units {
            let mut chr_rom_unit: ChrRomUnit =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            read_index += read_to_array(&mut chr_rom_unit, &in_bytes[read_index..]);
            chr_rom.push(chr_rom_unit);
        }

        let mut play_choice_rom = Option::None;

        if header.flag_7 & (HeaderFlag7::PlayChoice10 as u8) == 1 {
            let mut inst_rom: PlayChoiceInstRom =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            read_index += read_to_array(&mut inst_rom, &in_bytes[read_index..]);

            let mut data_output: PlayChoiceDecryptData =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            read_index += read_to_array(&mut data_output, &in_bytes[read_index..]);

            let mut counter_output: PlayChoiceDecryptData =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            read_to_array(&mut counter_output, &in_bytes[read_index..]);

            play_choice_rom = Some(PlayChoiceRom {
                inst_rom,
                data_output,
                counter_output,
            });
        }

        let mut prg_ram_size: u32 = 0;
        if header.flag_10 & (HeaderFlag10::PrgRAMPresent as u8) == 0 {
            prg_ram_size = common::PRG_RAM_UNIT_SIZE as u32 * (header.prg_ram_units as u32);
            if prg_ram_size == 0 {
                prg_ram_size = common::PRG_RAM_UNIT_SIZE as u32;
            }
        }

        let mapper_number = ((header.ho_n_mapper_number << 4) + header.lo_n_mapper_number) as u32;
        NesFile {
            trainer,
            prg_rom,
            chr_rom,
            play_choice_rom,
            prg_ram_size,
            mapper_number,
            mirroring,
        }
    }
}
