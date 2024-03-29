use crate::common;
use crate::mappers::*;

#[derive(PartialEq, Debug)]
enum NesFormat {
    Nes2_0,
    INes,
}
enum HeaderFlag6 {
    MirroringVertical = 0b00000001,
    _PrgRAMPresent = 0b00000010,
    TrainerPresent = 0b00000100,
    _IgnoreMirroring = 0b00001000,
}

enum HeaderFlag7 {
    _VsSystem = 0b00000001,
    PlayChoice10 = 0b00000010,
    _Flags8_15InNes2 = 0b00001100,
}

enum _HeaderFlag9 {
    TvSystem = 0b00000001,
}

enum HeaderFlag10 {
    _TvSystem = 0b00000011,
    PrgRAMPresent = 0b00010000,
    _BusConflictPresent = 0b00100000,
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
    _flag_9: u8,
    flag_10: u8,
}

type Trainer = [u8; 512];
type PrgRomUnit = [u8; common::PRG_ROM_UNIT_SIZE];
type ChrRomUnit = [u8; common::CHR_ROM_UNIT_SIZE];

type PlayChoiceInstRom = [u8; 8192];
type PlayChoiceDecryptData = [u8; 16];

struct PlayChoiceRom {
    _inst_rom: PlayChoiceInstRom,
    _data_output: PlayChoiceDecryptData,
    _counter_output: PlayChoiceDecryptData,
}

pub struct NesFile {
    _trainer: Option<Trainer>,
    prg_rom: Vec<PrgRomUnit>,
    chr_rom: Vec<ChrRomUnit>,
    _play_choice_rom: Option<PlayChoiceRom>,
    _prg_ram_size: u32,
    mapper_number: u32,
    mirroring: common::Mirroring,
}

fn read_to_array(array: &mut [u8], in_bytes: &[u8]) -> usize {
    let unit_size = array.len();
    array.copy_from_slice(&in_bytes[0..unit_size]);
    unit_size
}

impl NesFile {
    pub fn create_mapper(&self) -> MapperEnum {
        let mut prg_rom = Vec::<u8>::new();
        for prg_rom_chunk in &self.prg_rom {
            prg_rom.extend_from_slice(prg_rom_chunk);
        }

        let mut chr_rom = Vec::<u8>::new();
        for chr_rom_chunk in &self.chr_rom {
            chr_rom.extend_from_slice(chr_rom_chunk);
        }

        match self.mapper_number {
            0 => MapperEnum::Mapper0(Mapper0::new(prg_rom, chr_rom, self.mirroring)),
            1 => MapperEnum::Mapper1(Mapper1::new(prg_rom, chr_rom)),
            2 => MapperEnum::Mapper2(Mapper2::new(prg_rom, chr_rom, self.mirroring)),
            4 => MapperEnum::Mapper4(Mapper4::new(prg_rom, chr_rom)),
            7 => MapperEnum::Mapper7(Mapper7::new(prg_rom, chr_rom)),
            66 => MapperEnum::Mapper66(Mapper66::new(prg_rom, chr_rom, self.mirroring)),
            71 => MapperEnum::Mapper71(Mapper71::new(prg_rom, self.mirroring)),
            227 => MapperEnum::Mapper227(Mapper227::new(prg_rom, chr_rom)),
            _ => panic!("Unsupported mapper {}", self.mapper_number),
        }
    }

    fn get_format(header: &[u8]) -> NesFormat {
        let mut is_ines_format = false;
        let mut is_nes2_format = false;
        if header[0] == b'N' && header[1] == b'E' && header[2] == b'S' && header[3] == 0x1A {
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

    pub fn new(in_bytes: &[u8]) -> NesFile {
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
            _flag_9: in_bytes[read_index + 5],
            flag_10: in_bytes[read_index + 6],
        };

        let mirroring = if header.flag_6 & HeaderFlag6::MirroringVertical as u8 != 0 {
            common::Mirroring::Vertical
        } else {
            common::Mirroring::Horizontal
        };
        read_index = 16;
        let mut trainer = Option::None;
        if header.flag_6 & (HeaderFlag6::TrainerPresent as u8) == 1 {
            let mut trainer_data: Trainer = [0; 512];
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
            let mut chr_rom_unit: ChrRomUnit = [0; common::CHR_ROM_UNIT_SIZE];
            read_index += read_to_array(&mut chr_rom_unit, &in_bytes[read_index..]);
            chr_rom.push(chr_rom_unit);
        }

        let mut play_choice_rom = Option::None;

        if header.flag_7 & (HeaderFlag7::PlayChoice10 as u8) == 1 {
            let mut inst_rom: PlayChoiceInstRom = [0; 8192];
            read_index += read_to_array(&mut inst_rom, &in_bytes[read_index..]);

            let mut data_output: PlayChoiceDecryptData = [0; 16];
            read_index += read_to_array(&mut data_output, &in_bytes[read_index..]);

            let mut counter_output: PlayChoiceDecryptData = [0; 16];
            read_to_array(&mut counter_output, &in_bytes[read_index..]);

            play_choice_rom = Some(PlayChoiceRom {
                _inst_rom: inst_rom,
                _data_output: data_output,
                _counter_output: counter_output,
            });
        }

        let mut prg_ram_size: u32 = 0;
        if header.flag_10 & (HeaderFlag10::PrgRAMPresent as u8) == 0 {
            prg_ram_size = common::PRG_RAM_UNIT_SIZE as u32 * (header.prg_ram_units as u32);
            if prg_ram_size == 0 {
                prg_ram_size = common::PRG_RAM_UNIT_SIZE as u32;
            }
        }

        let ho_n_mapper_number = if in_bytes[12] as u32
            + in_bytes[13] as u32
            + in_bytes[14] as u32
            + in_bytes[15] as u32
            != 0
        {
            0
        } else {
            header.ho_n_mapper_number as u32
        };

        let mapper_number = (ho_n_mapper_number << 4) + header.lo_n_mapper_number as u32;
        NesFile {
            _trainer: trainer,
            prg_rom,
            chr_rom,
            _play_choice_rom: play_choice_rom,
            _prg_ram_size: prg_ram_size,
            mapper_number,
            mirroring,
        }
    }
}
