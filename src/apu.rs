use self::StatusRegisterFlag::*;
use crate::{io::AudioAccess, memory::DmcMemory, ram_apu::*};
use std::{cell::RefCell, default::Default, rc::Rc};

use crate::io::SampleFormat;

const LENGTH_COUNTER_LOOKUP_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_CYCLE_SEQUENCES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

const DMC_RATES_NTSC: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

const NOISE_PERIOD_NTSC: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

trait LengthCounterChannel {
    fn get_length_counter_load(&self) -> u8;
    fn get_length_counter(&self) -> u8;
    fn set_length_counter(&mut self, value: u8);

    fn reload_length_counter(&mut self) {
        self.set_length_counter(
            LENGTH_COUNTER_LOOKUP_TABLE[self.get_length_counter_load() as usize],
        );
    }
    fn reset_length_counter(&mut self) {
        self.set_length_counter(0);
    }
}

const FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES: u16 = 7457;
const FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES: u16 = 14913;
const FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES: u16 = 22371;
const FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES: u16 = 29829;
const FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES: u16 = 37281;

struct FrameCounter {
    data: u8,
}

impl FrameCounter {
    fn get_sequencer_mode(&self) -> u8 {
        (self.data & 0b10000000) >> 7
    }

    fn is_interrupt_inhibit_flag_set(&self) -> bool {
        self.data & 0b01000000 != 0
    }
}

#[derive(Copy, Clone)]
enum StatusRegisterFlag {
    Pulse1Enabled = 0b00000001,
    Pulse2Enabled = 0b00000010,
    TriangleEnabled = 0b00000100,
    NoiseEnabled = 0b00001000,
    DMCEnabled = 0b00010000,
    FrameInterrupt = 0b01000000,
    DMCInterrupt = 0b10000000,
}

#[derive(Default)]
struct StatusRegister {
    data: u8,
}

impl StatusRegister {
    fn is_flag_enabled(&self, flag: StatusRegisterFlag) -> bool {
        let flag = flag as u8;
        assert!(flag >= Pulse1Enabled as u8 && flag <= DMCEnabled as u8);
        self.data & flag != 0
    }

    fn set_flag_status(&mut self, flag: StatusRegisterFlag, is_enabled: bool) {
        let flag = flag as u8;
        if is_enabled {
            self.data |= flag
        } else {
            self.data &= !flag
        }
    }
}

#[derive(Default)]
struct Envelope {
    start_flag: bool,
    divider: u8,
    decay_level_counter: u8,
}

impl Envelope {
    fn clock(&mut self, divider_reload_value: u8, loop_flag: bool) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_level_counter = 15;
            self.divider = divider_reload_value;
        } else {
            if self.divider > 0 {
                self.divider -= 1;
            } else {
                self.divider = divider_reload_value;
                if self.decay_level_counter > 0 {
                    self.decay_level_counter -= 1;
                } else if loop_flag {
                    self.decay_level_counter = 15;
                }
            }
        }
    }
}

struct SweepUnit {
    reload_flag: bool,
    divider: u8,
    use_ones_complement: bool,
    is_muting: bool,
}

impl SweepUnit {
    fn new(use_ones_complement: bool) -> Self {
        SweepUnit {
            reload_flag: false,
            divider: 0,
            use_ones_complement,
            is_muting: false,
        }
    }

    fn get_target_period(
        &self,
        raw_period: u16,
        current_period: u16,
        shift: u8,
        negate: bool,
    ) -> u16 {
        let change_amount = raw_period >> shift;
        if negate {
            if self.use_ones_complement {
                current_period - change_amount - 1
            } else {
                current_period - change_amount
            }
        } else {
            current_period + change_amount
        }
    }

    fn update_muting_status(&mut self, target_period: u16, current_period: u16) {
        self.is_muting = target_period > 0x7FF || current_period < 8;
    }

    fn clock(&mut self, sweep_enabled: bool, sweep_period: u8) -> bool {
        let adjust = self.divider == 0 && sweep_enabled && !self.is_muting;
        if self.divider == 0 || self.reload_flag {
            self.divider = sweep_period;
            self.reload_flag = false;
        } else {
            self.divider -= 1;
        }
        adjust
    }
}

struct PulseWave {
    data: [u8; 4],
    length_counter: u8,
    sequencer_position: u8,
    timer_tick: u16,
    envelope: Envelope,
    sweep_unit: SweepUnit,
    current_period: u16,
}

impl PulseWave {
    fn new(use_ones_complement_for_sweep_unit: bool) -> Self {
        PulseWave {
            data: [0; 4],
            length_counter: 0,
            timer_tick: 0,
            current_period: 0,
            sequencer_position: 0,
            envelope: Envelope::default(),
            sweep_unit: SweepUnit::new(use_ones_complement_for_sweep_unit),
        }
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

    fn is_sweep_unit_enabled(&self) -> bool {
        (self.data[1] & 0b10000000) != 0
    }

    fn get_sweep_period(&self) -> u8 {
        (self.data[1] & 0b01110000) >> 4
    }

    fn is_sweep_negate_enabled(&self) -> bool {
        (self.data[1] & 0b00001000) != 0
    }

    fn get_sweep_shift(&self) -> u8 {
        self.data[1] & 0x7
    }

    fn get_raw_timer_period(&self) -> u16 {
        let timer_hi = ((self.data[3] & 0x7) as u16) << 8;
        self.data[2] as u16 + timer_hi
    }

    fn reset(&mut self) {
        self.sequencer_position = 0;
        self.current_period = self.get_raw_timer_period();
        self.envelope.start_flag = true;
        self.sweep_unit.reload_flag = true;
    }

    fn clock_timer(&mut self) {
        if self.timer_tick == 0 {
            if self.sequencer_position > 0 {
                self.sequencer_position -= 1;
            } else {
                self.sequencer_position = 7;
            }
            self.timer_tick = (2 * self.current_period) - 1;
        } else {
            self.timer_tick -= 1;
        }
    }

    fn get_sample(&self) -> u8 {
        DUTY_CYCLE_SEQUENCES[self.get_duty_cycle() as usize][self.sequencer_position as usize]
            * self.get_volume() as u8
    }

    fn get_volume(&self) -> u8 {
        if self.length_counter == 0 || self.sweep_unit.is_muting {
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

    fn clock_length_counter_and_sweep_unit(&mut self) {
        if self.length_counter > 0 && !self.is_length_counter_halt_envelope_loop_flag_set() {
            self.length_counter -= 1;
        }

        let target_period = self.sweep_unit.get_target_period(
            self.get_raw_timer_period(),
            self.current_period,
            self.get_sweep_shift(),
            self.is_sweep_negate_enabled(),
        );
        self.sweep_unit
            .update_muting_status(target_period, self.current_period);
        if self
            .sweep_unit
            .clock(self.is_sweep_unit_enabled(), self.get_sweep_period())
        {
            self.current_period = target_period;
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

#[derive(Default)]
struct TriangleWave {
    data: [u8; 4],
    length_counter: u8,
    timer_tick: u16,
    linear_counter: u8,
    linear_counter_reload_flag: bool,
    sequencer_position: usize,
}

impl TriangleWave {
    fn is_control_flag_set(&self) -> bool {
        (self.data[0] & 0b10000000) != 0
    }

    fn get_linear_counter_load(&self) -> u8 {
        self.data[0] & 0b01111111
    }

    fn get_timer(&self) -> u16 {
        let timer_hi = ((self.data[3] & 0x7) as u16) << 8;
        self.data[2] as u16 + timer_hi
    }

    fn clock_linear_counter(&mut self) {
        if self.linear_counter_reload_flag {
            self.linear_counter = self.get_linear_counter_load();
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.is_control_flag_set() {
            self.linear_counter_reload_flag = false;
        }
    }

    fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.is_control_flag_set() {
            self.length_counter -= 1;
        }
    }

    fn clock_timer(&mut self) {
        if self.length_counter > 0 && self.linear_counter > 0 {
            if self.timer_tick == 0 {
                self.sequencer_position = (1 + self.sequencer_position) % 32;
                self.timer_tick = self.get_timer();
            } else {
                self.timer_tick -= 1;
            }
        }
    }

    fn get_sample(&self) -> u8 {
        TRIANGLE_SEQUENCE[self.sequencer_position]
    }
}

impl LengthCounterChannel for TriangleWave {
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

struct Noise {
    data: [u8; 4],
    length_counter: u8,
    shift_register: u16,
    envelope: Envelope,
    timer_tick: u16,
}

impl Noise {
    fn new() -> Self {
        Noise {
            data: [0; 4],
            length_counter: 0,
            shift_register: 1,
            envelope: Default::default(),
            timer_tick: 0,
        }
    }

    fn reset(&mut self) {
        self.envelope.start_flag = true;
    }

    fn is_length_counter_halt_set(&self) -> bool {
        (self.data[0] & 0b00100000) != 0
    }

    fn is_constant_volume_set(&self) -> bool {
        (self.data[0] & 0b00010000) != 0
    }

    fn get_constant_volume_or_envelope_divider_reload_value(&self) -> u8 {
        self.data[0] & 0x0F
    }

    fn is_mode_flag_set(&self) -> bool {
        self.data[2] & 0b10000000 != 0
    }

    fn get_timer(&self) -> u16 {
        2 * NOISE_PERIOD_NTSC[(self.data[2] & 0x0F) as usize]
    }

    fn get_sample(&self) -> u8 {
        if self.length_counter == 0 || self.shift_register & 1 == 0 {
            0
        } else if self.is_constant_volume_set() {
            self.get_constant_volume_or_envelope_divider_reload_value()
        } else {
            self.envelope.decay_level_counter
        }
    }

    fn clock_timer(&mut self) {
        if self.timer_tick == 0 {
            let snd_xor_bit = if self.is_mode_flag_set() {
                (self.shift_register & 0b000000_01000000) >> 6
            } else {
                (self.shift_register & 0b000000_00000010) >> 1
            };
            let feedback_bit = (self.shift_register & 1) ^ snd_xor_bit;
            self.shift_register >>= 1;
            self.shift_register |= feedback_bit << 14;
            self.timer_tick = self.get_timer();
        } else {
            self.timer_tick -= 1;
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock(
            self.get_constant_volume_or_envelope_divider_reload_value(),
            self.is_length_counter_halt_set(),
        )
    }
    fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.is_length_counter_halt_set() {
            self.length_counter -= 1;
        }
    }
}

impl LengthCounterChannel for Noise {
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

struct DMC {
    data: [u8; 4],
    timer_tick: u16,
    sample_buffer: Option<u8>,
    shift_register: u8,
    silence_flag: bool,
    bits_counter: u8,
    bytes_remaining: u16,
    next_bytes_remaining: u16,
    output_value: u8,
    start_pending: bool,
    interrupt: bool,
    dmc_memory: Option<Rc<RefCell<dyn DmcMemory>>>,
}

impl DMC {
    fn new() -> Self {
        DMC {
            data: [0; 4],
            timer_tick: 0,
            bits_counter: 0,
            bytes_remaining: 0,
            next_bytes_remaining: 0,
            silence_flag: true,
            sample_buffer: None,
            shift_register: 0,
            output_value: 0,
            dmc_memory: None,
            start_pending: false,
            interrupt: false,
        }
    }

    fn start_sample(&mut self) {
        self.bytes_remaining = self.next_bytes_remaining;
        self.dmc_memory
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_sample_address(self.get_sample_address());
    }

    fn is_irq_enabled(&self) -> bool {
        (self.data[0] & 0b10000000) != 0
    }

    fn is_loop_enabled(&self) -> bool {
        (self.data[0] & 0b01000000) != 0
    }

    fn get_timer(&self) -> u16 {
        DMC_RATES_NTSC[(self.data[0] & 0x0F) as usize]
    }

    fn get_direct_load(&self) -> u8 {
        self.data[1] & 0b01111111
    }

    fn get_sample_address(&self) -> u8 {
        self.data[2]
    }

    fn get_sample_length(&self) -> u16 {
        (self.data[3] as u16 * 16) + 1
    }
    fn clock_timer(&mut self) {
        if self.timer_tick == 0 {
            if self.bits_counter == 0 {
                self.bits_counter = 8;
                self.silence_flag = if let Some(buffer) = self.sample_buffer.take() {
                    self.shift_register = buffer;
                    false
                } else {
                    true
                };
            }
            if !self.silence_flag {
                if self.shift_register & 1 == 1 {
                    if self.output_value <= 125 {
                        self.output_value += 2;
                    }
                } else if self.output_value >= 2 {
                    self.output_value -= 2;
                }
            }
            self.bits_counter -= 1;
            self.shift_register >>= 1;
            self.timer_tick = self.get_timer() - 1;
        } else {
            self.timer_tick -= 1;
        }
    }

    fn fetch_next_sample_buffer(&mut self) {
        if self.sample_buffer.is_none() {
            if self.bytes_remaining > 0 {
                self.sample_buffer = Some(
                    self.dmc_memory
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_next_sample_byte(),
                );
                self.bytes_remaining -= 1;
                if self.bytes_remaining == 0 {
                    if self.is_loop_enabled() {
                        self.next_bytes_remaining = self.get_sample_length();
                        self.start_sample();
                    } else if self.is_irq_enabled() {
                        self.interrupt = true;
                    }
                }
            } else if self.start_pending {
                self.start_sample();
                self.fetch_next_sample_buffer();
                self.start_pending = false;
            }
        }
    }
    fn get_sample(&self) -> u8 {
        self.output_value
    }
}

pub struct APU {
    audio_access: Rc<RefCell<dyn AudioAccess>>,
    frame_counter: FrameCounter,
    status: StatusRegister,
    pulse_1: PulseWave,
    pulse_2: PulseWave,
    triangle: TriangleWave,
    noise: Noise,
    dmc: DMC,
    cpu_cycle: u16,
    frame_interrupt: bool,
    frame: u128,
    pending_reset_cycle: Option<u16>,
    irq_flag_setting_in_progress: bool,
}

impl APU {
    pub fn new(audio_access: Rc<RefCell<dyn AudioAccess>>) -> Self {
        APU {
            frame_counter: FrameCounter { data: 0 },
            status: StatusRegister { data: 0 },
            pulse_1: PulseWave::new(false),
            pulse_2: PulseWave::new(true),
            triangle: TriangleWave::default(),
            noise: Noise::new(),
            dmc: DMC::new(),
            cpu_cycle: 0,
            frame_interrupt: false,
            audio_access,
            frame: 1,
            pending_reset_cycle: None,
            irq_flag_setting_in_progress: false,
        }
    }

    pub fn set_dmc_memory(&mut self, dmc_memory: Rc<RefCell<dyn DmcMemory>>) {
        self.dmc.dmc_memory = Some(dmc_memory);
    }

    fn get_length_counter_channel(
        &mut self,
        flag: StatusRegisterFlag,
    ) -> &mut dyn LengthCounterChannel {
        match flag {
            Pulse1Enabled => &mut self.pulse_1,
            Pulse2Enabled => &mut self.pulse_2,
            TriangleEnabled => &mut self.triangle,
            NoiseEnabled => &mut self.noise,
            _ => panic!("Incorrect status register flag {}", flag as u8),
        }
    }

    fn reload_length_counter_if_enabled(&mut self, flag: StatusRegisterFlag) {
        if self.status.is_flag_enabled(flag) {
            self.get_length_counter_channel(flag)
                .reload_length_counter();
        }
    }

    fn reset_length_counter_if_disabled(&mut self, flag: StatusRegisterFlag) {
        if !self.status.is_flag_enabled(flag) {
            self.get_length_counter_channel(flag).reset_length_counter();
        }
    }

    fn shifted_cpu_cycle(&mut self, shift: u16) -> u16 {
        let max = if self.frame_counter.get_sequencer_mode() == 0 {
            FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
        } else {
            FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES
        } + 1;
        (self.cpu_cycle + shift) % max
    }
    fn perform_half_frame_update(&mut self) {
        self.pulse_1.clock_length_counter_and_sweep_unit();
        self.pulse_2.clock_length_counter_and_sweep_unit();
        self.triangle.clock_length_counter();
        self.noise.clock_length_counter();
    }

    fn perform_quarter_frame_update(&mut self) {
        self.pulse_1.clock_envelope();
        self.pulse_2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    pub fn run_single_cpu_cycle(&mut self) {
        if let Some(pending_reset_cycle) = self.pending_reset_cycle {
            if pending_reset_cycle == self.cpu_cycle {
                self.cpu_cycle = 0;
                self.pending_reset_cycle = None;
                if self.frame_counter.get_sequencer_mode() == 1 {
                    self.perform_half_frame_update();
                    self.perform_quarter_frame_update();
                }
            }
        }
        if self.is_quarter_frame_reached() {
            self.perform_quarter_frame_update();
        }

        if self.is_half_frame_reached() {
            self.perform_half_frame_update();
        }

        self.pulse_1.clock_timer();
        self.pulse_2.clock_timer();
        self.triangle.clock_timer();
        self.noise.clock_timer();

        self.dmc.fetch_next_sample_buffer();
        self.dmc.clock_timer();

        if self.frame_counter.get_sequencer_mode() == 0
            && (self.cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES - 1
                || ((self.cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
                    || self.cpu_cycle == 0)
                    && self.irq_flag_setting_in_progress))
            && !self.frame_counter.is_interrupt_inhibit_flag_set()
        {
            if self.cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES - 1 {
                self.irq_flag_setting_in_progress = true;
            }
            if self.cpu_cycle == 0 {
                self.irq_flag_setting_in_progress = false;
            }
            self.frame_interrupt = true;
        }
        self.cpu_cycle = self.shifted_cpu_cycle(1);

        if self.cpu_cycle == 0 {
            if self.frame == std::u128::MAX {
                self.frame = 0;
            } else {
                self.frame += 1;
            }
        }

        let sample = Self::get_mixer_output(
            self.pulse_1.get_sample(),
            self.pulse_2.get_sample(),
            self.triangle.get_sample(),
            self.noise.get_sample(),
            self.dmc.get_sample(),
        ) - 0.5;

        self.audio_access.borrow_mut().add_sample(sample);
    }

    fn get_mixer_output(
        pulse_1: u8,
        pulse_2: u8,
        triangle: u8,
        noise: u8,
        dmc: u8,
    ) -> SampleFormat {
        let mut n = pulse_1 + pulse_2;
        let puls_out = if n != 0 {
            95.52 / ((8128.0 / (n as f32)) + 100.0)
        } else {
            0.0
        };
        n = 3 * triangle + 2 * noise + dmc;
        let tnd_out = if n != 0 {
            163.67 / ((24329.0 / (n as f32)) + 100.0)
        } else {
            0.0
        };
        (puls_out + tnd_out) as SampleFormat
    }

    fn is_half_frame_reached(&self) -> bool {
        let next_cpu_cycle = self.cpu_cycle + 1;
        if next_cpu_cycle == FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES {
            return true;
        } else if self.frame_counter.get_sequencer_mode() == 0
            && next_cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
        {
            return true;
        } else if self.frame_counter.get_sequencer_mode() == 1
            && next_cpu_cycle == FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES
        {
            return true;
        }
        false
    }

    fn is_quarter_frame_reached(&self) -> bool {
        let next_cpu_cycle = self.cpu_cycle + 1;
        if self.is_half_frame_reached() {
            return true;
        } else {
            if next_cpu_cycle == FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES {
                return true;
            }
            if next_cpu_cycle == FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES {
                return true;
            }
        }
        false
    }
}

impl WriteAcessRegisters for APU {
    fn write(&mut self, register: WriteAccessRegister, value: u8) -> () {
        match register {
            WriteAccessRegister::Pulse1_0 => self.pulse_1.data[0] = value,
            WriteAccessRegister::Pulse1_1 => {
                self.pulse_1.data[1] = value;
                self.pulse_1.sweep_unit.reload_flag = true;
            }
            WriteAccessRegister::Pulse1_2 => self.pulse_1.data[2] = value,
            WriteAccessRegister::Pulse1_3 => {
                self.pulse_1.data[3] = value;
                self.pulse_1.reset();
                self.reload_length_counter_if_enabled(StatusRegisterFlag::Pulse1Enabled);
            }

            WriteAccessRegister::Pulse2_0 => self.pulse_2.data[0] = value,
            WriteAccessRegister::Pulse2_1 => {
                self.pulse_2.data[1] = value;
                self.pulse_2.sweep_unit.reload_flag = true;
            }
            WriteAccessRegister::Pulse2_2 => self.pulse_2.data[2] = value,
            WriteAccessRegister::Pulse2_3 => {
                self.pulse_2.data[3] = value;
                self.pulse_2.reset();
                self.reload_length_counter_if_enabled(StatusRegisterFlag::Pulse2Enabled);
            }

            WriteAccessRegister::Triangle0 => self.triangle.data[0] = value,
            WriteAccessRegister::Triangle1 => self.triangle.data[1] = value,
            WriteAccessRegister::Triangle2 => self.triangle.data[2] = value,
            WriteAccessRegister::Triangle3 => {
                self.triangle.data[3] = value;
                self.triangle.linear_counter_reload_flag = true;
                self.reload_length_counter_if_enabled(StatusRegisterFlag::TriangleEnabled);
            }
            WriteAccessRegister::Noise0 => self.noise.data[0] = value,
            WriteAccessRegister::Noise1 => self.noise.data[1] = value,
            WriteAccessRegister::Noise2 => self.noise.data[2] = value,
            WriteAccessRegister::Noise3 => {
                self.noise.data[3] = value;
                self.noise.reset();
                self.reload_length_counter_if_enabled(StatusRegisterFlag::NoiseEnabled);
            }
            WriteAccessRegister::DMC0 => {
                self.dmc.data[0] = value;
                if !self.dmc.is_irq_enabled() {
                    self.dmc.interrupt = false;
                }
            }
            WriteAccessRegister::DMC1 => {
                self.dmc.data[1] = value;
                self.dmc.output_value = self.dmc.get_direct_load();
            }
            WriteAccessRegister::DMC2 => self.dmc.data[2] = value,
            WriteAccessRegister::DMC3 => self.dmc.data[3] = value,

            WriteAccessRegister::Status => {
                self.status.data = value;
                if !self.status.is_flag_enabled(StatusRegisterFlag::DMCEnabled) {
                    self.dmc.bytes_remaining = 0;
                } else if self.dmc.bytes_remaining == 0 {
                    self.dmc.next_bytes_remaining = self.dmc.get_sample_length();
                    self.dmc.start_pending = true;
                }
                self.reset_length_counter_if_disabled(StatusRegisterFlag::Pulse1Enabled);
                self.reset_length_counter_if_disabled(StatusRegisterFlag::Pulse2Enabled);
                self.reset_length_counter_if_disabled(StatusRegisterFlag::TriangleEnabled);
                self.reset_length_counter_if_disabled(StatusRegisterFlag::NoiseEnabled);

                self.dmc.interrupt = false;
            }
            WriteAccessRegister::FrameCounter => {
                let shift = if self.cpu_cycle % 2 == 0 { 3 } else { 2 };
                self.frame_counter.data = value;
                if self.frame_counter.is_interrupt_inhibit_flag_set() {
                    self.frame_interrupt = false;
                }
                self.pending_reset_cycle = Some(self.shifted_cpu_cycle(shift));
            }
        }
    }
}

impl ReadAccessRegisters for APU {
    fn read(&mut self, register: ReadAccessRegister) -> u8 {
        match register {
            ReadAccessRegister::Status => {
                let mut out = StatusRegister { data: 0 };
                out.set_flag_status(StatusRegisterFlag::FrameInterrupt, self.frame_interrupt);
                out.set_flag_status(StatusRegisterFlag::DMCInterrupt, self.dmc.interrupt);
                out.set_flag_status(
                    StatusRegisterFlag::DMCEnabled,
                    self.dmc.bytes_remaining > 0 || self.dmc.start_pending,
                );

                let mut set_status = |flag| {
                    let channel = self.get_length_counter_channel(flag);
                    out.set_flag_status(flag, channel.get_length_counter() > 0);
                };

                set_status(StatusRegisterFlag::NoiseEnabled);
                set_status(StatusRegisterFlag::TriangleEnabled);
                set_status(StatusRegisterFlag::Pulse1Enabled);
                set_status(StatusRegisterFlag::Pulse2Enabled);

                let interrupt_set_cycles = vec![FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES, 0];
                if !interrupt_set_cycles.contains(&self.cpu_cycle) {
                    self.frame_interrupt = false;
                }
                out.data
            }
        }
    }
}

impl ApuRegisterAccess for APU {}
