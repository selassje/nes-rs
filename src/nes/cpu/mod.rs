mod opcodes;

use self::{opcodes::IRQ_OPCODE, AddressingMode::*};
use super::apu::ApuState;
use super::ppu::PpuState;
use super::CpuBus;
use super::RamBus;
use super::{common::*, memory::Memory};
use super::{mappers::Mapper, ram_ppu::DmaWriteAccessRegister::OamDma};
use opcodes::{get_opcodes, OpCodes, NMI_OPCODE};
use serde::{Deserialize, Serialize};

use std::fmt::{Display, Formatter, Result};

const STACK_PAGE: u16 = 0x0100;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirectX,
    IndirectIndexedY,
}

impl AddressingMode {
    fn get_bytes(&self) -> u8 {
        match self {
            Implicit => 1,
            Accumulator => 1,
            Immediate => 2,
            ZeroPage => 2,
            ZeroPageX => 2,
            ZeroPageY => 2,
            Relative => 2,
            Absolute => 3,
            AbsoluteX => 3,
            AbsoluteY => 3,
            Indirect => 3,
            IndexedIndirectX => 2,
            IndirectIndexedY => 2,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Address {
    Implicit,
    Accumulator,
    Immediate(u8),
    Ram(u16),
    Relative(u16),
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Address::Implicit => write!(f, ""),
            Address::Accumulator => write!(f, "A"),
            Address::Immediate(i) => write!(f, "{:#X}", i),
            Address::Ram(address) => write!(f, "{:#X}", address),
            Address::Relative(pc) => write!(f, "{:#X}", pc),
        }
    }
}

enum ProcessorFlag {
    CarryFlag = 0b00000001,
    ZeroFlag = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode = 0b00001000,
    BFlagBit4 = 0b00010000,
    BFlagBit5 = 0b00100000,
    OverflowFlag = 0b01000000,
    NegativeFlag = 0b10000000,
}

pub type InstructionFun = fn(&mut Cpu, &mut CpuBus);

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Instruction {
    opcode: u8,
    total_cycles: u16,
    cycle: u16,
    bytes: u8,
}
macro_rules! ram_bus {
    ($cpu_bus:expr) => {{
        RamBus {
            ppu: $cpu_bus.ppu,
            apu: $cpu_bus.apu,
            mapper: $cpu_bus.mapper,
            controllers: $cpu_bus.controllers,
            callback: $cpu_bus.callback,
        }
    }};
}
impl CpuBus<'_> {
    fn get_byte(&mut self, address: u16) -> u8 {
        let mut ram_bus = ram_bus!(self);
        self.ram.get_byte(address, &mut ram_bus)
    }

    fn get_word(&mut self, address: u16) -> u16 {
        let mut ram_bus = ram_bus!(self);
        super::common::convert_2u8_to_u16(
            self.ram.get_byte(address, &mut ram_bus),
            self.ram.get_byte(address + 1, &mut ram_bus),
        )
    }

    fn store_byte(&mut self, address: u16, byte: u8) {
        let mut ram_bus = ram_bus!(self);
        self.ram.store_byte(address, byte, &mut ram_bus);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cpu {
    pc: u16,
    sp: u8,
    ps: u8,
    a: u8,
    x: u8,
    y: u8,
    cycle: u128,
    instruction: Option<Instruction>,
    address: Address,
    code_segment: (u16, u16),
    #[serde(skip, default = "get_opcodes")]
    opcodes: OpCodes,
    interrupt: Option<u8>,
    is_brk_or_irq_hijacked_by_nmi: bool,
    oam_dma_in_progress: Option<u16>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0xFD,
            ps: 0x24,
            a: 0,
            x: 0,
            y: 0,
            cycle: 0,
            instruction: None,
            code_segment: (0, 0),
            interrupt: None,
            address: Address::Implicit,
            opcodes: get_opcodes(),
            is_brk_or_irq_hijacked_by_nmi: false,
            oam_dma_in_progress: None,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0xFD,
            ps: 0x24,
            a: 0,
            x: 0,
            y: 0,
            cycle: 0,
            instruction: None,
            code_segment: (0, 0),
            interrupt: None,
            address: Address::Implicit,
            opcodes: get_opcodes(),
            is_brk_or_irq_hijacked_by_nmi: false,
            oam_dma_in_progress: None,
        }
    }

    pub fn power_cycle(&mut self, bus: &mut CpuBus) {
        self.pc = bus.get_word(0xFFFC);
        self.sp = 0xFD;
        self.ps = 0x04;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.cycle = 8;
        self.instruction = None;
        self.code_segment = (0, 0xFFFF);
        self.address = Address::Implicit;
        self.is_brk_or_irq_hijacked_by_nmi = false;
        self.oam_dma_in_progress = None;
        self.opcodes = get_opcodes();
    }

    fn set_flag(&mut self, flag: ProcessorFlag) {
        self.ps |= flag as u8;
    }

    fn clear_flag(&mut self, flag: ProcessorFlag) {
        self.ps &= !(flag as u8);
    }

    fn get_flag(&self, flag: ProcessorFlag) -> bool {
        (self.ps & (flag as u8)) != 0
    }

    fn set_or_clear_flag(&mut self, flag: ProcessorFlag, cond: bool) {
        if cond {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn carry(&mut self) -> u8 {
        if self.get_flag(ProcessorFlag::CarryFlag) {
            1
        } else {
            0
        }
    }

    fn pop_byte(&mut self, bus: &mut CpuBus) -> u8 {
        self.sp = ((self.sp as u16 + 1) & 0xFF) as u8;
        bus.get_byte(self.sp as u16 + STACK_PAGE)
    }

    fn push_byte(&mut self, val: u8, bus: &mut CpuBus) {
        bus.store_byte(self.sp as u16 + STACK_PAGE, val);
        self.sp = ((self.sp as i16 - 1) & 0xFF) as u8;
    }

    fn push_word(&mut self, val: u16, bus: &mut CpuBus) {
        self.push_byte(((val & 0xFF00) >> 8) as u8, bus);
        self.push_byte((val & 0x00FF) as u8, bus);
    }

    fn pop_word(&mut self, bus: &mut CpuBus) -> u16 {
        let low_byte = self.pop_byte(bus);
        let high_byte = self.pop_byte(bus);
        convert_2u8_to_u16(low_byte, high_byte)
    }
    pub fn maybe_fetch_next_instruction(&mut self, bus: &mut CpuBus) {
        if self.instruction.is_none() {
            self.fetch_next_instruction(bus);
        }
    }

    fn check_for_interrupts(&mut self, bus: &mut CpuBus) {
        if bus.ppu.check_for_nmi_pending() {
            bus.ppu.clear_nmi_pending();
            self.interrupt = Some(NMI_OPCODE as u8);
        } else if !self.get_flag(ProcessorFlag::InterruptDisable)
            && (bus.mapper.is_irq_pending() || bus.apu.is_irq_pending())
        {
            self.interrupt = Some(IRQ_OPCODE as u8)
        }
    }

    fn fetch_next_instruction(&mut self, bus: &mut CpuBus) {
        let ppu_time = bus.ppu.get_time();
        let op = if let Some(op) = self.interrupt.take() {
            op
        } else {
            bus.get_byte(self.pc)
        };

        let (_, code_segment_end) = self.code_segment;
        if let Some(opcode) = self.opcodes[op as usize] {
            let mut operand_1 = 0;
            let mut operand_2 = 0;

            if self.pc < code_segment_end {
                operand_1 = bus.get_byte(self.pc + 1);
            }
            if self.pc + 1 < code_segment_end {
                operand_2 = bus.get_byte(self.pc + 2);
            }

            let (address, extra_cycles_from_page_crossing) = self
                .get_address_and_extra_cycle_from_page_crossing(
                    operand_1,
                    operand_2,
                    opcode.mode,
                    opcode.extra_cycle_on_page_crossing,
                    bus,
                );
            self.address = address;

            let extra_cycles_from_branching =
                self.get_extra_cycles_from_branching(opcode.instruction as usize);

            let extra_cycles_from_oam_dma = self.get_extra_cycles_from_oam_dma();
            if extra_cycles_from_oam_dma != 0 {
                self.oam_dma_in_progress = Some(
                    opcode.base_cycles as u16
                        + extra_cycles_from_page_crossing
                        + extra_cycles_from_branching,
                )
            } else {
                self.oam_dma_in_progress = None;
            }

            let total_cycles = opcode.base_cycles as u16
                + extra_cycles_from_page_crossing
                + extra_cycles_from_branching
                + extra_cycles_from_oam_dma;

            self.instruction = Some(Instruction {
                opcode: op,
                total_cycles,
                cycle: 0,
                bytes: opcode.mode.get_bytes(),
            });

            if false {
                println!(
                  "{:04X} {:02X} {:02X} {:02X} \t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:<3} SL:{:<3} FC:{} CPU Cycle:{}",
                  self.pc,
                  op,
                  operand_1,
                  operand_2,
                  self.a,
                  self.x,
                  self.y,
                  self.ps,
                  self.sp,
                  ppu_time.cycle,
                  ppu_time.scanline,
                  ppu_time.frame,
                  self.cycle,
              );
            }
            if opcode.instruction as usize == Self::rti as usize {
                self.rti_restore_ps(bus);
            }
        } else {
            panic!("Unknown instruction {:#04x} at {:#X} ", op, self.pc);
        }
    }

    pub fn run_single_cycle(&mut self, bus: &mut CpuBus) {
        self.instruction.as_mut().unwrap().cycle += 1;
        let instruction = self.instruction.unwrap();
        let ins_fun = self.opcodes[instruction.opcode as usize]
            .unwrap()
            .instruction;
        let is_brk_or_irq_executing =
            ins_fun as usize == Self::brk as usize || ins_fun as usize == Self::irq as usize;
        let is_nmi_executing = ins_fun as usize == Self::nmi as usize;
        let is_branching_executing = self.is_current_instruction_branching();
        if instruction.cycle == instruction.total_cycles {
            (ins_fun)(self, bus);
            self.pc = ((self.pc as u32 + instruction.bytes as u32) % u16::MAX as u32) as u16;
            let cycles_left = u128::MAX - self.cycle;
            if cycles_left < instruction.total_cycles as u128 {
                self.cycle = instruction.total_cycles as u128 - cycles_left;
            } else {
                self.cycle += instruction.total_cycles as u128;
            }
            self.instruction = None;
        } else if is_brk_or_irq_executing {
            if bus.ppu.check_for_nmi_pending() && instruction.cycle <= 4 {
                self.is_brk_or_irq_hijacked_by_nmi = true;
                bus.ppu.clear_nmi_pending()
            }
        } else if is_branching_executing {
            if instruction.cycle == 1 || (instruction.cycle == 3 && instruction.total_cycles == 4) {
                self.check_for_interrupts(bus);
            }
        } else if !is_nmi_executing
            && ((self.oam_dma_in_progress.is_some()
                && instruction.cycle == self.oam_dma_in_progress.unwrap() - 1)
                || (self.oam_dma_in_progress.is_none()
                    && instruction.cycle == instruction.total_cycles - 1))
        {
            self.check_for_interrupts(bus)
        }
    }

    fn get_extra_cycles_from_oam_dma(&mut self) -> u16 {
        let mut extra_cycles = 0;
        if self.address == Address::Ram(OamDma as u16) {
            extra_cycles = 513;
            if self.cycle % 2 == 1 {
                extra_cycles += 1;
            }
        }
        extra_cycles
    }

    fn is_current_instruction_branching(&self) -> bool {
        let ins = self.opcodes[self.instruction.unwrap().opcode as usize]
            .unwrap()
            .instruction as usize;
        let bcc_fn: usize = Cpu::bcc as usize;
        let bcs_fn: usize = Cpu::bcs as usize;
        let bpl_fn: usize = Cpu::bpl as usize;
        let bmi_fn: usize = Cpu::bmi as usize;
        let bne_fn: usize = Cpu::bne as usize;
        let beq_fn: usize = Cpu::beq as usize;
        let bvc_fn: usize = Cpu::bvc as usize;
        let bvs_fn: usize = Cpu::bvs as usize;

        ins == bcc_fn
            || ins == bcs_fn
            || ins == bpl_fn
            || ins == bmi_fn
            || ins == bne_fn
            || ins == beq_fn
            || ins == bvc_fn
            || ins == bvs_fn
    }

    fn is_page_crossed(addr_1: u16, addr_2: u16) -> bool {
        addr_1 & 0xFF00 != addr_2 & 0xFF00
    }

    fn get_extra_cycles_from_branching(&self, ins: usize) -> u16 {
        let bcc_fn = Self::bcc as usize;
        let bcs_fn = Self::bcs as usize;
        let bpl_fn = Self::bpl as usize;
        let bmi_fn = Self::bmi as usize;
        let bne_fn = Self::bne as usize;
        let beq_fn = Self::beq as usize;
        let bvc_fn = Self::bvc as usize;
        let bvs_fn = Self::bvs as usize;

        if (ins == bcc_fn && self.check_condition_for_bcc())
            || (ins == bcs_fn && self.check_condition_for_bcs())
            || (ins == bpl_fn && self.check_condition_for_bpl())
            || (ins == bmi_fn && self.check_condition_for_bmi())
            || (ins == bne_fn && self.check_condition_for_bne())
            || (ins == beq_fn && self.check_condition_for_beq())
            || (ins == bvc_fn && self.check_condition_for_bvc())
            || (ins == bvs_fn && self.check_condition_for_bvs())
        {
            let mut extra_cycles = 1;
            if let Address::Relative(new_pc) = self.address {
                if Self::is_page_crossed(new_pc + 2, self.pc + 2) {
                    extra_cycles += 1;
                }
            } else {
                panic!("");
            }
            extra_cycles
        } else {
            0
        }
    }

    fn get_address_and_extra_cycle_from_page_crossing(
        &self,
        operand_1: u8,
        operand_2: u8,
        mode: AddressingMode,
        extra_cycle_on_page_crossing: bool,
        bus: &mut CpuBus,
    ) -> (Address, u16) {
        let b0_u16 = operand_1 as u16;
        let b1_u16 = operand_2 as u16;
        let x_u16 = self.x as u16;
        let y_u16 = self.y as u16;
        let zero_page_x = (b0_u16 + x_u16) & 0xFF;
        let zero_page_x_hi = (b0_u16 + x_u16 + 1) & 0xFF;
        let zero_page_y = (b0_u16 + y_u16) & 0xFF;
        let mut extra_cycle = 0;

        let address = match mode {
            AddressingMode::Implicit => Address::Implicit,
            AddressingMode::Accumulator => Address::Accumulator,
            AddressingMode::Immediate => Address::Immediate(operand_1),
            AddressingMode::ZeroPage => Address::Ram(b0_u16),
            AddressingMode::ZeroPageX => Address::Ram(zero_page_x),
            AddressingMode::ZeroPageY => Address::Ram(zero_page_y),
            AddressingMode::Absolute => Address::Ram(convert_2u8_to_u16(operand_1, operand_2)),

            AddressingMode::AbsoluteX => {
                if extra_cycle_on_page_crossing && b0_u16 + x_u16 > 0xFF {
                    extra_cycle = 1
                }
                let address = b0_u16 as u32 + x_u16 as u32 + (b1_u16 << 8) as u32;
                Address::Ram(address as u16)
            }
            AddressingMode::AbsoluteY => {
                if extra_cycle_on_page_crossing && b0_u16 + y_u16 > 0xFF {
                    extra_cycle = 1
                }
                let address = b0_u16 as u32 + y_u16 as u32 + (b1_u16 << 8) as u32;
                Address::Ram(address as u16)
            }
            AddressingMode::IndexedIndirectX => {
                let indexed_indirect =
                    convert_2u8_to_u16(bus.get_byte(zero_page_x), bus.get_byte(zero_page_x_hi));
                Address::Ram(indexed_indirect)
            }
            AddressingMode::IndirectIndexedY => {
                let indirect =
                    convert_2u8_to_u16(bus.get_byte(b0_u16), bus.get_byte((b0_u16 + 1) & 0xFF));
                let indirect_indexed = (indirect as u32 + y_u16 as u32) as u16;
                if extra_cycle_on_page_crossing && Self::is_page_crossed(indirect_indexed, indirect)
                {
                    extra_cycle = 1;
                }
                Address::Ram(indirect_indexed)
            }
            AddressingMode::Relative => {
                let new_pc = (self.pc as i16 + (operand_1 as i8 as i16)) as u16;
                Address::Relative(new_pc)
            }
            AddressingMode::Indirect => {
                let indirect = if b0_u16 == 0xFF {
                    convert_2u8_to_u16(
                        bus.get_byte(b0_u16 + (b1_u16 << 8)),
                        bus.get_byte(b1_u16 << 8),
                    )
                } else {
                    convert_2u8_to_u16(
                        bus.get_byte(b0_u16 + (b1_u16 << 8)),
                        bus.get_byte(b0_u16 + (b1_u16 << 8) + 1),
                    )
                };
                Address::Ram(indirect)
            }
        };
        (address, extra_cycle)
    }

    fn load_from_address(&self, bus: &mut CpuBus) -> u8 {
        match &self.address {
            Address::Implicit => panic!("load_from_address can't be used for implicit mode"),
            Address::Accumulator => self.a,
            Address::Immediate(i) => *i,
            Address::Ram(address) => bus.get_byte(*address),
            Address::Relative(_) => panic!("load_from_address can't be used for the Relative mode"),
        }
    }
    fn store_to_address(&mut self, byte: u8, bus: &mut CpuBus) {
        match &self.address {
            Address::Implicit => panic!("store_to_address can't be used for implicit mode"),
            Address::Accumulator => self.a = byte,
            Address::Immediate(_) => panic!("Not possible to store in Immediate addressing"),
            Address::Ram(address) => bus.store_byte(*address, byte),
            Address::Relative(_) => panic!("store_to_address can't be used for the Relative mode"),
        }
    }

    fn get_ram_address(&self) -> u16 {
        match &self.address {
            Address::Ram(address) => *address,
            _ => panic!("Invalid address type {:?}", self.address),
        }
    }

    fn _adc(&mut self, m: u8) {
        let result = m as u16 + self.a as u16 + self.carry() as u16;
        self.set_or_clear_flag(
            ProcessorFlag::OverflowFlag,
            m & 0x80 == self.a & 0x80 && result & 0x80 != m as u16 & 0x80,
        );
        self.a = (result & 0x00FF) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, (self.a as i8) < 0);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, result & 0xFF00 != 0);
    }

    fn adc(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self._adc(m);
    }

    fn sbc(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self._adc(!m);
    }

    fn asl(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        let old_bit_7 = m & 0x80;
        m <<= 1;
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.store_to_address(m, bus);
    }

    fn and(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.a &= m;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ora(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.a |= m;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn eor(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.a ^= m;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ror(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        let old_bit_0 = m & 0x1;
        m = m >> 1 | (self.carry() << 7);
        self.store_to_address(m, bus);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
    }

    fn rol(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        let old_bit_7 = m & 0x80;
        m = m << 1 | self.carry();
        self.store_to_address(m, bus);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
    }

    fn lsr(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        let old_bit_0 = m & 1;
        m >>= 1;
        self.store_to_address(m, bus);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
    }

    fn nop(&mut self, _bus: &mut CpuBus) {}

    fn update_pc_for_brk_or_irq(&mut self, bus: &mut CpuBus) {
        let new_pc = if self.is_brk_or_irq_hijacked_by_nmi {
            bus.get_word(0xFFFA)
        } else {
            bus.get_word(0xFFFE)
        };
        if new_pc > 0 {
            self.pc = new_pc - 1;
        } else {
            self.pc = u16::MAX;
        }
        self.is_brk_or_irq_hijacked_by_nmi = false;
    }

    fn brk(&mut self, bus: &mut CpuBus) {
        self.push_word(self.pc + 2, bus);
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_byte(ps, bus);
        self.set_flag(ProcessorFlag::InterruptDisable);
        self.update_pc_for_brk_or_irq(bus);
    }

    fn irq(&mut self, bus: &mut CpuBus) {
        self.push_word(self.pc, bus);
        let mut ps = self.ps;
        ps &= !(ProcessorFlag::BFlagBit4 as u8);
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_byte(ps, bus);
        self.set_flag(ProcessorFlag::InterruptDisable);
        self.update_pc_for_brk_or_irq(bus);
    }

    fn nmi(&mut self, bus: &mut CpuBus) {
        self.push_word(self.pc, bus);
        let mut ps = self.ps;
        ps &= !(ProcessorFlag::BFlagBit4 as u8);
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_byte(ps, bus);
        self.set_flag(ProcessorFlag::InterruptDisable);
        self.pc = bus.get_word(0xFFFA) - 1;
    }

    fn jsr(&mut self, bus: &mut CpuBus) {
        self.push_word(self.pc + 2, bus);
        self.pc = self.get_ram_address() - 3;
    }

    fn rts(&mut self, bus: &mut CpuBus) {
        self.pc = self.pop_word(bus);
    }

    fn rti_restore_ps(&mut self, bus: &mut CpuBus) {
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_byte(bus);
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
    }

    fn rti(&mut self, bus: &mut CpuBus) {
        let popped = self.pop_word(bus);
        if popped == 0 {
            println!("what?");
        }

        self.pc = popped - 1;
    }

    fn jmp(&mut self, _bus: &mut CpuBus) {
        self.pc = self.get_ram_address() - 3;
    }

    fn sei(&mut self, _bus: &mut CpuBus) {
        self.set_flag(ProcessorFlag::InterruptDisable);
    }

    fn cld(&mut self, _bus: &mut CpuBus) {
        self.clear_flag(ProcessorFlag::DecimalMode);
    }

    fn cli(&mut self, _bus: &mut CpuBus) {
        self.clear_flag(ProcessorFlag::InterruptDisable);
    }

    fn clv(&mut self, _bus: &mut CpuBus) {
        self.clear_flag(ProcessorFlag::OverflowFlag);
    }

    fn lda(&mut self, bus: &mut CpuBus) {
        self.a = self.load_from_address(bus);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ldx(&mut self, bus: &mut CpuBus) {
        self.x = self.load_from_address(bus);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn ldy(&mut self, bus: &mut CpuBus) {
        self.y = self.load_from_address(bus);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn sta(&mut self, bus: &mut CpuBus) {
        self.store_to_address(self.a, bus);
    }

    fn sty(&mut self, bus: &mut CpuBus) {
        self.store_to_address(self.y, bus);
    }

    fn stx(&mut self, bus: &mut CpuBus) {
        self.store_to_address(self.x, bus);
    }

    fn txs(&mut self, _bus: &mut CpuBus) {
        self.sp = self.x;
    }

    fn tsx(&mut self, _bus: &mut CpuBus) {
        self.x = self.sp;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn txa(&mut self, _bus: &mut CpuBus) {
        self.a = self.x;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn tax(&mut self, _bus: &mut CpuBus) {
        self.x = self.a;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn tya(&mut self, _bus: &mut CpuBus) {
        self.a = self.y;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn tay(&mut self, _bus: &mut CpuBus) {
        self.y = self.a;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn branch_if(&mut self, condition: bool) {
        if condition {
            if let Address::Relative(new_pc) = self.address {
                self.pc = new_pc;
            } else {
                panic!("Unexpected address type in branch instruction.");
            }
        }
    }

    fn check_condition_for_beq(&self) -> bool {
        self.get_flag(ProcessorFlag::ZeroFlag)
    }

    fn beq(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_beq());
    }

    fn check_condition_for_bne(&self) -> bool {
        !self.check_condition_for_beq()
    }

    fn bne(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bne());
    }

    fn check_condition_for_bmi(&self) -> bool {
        self.get_flag(ProcessorFlag::NegativeFlag)
    }

    fn bmi(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bmi());
    }

    fn check_condition_for_bpl(&self) -> bool {
        !self.check_condition_for_bmi()
    }

    fn bpl(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bpl());
    }

    fn check_condition_for_bvs(&self) -> bool {
        self.get_flag(ProcessorFlag::OverflowFlag)
    }

    fn bvs(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bvs());
    }

    fn check_condition_for_bvc(&self) -> bool {
        !self.check_condition_for_bvs()
    }

    fn bvc(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bvc());
    }

    fn check_condition_for_bcs(&self) -> bool {
        self.get_flag(ProcessorFlag::CarryFlag)
    }

    fn bcs(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bcs());
    }

    fn check_condition_for_bcc(&self) -> bool {
        !self.check_condition_for_bcs()
    }

    fn bcc(&mut self, _bus: &mut CpuBus) {
        self.branch_if(self.check_condition_for_bcc());
    }

    fn bit(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a & m == 0);
        self.set_or_clear_flag(ProcessorFlag::OverflowFlag, m & (1 << 6) != 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & (1 << 7) != 0);
    }

    fn cmp(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        let result = (self.a as i16 - m as i16) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == m);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, self.a >= m);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn cpx(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        let result = (self.x as i16 - m as i16) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == m);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, self.x >= m);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn cpy(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        let result = (self.y as i16 - m as i16) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.y == m);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, self.y >= m);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn dey(&mut self, _bus: &mut CpuBus) {
        if self.y == 0 {
            self.y = 0xFF;
        } else {
            self.y -= 1;
        }
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn dec(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        if m == 0 {
            m = 0xFF;
        } else {
            m -= 1;
        }
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(m, bus);
    }

    fn dex(&mut self, _bus: &mut CpuBus) {
        if self.x == 0 {
            self.x = 0xFF;
        } else {
            self.x -= 1;
        }
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn pha(&mut self, bus: &mut CpuBus) {
        self.push_byte(self.a, bus);
    }

    fn php(&mut self, bus: &mut CpuBus) {
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_byte(ps, bus);
    }

    fn pla(&mut self, bus: &mut CpuBus) {
        self.a = self.pop_byte(bus);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn plp(&mut self, bus: &mut CpuBus) {
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_byte(bus);
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
    }

    fn inc(&mut self, bus: &mut CpuBus) {
        let mut m = self.load_from_address(bus);
        let result = m as u16 + 1;
        m = (result & 0xFF) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(m, bus);
    }

    fn inx(&mut self, _bus: &mut CpuBus) {
        let result = self.x as u16 + 1;
        self.x = (result & 0xFF) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn iny(&mut self, _bus: &mut CpuBus) {
        let result = self.y as u16 + 1;
        self.y = (result & 0xFF) as u8;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn clc(&mut self, _bus: &mut CpuBus) {
        self.clear_flag(ProcessorFlag::CarryFlag);
    }

    fn sec(&mut self, _bus: &mut CpuBus) {
        self.set_flag(ProcessorFlag::CarryFlag);
    }

    fn sed(&mut self, _bus: &mut CpuBus) {
        self.set_flag(ProcessorFlag::DecimalMode);
    }

    fn lax(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.a = m;
        self.x = m;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
    }

    fn alr(&mut self, bus: &mut CpuBus) {
        self.and(bus);
        self.address = Address::Accumulator;
        self.lsr(bus);
    }

    fn anc(&mut self, bus: &mut CpuBus) {
        self.and(bus);
        self.set_or_clear_flag(
            ProcessorFlag::CarryFlag,
            self.get_flag(ProcessorFlag::NegativeFlag),
        );
    }

    fn arr(&mut self, bus: &mut CpuBus) {
        self.and(bus);
        self.address = Address::Accumulator;
        self.ror(bus);
    }

    fn axa(&mut self, bus: &mut CpuBus) {
        self.and(bus);
        self.lsr(bus);
    }

    fn dcp(&mut self, bus: &mut CpuBus) {
        self.dec(bus);
        self.cmp(bus);
    }

    fn isc(&mut self, bus: &mut CpuBus) {
        self.inc(bus);
        self.sbc(bus);
    }

    fn las(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        let result = m & self.x & self.sp;
        self.a = result;
        self.x = result;
        self.sp = result;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, result == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn oal(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        self.a &= m;
        self.sp = self.a;
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn sbx(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus);
        let ax = self.a & self.x;
        let result = ax.wrapping_sub(m);
        self.set_or_clear_flag(ProcessorFlag::CarryFlag, ax >= m);
        self.set_or_clear_flag(ProcessorFlag::ZeroFlag, result == 0);
        self.set_or_clear_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
        self.x = result;
    }

    fn sax(&mut self, bus: &mut CpuBus) {
        let result = self.a & self.x;
        self.store_to_address(result, bus);
    }

    fn slo(&mut self, bus: &mut CpuBus) {
        self.asl(bus);
        self.ora(bus);
    }

    fn rla(&mut self, bus: &mut CpuBus) {
        self.rol(bus);
        self.and(bus);
    }

    fn say(&mut self, bus: &mut CpuBus) {
        let result = self.a & self.y;
        self.store_to_address(result, bus);
    }

    fn sre(&mut self, bus: &mut CpuBus) {
        self.lsr(bus);
        self.eor(bus);
    }

    fn rra(&mut self, bus: &mut CpuBus) {
        self.ror(bus);
        self.adc(bus);
    }

    fn tas(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus) as u16;
        let result = self.a & self.x;
        self.sp = result;
        let addr = m & 0xFF00 | (self.sp as u16);
        bus.store_byte(addr, result);
    }

    fn xaa(&mut self, bus: &mut CpuBus) {
        self.txa(bus);
        self.and(bus);
    }

    fn xas(&mut self, bus: &mut CpuBus) {
        let m = self.load_from_address(bus) as u16;
        let result = self.a & self.x;
        let addr = m & 0xFF00 | (self.sp as u16);
        bus.store_byte(addr, result);
    }
}
