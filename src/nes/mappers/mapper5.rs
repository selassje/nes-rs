use super::Mapper;
use super::mapper_internal::BankSize;
use super::mapper_internal::MapperInternal;
use crate::nes::apu::DUTY_CYCLE_SEQUENCES;
use crate::nes::apu::Envelope;
use crate::nes::apu::FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES;
use crate::nes::apu::FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES;
use crate::nes::apu::FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES;
use crate::nes::apu::FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES;
use crate::nes::apu::LengthCounterChannel;
use crate::nes::apu::StatusRegister;
use crate::nes::apu::StatusRegisterFlag::{Pulse1Enabled, Pulse2Enabled};

use crate::nes::common::Mirroring;
use crate::nes::common::NametableSource;
use crate::nes::mappers::PRG_RAM_RANGE;
use crate::nes::mappers::PRG_RANGE;
use crate::nes::ram_ppu::ReadAccessRegister;
use crate::nes::ram_ppu::WriteAccessRegister;

use BankSize::*;

use serde::{Deserialize, Serialize};
use serde_arrays;

const PULSE_REGISTER_1: u16 = 0x5000;
const PULSE_REGISTER_8: u16 = 0x5007;
const AUDIO_STATUS_REGISTER: u16 = 0x5015;
const PCM_MODE_REGISTER: u16 = 0x5010;
const PCM_RAW_REGISTER: u16 = 0x5011;
const PRG_MODE_SELECTION_REGISTER: u16 = 0x5100;
const CHR_MODE_SELECTION_REGISTER: u16 = 0x5101;
const PRG_RAM_PROTECT_REGISTER_1: u16 = 0x5102;
const PRG_RAM_PROTECT_REGISTER_2: u16 = 0x5103;
const EXTENDED_RAM_MODE_REGISTER: u16 = 0x5104;
const NAMETABLE_MAPPING_REGISTER: u16 = 0x5105;
const FILL_MODE_TILE_REGISTER: u16 = 0x5106;
const FILL_MODE_COLOR_REGISTER: u16 = 0x5107;
const PRG_BANK_REGISTER_1: u16 = 0x5113;
const PRG_BANK_REGISTER_5: u16 = 0x5117;
const CHR_BANK_REGISTER_1: u16 = 0x5120;
const CHR_BANK_REGISTER_12: u16 = 0x512B;
const UPPER_CHR_BITS_REGISTER: u16 = 0x5130;
const SPLIT_MODE_CONTROL_REGISTER: u16 = 0x5200;
const SPLIT_MODE_SCROLL_REGISTER: u16 = 0x5201;
const SPLIT_MODE_BANK_REGISTER: u16 = 0x5202;
const IRQ_SCANLINE_COMPARE_REGISTER: u16 = 0x5203;
const IRQ_SCANLINE_STATUS_REGISTER: u16 = 0x5204;
const MULTIPLIER_A_REGISTER: u16 = 0x5205;
const MULTIPLIER_B_REGISTER: u16 = 0x5206;
const EXPANSION_RAM_START: u16 = 0x5C00;
const EXPANSION_RAM_END: u16 = 0x5FFF;
#[derive(Serialize, Deserialize)]
struct PulseWave {
    data: [u8; 4],
    length_counter: u8,
    sequencer_position: u8,
    timer_tick: u16,
    envelope: Envelope,
    current_period: u16,
}

impl PulseWave {
    fn new() -> Self {
        PulseWave {
            data: [0; 4],
            length_counter: 0,
            timer_tick: 0,
            sequencer_position: 0,
            envelope: Envelope::default(),
            current_period: 0,
        }
    }

    fn power_cycle(&mut self) {
        self.data = [0; 4];
        self.length_counter = 0;
        self.timer_tick = 0;
        self.sequencer_position = 0;
        self.current_period = 0;
        self.envelope = Envelope::default();
    }
    fn update_period(&mut self) {
        self.current_period = self.get_raw_timer_period();
    }

    fn reset_phase(&mut self) {
        self.sequencer_position = 0;
        self.envelope.start_flag = true;
    }

    fn get_duty_cycle(&self) -> u8 {
        (self.data[0] & 0b11000000) >> 6
    }

    fn is_length_counter_halt_envelope_loop_flag_set(&self) -> bool {
        (self.data[0] & 0b00100000) != 0
    }

    fn is_constant_volume_set(&self) -> bool {
        (self.data[0] & 0b00010000) != 0
    }

    fn get_constant_volume_or_envelope_divider_reload_value(&self) -> u8 {
        self.data[0] & 0x0F
    }

    fn get_raw_timer_period(&self) -> u16 {
        let timer_hi = ((self.data[3] & 0x7) as u16) << 8;
        self.data[2] as u16 + timer_hi
    }

    fn clock_timer(&mut self) {
        if self.timer_tick == 0 {
            if self.sequencer_position > 0 {
                self.sequencer_position -= 1;
            } else {
                self.sequencer_position = 7;
            }
            self.timer_tick = self.current_period;
        } else {
            self.timer_tick -= 1;
        }
    }

    fn get_sample(&self) -> u8 {
        DUTY_CYCLE_SEQUENCES[self.get_duty_cycle() as usize][self.sequencer_position as usize]
            * self.get_volume()
    }

    fn get_volume(&self) -> u8 {
        if self.length_counter == 0 {
            0
        } else if self.is_constant_volume_set() {
            self.get_constant_volume_or_envelope_divider_reload_value()
        } else {
            self.envelope.decay_level_counter
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock(
            self.get_constant_volume_or_envelope_divider_reload_value(),
            self.is_length_counter_halt_envelope_loop_flag_set(),
        )
    }

    fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.is_length_counter_halt_envelope_loop_flag_set() {
            self.length_counter -= 1;
        }
    }
}

impl LengthCounterChannel for PulseWave {
    fn get_length_counter_load(&self) -> u8 {
        (self.data[3] & 0b11111000) >> 3
    }

    fn set_length_counter(&mut self, counter: u8) {
        self.length_counter = counter
    }

    fn get_length_counter(&self) -> u8 {
        self.length_counter
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
enum FetchMode {
    Cpu,
    Background,
    Sprites,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CpuExRamAccess {
    None = 0,
    Read = 1,
    Write = 2,
    ReadWrite = 3,
}

#[derive(Serialize, Deserialize)]
pub struct Mapper5 {
    mapper_internal: MapperInternal,
    prg_selection_mode: u8,
    chr_selection_mode: u8,
    prg_ram_protect_1: u8,
    prg_ram_protect_2: u8,
    extended_ram_mode: u8,
    prg_bank_registers: [u8; 5],
    chr_bank_registers: [u8; 12],
    chr_bank_upper_bits: u8,
    fill_mode_tile: u8,
    fill_mode_color: u8,
    split_mode_control: u8,
    split_mode_scroll: u8,
    split_mode_bank: u8,
    scanline_compare_value: u8,
    scanline_counter: u8,
    scanline_irq_enabled: bool,
    scanline_irq_pending: bool,
    in_frame: bool,
    nametable_mapping: u8,
    #[serde(with = "serde_arrays")]
    expansion_ram: [u8; 1024],
    is_sprite_mode_8x16_enabled: bool,
    are_ext_features_enabled: bool,
    fetch_mode: FetchMode,
    use_ext_as_default_for_8x16_sprite_mode: bool,
    attr_tile_index: u16,
    vertical_split_tile_index: u8,
    multiplier_a: u8,
    multiplier_b: u8,
    pulse_1: PulseWave,
    pulse_2: PulseWave,
    cpu_cycle: u16,
    audio_status_register: StatusRegister,
    pcm_mode_register: u8,
    raw_pcm: u8,
    pcm_irq_pending: bool,
}

impl Mapper5 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mapper_internal = MapperInternal::new(prg_rom, chr_rom);
        Self {
            mapper_internal,
            prg_selection_mode: 3,
            chr_selection_mode: 3,
            prg_ram_protect_1: 0,
            prg_ram_protect_2: 0,
            extended_ram_mode: 0,
            prg_bank_registers: [0x00, 0xFF, 0xFF, 0xFF, 0xFF],
            chr_bank_registers: [0x00; 12],
            chr_bank_upper_bits: 0,
            fill_mode_tile: 0,
            fill_mode_color: 0,
            split_mode_control: 0,
            split_mode_scroll: 0,
            split_mode_bank: 0,
            scanline_compare_value: 0,
            scanline_counter: 0,
            scanline_irq_enabled: false,
            scanline_irq_pending: false,
            in_frame: false,
            nametable_mapping: 0,
            expansion_ram: [0; 1024],
            is_sprite_mode_8x16_enabled: false,
            are_ext_features_enabled: false,
            fetch_mode: FetchMode::Cpu,
            use_ext_as_default_for_8x16_sprite_mode: false,
            attr_tile_index: 0,
            vertical_split_tile_index: 0,
            multiplier_a: 0xFF,
            multiplier_b: 0xFF,
            cpu_cycle: 8,
            pulse_1: PulseWave::new(),
            pulse_2: PulseWave::new(),
            audio_status_register: StatusRegister { data: 0 },
            pcm_mode_register: 0,
            raw_pcm: 0,
            pcm_irq_pending: false,
        }
    }

    fn get_prg_bank_register_index_and_size(&self, address: u16) -> (usize, BankSize) {
        let index_8_kb = (address - PRG_RAM_RANGE.start) / _8KB as u16;
        const INDEX_AND_MODE_TO_REGISTER_AND_SIZE: [[(usize, BankSize); 5]; 4] = [
            [(0, _8KB), (4, _32KB), (4, _32KB), (4, _32KB), (4, _32KB)],
            [(0, _8KB), (2, _16KB), (2, _16KB), (4, _16KB), (4, _16KB)],
            [(0, _8KB), (2, _16KB), (2, _16KB), (3, _8KB), (4, _8KB)],
            [(0, _8KB), (1, _8KB), (2, _8KB), (3, _8KB), (4, _8KB)],
        ];
        INDEX_AND_MODE_TO_REGISTER_AND_SIZE[self.prg_selection_mode as usize][index_8_kb as usize]
    }
    fn get_chr_bank_register_index_and_size(
        &self,
        address: u16,
        use_ext: bool,
    ) -> (usize, BankSize) {
        let index = (address / _1KB as u16) as usize;
        const INDEX_AND_MODE_TO_REGISTER: [[usize; 8]; 4] = [
            [7, 7, 7, 7, 7, 7, 7, 7],
            [3, 3, 3, 3, 7, 7, 7, 7],
            [1, 1, 3, 3, 5, 5, 7, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
        ];
        const INDEX_AND_MODE_TO_REGISTER_EXT: [[usize; 8]; 4] = [
            [11, 11, 11, 11, 11, 11, 11, 11],
            [11, 11, 11, 11, 11, 11, 11, 11],
            [9, 9, 11, 11, 9, 9, 11, 11],
            [8, 9, 10, 11, 8, 9, 10, 11],
        ];
        let mode = self.chr_selection_mode as usize;
        const MODE_TO_SIZE: [BankSize; 4] = [_8KB, _4KB, _2KB, _1KB];
        let register_index = if use_ext {
            INDEX_AND_MODE_TO_REGISTER_EXT[mode][index]
        } else {
            INDEX_AND_MODE_TO_REGISTER[mode][index]
        };
        (register_index, MODE_TO_SIZE[mode])
    }

    fn decode_prg_bank_register(&self, index: u8, bank_size: BankSize) -> (usize, bool) {
        let byte = self.prg_bank_registers[index as usize];
        let mut is_rom = (byte & 0b1000_0000) != 0;
        let mut bank = (byte & 0b0111_1111) as usize;

        if index == 0 {
            bank &= 0b0000_1111;
            is_rom = false;
        }
        if index == 4 {
            is_rom = true;
        }
        if bank_size == _16KB {
            bank = ((byte & 0b0111_1110) >> 1) as usize;
        }
        if bank_size == _32KB {
            bank = ((byte & 0b0111_1100) >> 2) as usize;
        }
        (bank, is_rom)
    }

    fn is_prg_ram_writable(&self) -> bool {
        (self.prg_ram_protect_1 & 0b11) == 0b10 && (self.prg_ram_protect_2 & 0b11) == 0b01
    }

    fn is_rendering(&self) -> bool {
        self.fetch_mode != FetchMode::Cpu
    }
    fn get_cpu_ex_ram_access_mode(&self) -> CpuExRamAccess {
        const CPU_EXRAM_ACCESS_MODE_DURING_BLANKING: [CpuExRamAccess; 4] = [
            CpuExRamAccess::None,
            CpuExRamAccess::None,
            CpuExRamAccess::ReadWrite,
            CpuExRamAccess::Read,
        ];

        const CPU_EXRAM_ACCESS_MODE_DURING_RENDERING: [CpuExRamAccess; 4] = [
            CpuExRamAccess::Write,
            CpuExRamAccess::Write,
            CpuExRamAccess::ReadWrite,
            CpuExRamAccess::Read,
        ];
        if self.is_rendering() {
            CPU_EXRAM_ACCESS_MODE_DURING_RENDERING[self.extended_ram_mode as usize]
        } else {
            CPU_EXRAM_ACCESS_MODE_DURING_BLANKING[self.extended_ram_mode as usize]
        }
    }
    fn is_in_split_region(&self) -> bool {
        if self.are_ext_features_enabled
            && self.split_mode_control & 0b1000_0000 != 0
            && self.extended_ram_mode < 2
        {
            let right_side = self.split_mode_control & 0b0100_0000 != 0;
            let split_threshold = self.split_mode_control & 0b0001_1111;
            let effective_tile = self.vertical_split_tile_index;
            if (right_side && effective_tile >= split_threshold)
                || (!right_side && effective_tile < split_threshold)
            {
                return true;
            }
        }
        false
    }
    fn is_half_frame_reached(&self) -> bool {
        self.cpu_cycle == FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES
            || self.cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
    }

    fn is_quarter_frame_reached(&self) -> bool {
        self.is_half_frame_reached()
            || self.cpu_cycle == FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES
            || self.cpu_cycle == FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES
    }
}

impl Mapper for Mapper5 {
    fn get_chr_byte(&mut self, address: u16) -> u8 {
        let is_ext_bg = self.are_ext_features_enabled
            && self.fetch_mode == FetchMode::Background
            && (self.extended_ram_mode == 1 || self.is_in_split_region());

        if is_ext_bg {
            let in_split = self.is_in_split_region();
            let exram_byte = self.expansion_ram[self.attr_tile_index as usize];
            let bank = if in_split {
                ((self.chr_bank_upper_bits as usize) << 8) | self.split_mode_bank as usize
            } else {
                ((self.chr_bank_upper_bits as usize & 0x03) << 6)
                    | (exram_byte & 0b0011_1111) as usize
            };
            return self.mapper_internal.get_chr_byte(address, bank, _4KB);
        }

        let is_sprite_mode_8x16 = self.are_ext_features_enabled && self.is_sprite_mode_8x16_enabled;
        let use_ext = is_sprite_mode_8x16
            && match self.fetch_mode {
                FetchMode::Background => true,
                FetchMode::Sprites => false,
                FetchMode::Cpu => self.use_ext_as_default_for_8x16_sprite_mode,
            };

        let (register, bank_size) = self.get_chr_bank_register_index_and_size(address, use_ext);
        let bank =
            ((self.chr_bank_upper_bits as usize) << 8) | self.chr_bank_registers[register] as usize;

        self.mapper_internal.get_chr_byte(address, bank, bank_size)
    }

    fn store_chr_byte(&mut self, address: u16, byte: u8) {
        let (register, bank_size) = self.get_chr_bank_register_index_and_size(address, false);
        let bank =
            ((self.chr_bank_upper_bits as usize) << 8) | self.chr_bank_registers[register] as usize;
        self.mapper_internal
            .store_chr_byte(address, bank, bank_size, byte);
    }

    fn get_prg_byte(&mut self, address: u16) -> u8 {
        match address {
            IRQ_SCANLINE_STATUS_REGISTER => {
                let mut byte: u8 = 0;
                if self.scanline_irq_pending {
                    byte |= 0b1000_0000;
                }
                if self.in_frame {
                    byte |= 0b0100_0000
                }
                self.scanline_irq_pending = false;
                byte
            }
            EXPANSION_RAM_START..=EXPANSION_RAM_END => {
                let access_mode = self.get_cpu_ex_ram_access_mode();
                if access_mode == CpuExRamAccess::Read || access_mode == CpuExRamAccess::ReadWrite {
                    let index = (address - EXPANSION_RAM_START) as usize;
                    self.expansion_ram[index]
                } else {
                    0
                }
            }
            MULTIPLIER_A_REGISTER => {
                let result = self.multiplier_a as u16 * self.multiplier_b as u16;
                result as u8
            }
            MULTIPLIER_B_REGISTER => {
                let result = self.multiplier_a as u16 * self.multiplier_b as u16;
                (result >> 8) as u8
            }
            AUDIO_STATUS_REGISTER => {
                let mut out = StatusRegister { data: 0 };
                out.set_flag_status(Pulse1Enabled, self.pulse_1.length_counter > 0);
                out.set_flag_status(Pulse2Enabled, self.pulse_2.length_counter > 0);
                out.data
            }
            PCM_MODE_REGISTER => {
                let mut out = self.pcm_mode_register & 1;
                let irq_enabled = self.pcm_mode_register & 0b1000_0000 != 0;
                if irq_enabled && self.pcm_irq_pending {
                    out |= 0b1000_0000;
                }
                self.pcm_irq_pending = false;
                out
            }
            0x5016..=0x5BFF => 0,
            0x4020..=0x4FFF => 0,
            address if PRG_RANGE.contains(&address) => {
                if address == 0xFFFA || address == 0xFFFB {
                    self.in_frame = false;
                    self.scanline_irq_pending = false;
                    self.scanline_counter = 0;
                }
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let (bank, is_rom) = self.decode_prg_bank_register(index as u8, bank_size);
                let byte = if is_rom {
                    self.mapper_internal
                        .get_prg_rom_byte(address, bank, bank_size)
                } else {
                    self.mapper_internal
                        .get_prg_ram_byte(address, bank, bank_size)
                };
                if (0x8000..=0xBFFF).contains(&address) && self.pcm_mode_register & 1 == 1 {
                    if byte == 0 {
                        self.pcm_irq_pending = true;
                    } else {
                        self.pcm_irq_pending = false;
                        self.raw_pcm = byte;
                    }
                }
                byte
            }
            _ => {
                println!("Get prg byte : Unknown address ${:04X}", address);
                0
            }
        }
    }

    fn store_prg_byte(&mut self, address: u16, byte: u8) {
        match address {
            PCM_MODE_REGISTER => {
                self.pcm_mode_register = byte;
            }
            PCM_RAW_REGISTER => {
                let write_mode = self.pcm_mode_register & 1 == 0;
                if write_mode {
                    if byte == 0 {
                        self.pcm_irq_pending = true;
                    } else {
                        self.pcm_irq_pending = false;
                        self.raw_pcm = byte;
                    }
                }
            }
            PRG_MODE_SELECTION_REGISTER => {
                self.prg_selection_mode = byte & 0b11;
            }
            CHR_MODE_SELECTION_REGISTER => {
                self.chr_selection_mode = byte & 0b11;
            }
            PRG_RAM_PROTECT_REGISTER_1 => {
                self.prg_ram_protect_1 = byte;
            }
            PRG_RAM_PROTECT_REGISTER_2 => {
                self.prg_ram_protect_2 = byte;
            }
            EXTENDED_RAM_MODE_REGISTER => {
                self.extended_ram_mode = byte & 0b11;
            }
            NAMETABLE_MAPPING_REGISTER => {
                self.nametable_mapping = byte;
            }
            FILL_MODE_TILE_REGISTER => {
                self.fill_mode_tile = byte;
            }
            FILL_MODE_COLOR_REGISTER => {
                self.fill_mode_color = byte;
            }
            UPPER_CHR_BITS_REGISTER => {
                self.chr_bank_upper_bits = byte;
            }
            PRG_BANK_REGISTER_1..=PRG_BANK_REGISTER_5 => {
                let index = (address - PRG_BANK_REGISTER_1) as usize;
                self.prg_bank_registers[index] = byte;
            }
            CHR_BANK_REGISTER_1..=CHR_BANK_REGISTER_12 => {
                let index = (address - CHR_BANK_REGISTER_1) as usize;
                self.use_ext_as_default_for_8x16_sprite_mode = index > 7;
                self.chr_bank_registers[index] = byte;
            }
            SPLIT_MODE_CONTROL_REGISTER => {
                self.split_mode_control = byte;
            }
            SPLIT_MODE_SCROLL_REGISTER => {
                self.split_mode_scroll = byte;
            }
            SPLIT_MODE_BANK_REGISTER => {
                self.split_mode_bank = byte;
            }
            IRQ_SCANLINE_COMPARE_REGISTER => {
                self.scanline_compare_value = byte;
            }
            IRQ_SCANLINE_STATUS_REGISTER => {
                self.scanline_irq_enabled = byte & 0b1000_0000 != 0;
            }
            MULTIPLIER_A_REGISTER => {
                self.multiplier_a = byte;
            }
            MULTIPLIER_B_REGISTER => {
                self.multiplier_b = byte;
            }
            EXPANSION_RAM_START..=EXPANSION_RAM_END => {
                let access_mode = self.get_cpu_ex_ram_access_mode();
                let index = (address - EXPANSION_RAM_START) as usize;
                if access_mode == CpuExRamAccess::Write || access_mode == CpuExRamAccess::ReadWrite
                {
                    self.expansion_ram[index] = byte;
                }
            }
            AUDIO_STATUS_REGISTER => {
                self.audio_status_register.data = byte;
                if !self.audio_status_register.is_flag_enabled(Pulse1Enabled) {
                    self.pulse_1.reset_length_counter();
                }
                if !self.audio_status_register.is_flag_enabled(Pulse2Enabled) {
                    self.pulse_2.reset_length_counter();
                }
            }

            PULSE_REGISTER_1..=PULSE_REGISTER_8 => {
                let index = (address - PULSE_REGISTER_1) as usize;
                let (pulse, status_flag) = if index < 4 {
                    (&mut self.pulse_1, Pulse1Enabled)
                } else {
                    (&mut self.pulse_2, Pulse2Enabled)
                };
                match index % 4 {
                    0 => pulse.data[0] = byte,
                    1 => pulse.data[1] = byte,
                    2 => {
                        pulse.data[2] = byte;
                        pulse.update_period();
                    }
                    3 => {
                        pulse.data[3] = byte;
                        pulse.update_period();
                        pulse.reset_phase();
                        if self.audio_status_register.is_flag_enabled(status_flag) {
                            pulse.reload_length_counter();
                        }
                    }
                    _ => unreachable!(),
                }
            }
            0x5016..=0x5BFF => {}
            address if PRG_RANGE.contains(&address) => {
                let (index, bank_size) = self.get_prg_bank_register_index_and_size(address);
                let (bank, is_rom) = self.decode_prg_bank_register(index as u8, bank_size);
                if self.is_prg_ram_writable() && !is_rom {
                    self.mapper_internal
                        .store_prg_ram_byte(address, bank, bank_size, byte);
                }
            }
            _ => {
                println!("Store prg byte: Unknown address ${:04X}", address)
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        let mut tables = [NametableSource::Vram0; 4];
        for nametable in 0..4 {
            let mask = 0b0000_0011 << (nametable * 2);
            let nametable_source = (self.nametable_mapping & mask) >> (nametable * 2);
            tables[nametable as usize] = NametableSource::try_from(nametable_source).unwrap();
        }
        Mirroring { tables }
    }

    fn power_cycle(&mut self) {
        self.prg_selection_mode = 3;
        self.chr_selection_mode = 3;
        self.prg_ram_protect_1 = 0;
        self.prg_ram_protect_2 = 0;
        self.extended_ram_mode = 0;
        self.prg_bank_registers = [0x0, 0xFF, 0xFF, 0xFF, 0xFF];
        self.chr_bank_registers = [0xFF; 12];
        self.chr_bank_upper_bits = 0;
        self.fill_mode_tile = 0;
        self.fill_mode_color = 0;
        self.split_mode_control = 0;
        self.split_mode_scroll = 0;
        self.split_mode_bank = 0;
        self.scanline_compare_value = 0;
        self.scanline_counter = 0;
        self.scanline_irq_enabled = false;
        self.scanline_irq_pending = false;
        self.in_frame = false;
        self.nametable_mapping = 0;
        self.expansion_ram = [0; 1024];
        self.is_sprite_mode_8x16_enabled = false;
        self.are_ext_features_enabled = false;
        self.fetch_mode = FetchMode::Cpu;
        self.use_ext_as_default_for_8x16_sprite_mode = false;
        self.attr_tile_index = 0;
        self.vertical_split_tile_index = 0;
        self.multiplier_a = 0xFF;
        self.multiplier_b = 0xFF;
        self.cpu_cycle = 8;
        self.audio_status_register.data = 0;
        self.pcm_mode_register = 0;
        self.raw_pcm = 0;
        self.pcm_irq_pending = false;
        self.pulse_1.power_cycle();
        self.pulse_2.power_cycle();
        self.mapper_internal.power_cycle();
    }

    fn notify_scanline(&mut self) {
        if !self.in_frame {
            self.in_frame = true;
            self.scanline_counter = 0;
        } else {
            self.scanline_counter += 1;
        }
        if self.scanline_counter == self.scanline_compare_value && self.scanline_compare_value != 0
        {
            self.scanline_irq_pending = true;
        }
    }

    fn is_irq_pending(&self) -> bool {
        (self.scanline_irq_enabled && self.scanline_irq_pending)
            || (self.pcm_mode_register & 0b1000_0000 != 0 && self.pcm_irq_pending)
    }

    fn get_nametable_byte(&self, source: NametableSource, offset: u16) -> Option<u8> {
        if self.is_in_split_region() {
            let effective_y = (self.split_mode_scroll as u16 + self.scanline_counter as u16) % 240;
            let coarse_y = effective_y / 8;
            let tile_x = self.vertical_split_tile_index;
            let index = (coarse_y as usize * 32) + tile_x as usize;
            return Some(self.expansion_ram[index]);
        }
        match source {
            NametableSource::ExRam => {
                if !self.is_rendering() && self.extended_ram_mode > 1 {
                    return Some(0);
                }
                let index = (offset & 0x3FF) as usize;
                Some(self.expansion_ram[index])
            }
            NametableSource::Fill => {
                if offset & 0x3FF < 0x3C0 {
                    Some(self.fill_mode_tile)
                } else {
                    let color = self.fill_mode_color & 0x03;
                    Some(color | (color << 2) | (color << 4) | (color << 6))
                }
            }
            _ => None,
        }
    }

    fn store_nametable_byte(&mut self, source: NametableSource, offset: u16, byte: u8) -> bool {
        let index = (offset & 0x3FF) as usize;
        match source {
            NametableSource::ExRam => {
                self.expansion_ram[index] = byte;
                true
            }
            NametableSource::Fill => true,
            NametableSource::Vram0 | NametableSource::Vram1 => false,
        }
    }

    fn notify_oam_dma_write(&mut self) {
        self.scanline_counter = 0;
    }

    fn notify_ppu_register_write(&mut self, address: u16, value: u8) {
        if let Ok(register) = WriteAccessRegister::try_from(address) {
            match register {
                WriteAccessRegister::PpuCtrl => {
                    self.is_sprite_mode_8x16_enabled = value & 0b0010_0000 != 0;
                }
                WriteAccessRegister::PpuMask => {
                    self.are_ext_features_enabled = value & 0b0001_1000 != 0;
                }
                WriteAccessRegister::PpuData => {
                    self.fetch_mode = FetchMode::Cpu;
                }
                _ => {}
            }
        }
    }

    fn notify_ppu_register_read(&mut self, address: u16) {
        if let Ok(register) = ReadAccessRegister::try_from(address)
            && register == ReadAccessRegister::PpuData
        {
            self.fetch_mode = FetchMode::Cpu;
        }
    }
    fn notify_background_tile_data_prefetch_start(&mut self) {
        if self.in_frame && self.scanline_counter >= 239 {
            self.in_frame = false;
            self.scanline_counter = 0;
        }
        self.vertical_split_tile_index = 0;
    }

    fn notify_background_tile_data_fetch_complete(&mut self) {
        self.vertical_split_tile_index = (self.vertical_split_tile_index + 1) & 0x1F;
    }

    fn notify_background_pattern_data_fetch(&mut self) {
        self.fetch_mode = FetchMode::Background;
    }

    fn notify_sprite_pattern_data_fetch(&mut self) {
        self.fetch_mode = FetchMode::Sprites;
    }

    fn get_background_palette_index(&mut self, tile_x: u8, tile_y: u8) -> Option<u8> {
        if !self.are_ext_features_enabled {
            return None;
        }

        if self.is_in_split_region() {
            let effective_y = (self.split_mode_scroll as u16 + self.scanline_counter as u16) % 240;
            let coarse_y = (effective_y / 8) as u8;
            let split_tile_x = self.vertical_split_tile_index;
            let attr_x = split_tile_x / 4;
            let attr_y = coarse_y / 4;
            let attr_index = 960 + (attr_y as usize * 8) + attr_x as usize;
            let attr_byte = self.expansion_ram[attr_index];
            let quadrant = ((coarse_y >> 1) & 1) * 2 + ((split_tile_x >> 1) & 1);
            let palette = (attr_byte >> (quadrant * 2)) & 0b11;
            return Some(palette);
        }

        if self.extended_ram_mode != 1 {
            return None;
        }
        self.attr_tile_index = tile_y as u16 * 32 + tile_x as u16;
        let exram_byte = self.expansion_ram[self.attr_tile_index as usize];
        let palette = (exram_byte & 0b1100_0000) >> 6;
        Some(palette)
    }

    fn clock_audio(&mut self) -> Option<f32> {
        self.pulse_1.clock_timer();
        self.pulse_2.clock_timer();
        if self.is_quarter_frame_reached() {
            self.pulse_1.clock_length_counter();
            self.pulse_2.clock_length_counter();
            self.pulse_1.clock_envelope();
            self.pulse_2.clock_envelope();
        }
        let n = self.pulse_1.get_sample() + self.pulse_2.get_sample();
        let pulse_out = if n != 0 {
            95.52 / ((8128.0 / (n as f32)) + 100.0)
        } else {
            0.0
        };
        let pcm_out = self.raw_pcm as f32 / 256.0;
        self.cpu_cycle = (self.cpu_cycle + 1) % (FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES + 1);
        Some(pulse_out + pcm_out)
    }
}
