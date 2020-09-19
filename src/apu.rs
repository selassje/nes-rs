use self::StatusRegisterFlag::*;
use crate::ram_apu::*;
use std::default::Default;
use std::time::{Duration, Instant};

use crate::io_sdl::{SampleFormat, BUFFER_SIZE, SAMPLE_BUFFER, SAMPLE_RATE};

const CPU_CYCLES_PER_SECOND: u128 = Duration::from_secs(1).as_nanos() / 559;
const CPU_CYCLES_PER_SAMPLE: u128 = CPU_CYCLES_PER_SECOND / SAMPLE_RATE as u128;

const LENGTH_COUNTER_LOOKUP_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 2, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_CYCLE_SEQUENCES: [[SampleFormat; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

trait LengthCounterSupport {
    fn get_length_counter_load(&self) -> u8;
    fn set_length_counter(&mut self, count: u8);
}

fn reload_length_counter<T: LengthCounterSupport>(channel: &mut T) {
    channel.set_length_counter(
        LENGTH_COUNTER_LOOKUP_TABLE[channel.get_length_counter_load() as usize],
    );
}

fn reset_length_counter<T: LengthCounterSupport>(channel: &mut T) {
    channel.set_length_counter(0);
}

const FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES: u16 = 7457;
const FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES: u16 = 14913;
const FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES: u16 = 22371;
const FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES: u16 = 29829;
const FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES: u16 = 37282;

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
        target_period: u16,
        shift: u8,
        negate: bool,
    ) -> u16 {
        let change_amount = raw_period >> shift;
        if negate {
            if self.use_ones_complement {
                target_period - change_amount - 1
            } else {
                target_period - change_amount
            }
        } else {
            target_period + change_amount
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
    left_over_cpu_cycles: u8,
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
            left_over_cpu_cycles: 0,
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
        self.timer_tick = self.get_raw_timer_period();
        self.envelope.start_flag = true;
        self.sweep_unit.reload_flag = true;
    }

    fn process_elapsed_cpu_cycles(&mut self, elapsed_cpu_cycles: u16) {
        let total_elapsed_cycles = elapsed_cpu_cycles + self.left_over_cpu_cycles as u16;
        self.left_over_cpu_cycles = (total_elapsed_cycles % 2) as u8;
        let number_of_elapsed_ticks = (total_elapsed_cycles / 2) as u16;
        if number_of_elapsed_ticks as u16 > self.timer_tick && self.current_period != 0 {
            // println!("current_period {} number_of_elapsed_ticks {}",self.current_period, number_of_elapsed_ticks);
            self.timer_tick = self.current_period + 1 - number_of_elapsed_ticks;

            if self.sequencer_position > 0 {
                self.sequencer_position -= 1;
            } else {
                self.sequencer_position = 7;
            }
        } else if self.timer_tick >= number_of_elapsed_ticks {
            self.timer_tick -= number_of_elapsed_ticks;
        }
    }

    fn get_volume(&self) -> u8 {
        //return 15;
        if self.length_counter == 0 {
            if self.sweep_unit.is_muting {
                // println!("muted by the sweep");
            }
            0
        } else if self.is_constant_volume_set() {
            self.get_constant_volume_or_envelope_divider_reload_value()
        } else {
            self.envelope.decay_level_counter
        }
    }

    fn get_sample_value(&self) -> SampleFormat {
        DUTY_CYCLE_SEQUENCES[self.get_duty_cycle() as usize][self.sequencer_position as usize]
            * self.get_volume() as SampleFormat
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

impl LengthCounterSupport for PulseWave {
    fn get_length_counter_load(&self) -> u8 {
        (self.data[3] & 0b11111000) >> 3
    }

    fn set_length_counter(&mut self, counter: u8) {
        self.length_counter = counter
    }
}

#[derive(Default)]
struct TriangleWave {
    data: [u8; 4],
    length_counter: u8,
}

impl TriangleWave {
    fn is_length_counter_halt_set(&self) -> bool {
        (self.data[0] & 0b00100000) != 0
    }

    fn get_linear_counter_load(&self) -> u8 {
        self.data[0] & 0b01111111
    }

    fn get_timer(&self) -> u16 {
        let timer_hi = ((self.data[3] & 0x7) as u16) << 8;
        self.data[2] as u16 + timer_hi
    }

    fn clock_linear_counter(&mut self) {}

    fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.is_length_counter_halt_set() {
            self.length_counter -= 1;
        }
    }
}

impl LengthCounterSupport for TriangleWave {
    fn get_length_counter_load(&self) -> u8 {
        (self.data[3] & 0b11111000) >> 3
    }

    fn set_length_counter(&mut self, counter: u8) {
        self.length_counter = counter
    }
}

#[derive(Default)]
struct Noise {
    data: [u8; 4],
    length_counter: u8,
}

impl Noise {
    fn is_length_counter_halt_set(&self) -> bool {
        (self.data[0] & 0b00100000) != 0
    }

    fn is_constant_volume_set(&self) -> bool {
        (self.data[0] & 0b00010000) != 0
    }

    fn get_volume_or_envelope(&self) -> u8 {
        self.data[0] & 0x0F
    }

    fn get_linear_counter_load(&self) -> u8 {
        self.data[0] & 0b01111111
    }

    fn is_noise_loop_set(&self) -> bool {
        (self.data[2] & 0b10000000) != 0
    }

    fn get_noise_period(&self) -> u8 {
        self.data[2] & 0x0F
    }

    fn clock_envelope(&self) {}

    fn clock_length_counter_and_sweep_unit(&mut self) {
        if self.length_counter > 0 && !self.is_length_counter_halt_set() {
            self.length_counter -= 1;
        }
    }
}

impl LengthCounterSupport for Noise {
    fn get_length_counter_load(&self) -> u8 {
        (self.data[3] & 0b11111000) >> 3
    }

    fn set_length_counter(&mut self, counter: u8) {
        self.length_counter = counter
    }
}

#[derive(Default)]
struct DMC {
    data: [u8; 4],
}

impl DMC {
    fn is_irq_enabled(&self) -> bool {
        (self.data[0] & 0b10000000) != 0
    }

    fn is_loop_enabled(&self) -> bool {
        (self.data[0] & 0b01000000) != 0
    }

    fn get_frequency(&self) -> u8 {
        self.data[0] & 0x0F
    }

    fn get_load_counter(&self) -> u8 {
        self.data[1] & 0b01111111
    }

    fn get_sample_address(&self) -> u8 {
        self.data[2]
    }

    fn get_sample_length(&self) -> u8 {
        self.data[3]
    }
}

pub struct APU {
    frame_counter: FrameCounter,
    status: StatusRegister,
    pulse_1: PulseWave,
    pulse_2: PulseWave,
    triangle: TriangleWave,
    noise: Noise,
    dmc: DMC,
    cpu_cycles: u16,
    cpu_cycles_sample: u16,
    frame_interrupt: bool,
    dmc_interrupt: bool,
    sample_timer: Instant,
    t_inc: f32,
    t_phase: f32,
}

impl APU {
    pub fn new() -> Self {
        APU {
            frame_counter: FrameCounter { data: 0 },
            status: StatusRegister { data: 0 },
            pulse_1: PulseWave::new(false),
            pulse_2: PulseWave::new(true),
            triangle: TriangleWave::default(),
            noise: Noise::default(),
            dmc: DMC::default(),
            cpu_cycles: 0,
            cpu_cycles_sample: 0,
            frame_interrupt: false,
            dmc_interrupt: false,
            sample_timer: Instant::now(),
            t_inc: 440.0 / SAMPLE_RATE as f32,
            t_phase: 0.0,
        }
    }

    fn perform_half_frame_update(&mut self) {
        self.pulse_1.clock_length_counter_and_sweep_unit();
        self.pulse_2.clock_length_counter_and_sweep_unit();
        self.triangle.clock_length_counter();
        self.noise.clock_length_counter_and_sweep_unit();
    }

    fn perform_quarter_frame_update(&mut self) {
        self.pulse_1.clock_envelope();
        self.pulse_2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    pub fn process_cpu_cycles(&mut self, cpu_cycles: u8, enable_sound: bool) {
        static mut last_p1_sample: SampleFormat = 0;

        let elapsed_cpu_cycles = cpu_cycles as u16;

        if self.is_quarter_frame_reached(elapsed_cpu_cycles) {
            self.perform_quarter_frame_update();
        }

        if self.is_half_frame_reached(elapsed_cpu_cycles) {
            self.perform_half_frame_update();
        }

        self.pulse_1.process_elapsed_cpu_cycles(elapsed_cpu_cycles);
        self.pulse_2.process_elapsed_cpu_cycles(elapsed_cpu_cycles);

        if self.frame_counter.get_sequencer_mode() == 0
            && (self.cpu_cycles + elapsed_cpu_cycles) >= FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
            && !self.frame_counter.is_interrupt_inhibit_flag_set()
        {
            self.frame_interrupt = true;
        }

        if self.frame_counter.get_sequencer_mode() == 0 {
            self.cpu_cycles = (self.cpu_cycles + elapsed_cpu_cycles)
                % FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES;
        } else {
            self.cpu_cycles = (self.cpu_cycles + elapsed_cpu_cycles)
                % FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES;
        }
        let mut sample = Self::get_mixer_output(
            self.pulse_1.get_sample_value(),
            self.pulse_2.get_sample_value(),
            0,
            0,
            0,
        );
        unsafe {
            if sample != last_p1_sample {
                /*
                                println!(
                                    "PULSE 1 Sample {} duty {} duty_pos {} period {} length_counter {} halt_counter {} sweep_enabled {} sweep_shift {}, sweep_negate {} sweep_period {}",
                                    sample,
                                    self.pulse_1.get_duty_cycle(),
                                    self.pulse_1.sequencer_position,
                                    self.pulse_1.current_period,
                                    self.pulse_1.length_counter,
                                    self.pulse_1.is_length_counter_halt_envelope_loop_flag_set(),
                                    self.pulse_1.is_sweep_unit_enabled(),
                                    self.pulse_1.get_sweep_shift(),
                                    self.pulse_1.is_sweep_negate_enabled(),
                                    self.pulse_1.get_sweep_period(),
                                );
                */
                last_p1_sample = sample;
            }
        }

        if self.cpu_cycles_sample + elapsed_cpu_cycles >= CPU_CYCLES_PER_SAMPLE as u16
            && enable_sound
        {
            loop {
                if SAMPLE_BUFFER.lock().unwrap().index != BUFFER_SIZE {
                    break;
                }
            }
            if false {
                if self.t_phase >= 0.0 && self.t_phase < 0.5 {
                    sample = 15;
                } else {
                    sample = 0;
                }
            }
            self.t_phase = (self.t_phase + self.t_inc) % 1.0;

            let mut sample_buffer = SAMPLE_BUFFER.lock().unwrap();
            let index = sample_buffer.index;
            sample_buffer.buffer[index] = sample;
            sample_buffer.index += 1;
            self.sample_timer = Instant::now();
        }
        self.cpu_cycles_sample =
            (self.cpu_cycles_sample + elapsed_cpu_cycles) % CPU_CYCLES_PER_SAMPLE as u16;
    }

    fn get_mixer_output(
        pulse_1: u8,
        pulse_2: u8,
        triangle: u8,
        noise: u8,
        dmc: u8,
    ) -> SampleFormat {
        let mut n = (pulse_1 + pulse_2) as f32;
        let puls_out = 95.52 / (8128.0 / n + 100.0);
        n = (3 * triangle + 2 * noise + dmc) as f32;
        let tnd_out = 163.67 / (24329.0 / n + 100.0);
        ((puls_out + tnd_out) * 100.0) as SampleFormat
    }

    fn is_half_frame_reached(&self, elapsed_cpu_cycles: u16) -> bool {
        if self.cpu_cycles < FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES
            && self.cpu_cycles + elapsed_cpu_cycles >= FRAME_COUNTER_HALF_FRAME_1_CPU_CYCLES
        {
            return true;
        } else if self.frame_counter.get_sequencer_mode() == 0
            && self.cpu_cycles < FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
            && self.cpu_cycles + elapsed_cpu_cycles >= FRAME_COUNTER_HALF_FRAME_0_MOD_0_CPU_CYCLES
        {
            return true;
        } else if self.frame_counter.get_sequencer_mode() == 1
            && self.cpu_cycles < FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES
            && self.cpu_cycles + elapsed_cpu_cycles >= FRAME_COUNTER_HALF_FRAME_0_MOD_1_CPU_CYCLES
        {
            return true;
        }
        false
    }

    fn is_quarter_frame_reached(&self, elapsed_cpu_cycles: u16) -> bool {
        if self.is_half_frame_reached(elapsed_cpu_cycles) {
            return true;
        } else {
            if self.cpu_cycles < FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES
                && self.cpu_cycles + elapsed_cpu_cycles >= FRAME_COUNTER_QUARTER_FRAME_1_CPU_CYCLES
            {
                return true;
            }
            if self.cpu_cycles < FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES
                && self.cpu_cycles + elapsed_cpu_cycles >= FRAME_COUNTER_QUARTER_FRAME_3_CPU_CYCLES
            {
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
                if self
                    .status
                    .is_flag_enabled(StatusRegisterFlag::Pulse1Enabled)
                {
                    reload_length_counter(&mut self.pulse_1);
                }
                self.pulse_1.reset();
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
            }

            WriteAccessRegister::Triangle0 => self.triangle.data[0] = value,
            WriteAccessRegister::Triangle1 => self.triangle.data[1] = value,
            WriteAccessRegister::Triangle2 => self.triangle.data[2] = value,
            WriteAccessRegister::Triangle3 => self.triangle.data[3] = value,

            WriteAccessRegister::Noise0 => self.noise.data[0] = value,
            WriteAccessRegister::Noise1 => self.noise.data[1] = value,
            WriteAccessRegister::Noise2 => self.noise.data[2] = value,
            WriteAccessRegister::Noise3 => self.noise.data[3] = value,

            WriteAccessRegister::DMC0 => self.dmc.data[0] = value,
            WriteAccessRegister::DMC1 => self.dmc.data[1] = value,
            WriteAccessRegister::DMC2 => self.dmc.data[2] = value,
            WriteAccessRegister::DMC3 => self.dmc.data[3] = value,

            WriteAccessRegister::Status => {
                self.status.data = value;
                if !self
                    .status
                    .is_flag_enabled(StatusRegisterFlag::Pulse1Enabled)
                {
                    reset_length_counter(&mut self.pulse_1);
                }
                if !self
                    .status
                    .is_flag_enabled(StatusRegisterFlag::Pulse2Enabled)
                {
                    reset_length_counter(&mut self.pulse_2);
                }

                if !self
                    .status
                    .is_flag_enabled(StatusRegisterFlag::TriangleEnabled)
                {
                    reset_length_counter(&mut self.triangle);
                }

                if !self
                    .status
                    .is_flag_enabled(StatusRegisterFlag::NoiseEnabled)
                {
                    reset_length_counter(&mut self.noise);
                }
                self.frame_interrupt = false;
            }
            WriteAccessRegister::FrameCounter => {
                self.frame_counter.data = value;
                self.cpu_cycles = 0;
                self.perform_half_frame_update();
                self.perform_quarter_frame_update();
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
                out.set_flag_status(StatusRegisterFlag::DMCInterrupt, self.dmc_interrupt);
                out.set_flag_status(
                    StatusRegisterFlag::NoiseEnabled,
                    self.noise.length_counter > 0,
                );
                out.set_flag_status(
                    StatusRegisterFlag::TriangleEnabled,
                    self.triangle.length_counter > 0,
                );
                out.set_flag_status(
                    StatusRegisterFlag::Pulse2Enabled,
                    self.pulse_2.length_counter > 0,
                );
                out.set_flag_status(
                    StatusRegisterFlag::Pulse1Enabled,
                    self.pulse_1.length_counter > 0,
                );
                self.frame_interrupt = false;
                out.data
            }
        }
    }
}

impl ApuRegisterAccess for APU {}
