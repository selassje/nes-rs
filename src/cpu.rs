use crate::common::*;
use crate::memory::CpuMemory;
use spin_sleep::SpinSleeper;
use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;
use std::time::{Duration, Instant};

const NANOS_PER_CPU_CYCLE: u128 = 559;

const DEBUG: bool = false;

const STACK_PAGE: u16 = 0x0100;

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
enum Address {
    Implicit,
    Accumulator,
    Immediate(u8),
    RAM(u16),
    Relative(u16),
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Address::Implicit => write!(f, ""),
            Address::Accumulator => write!(f, "A"),
            Address::Immediate(i) => write!(f, "{:#X}", i),
            Address::RAM(address) => write!(f, "{:#X}", address),
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

pub struct CPU {
    pc: u16,
    sp: u8,
    ps: u8,
    a: u8,
    x: u8,
    y: u8,
    ram: Rc<RefCell<dyn CpuMemory>>,
    code_segment: (u16, u16),
}

impl CPU {
    pub fn new(ram: Rc<RefCell<dyn CpuMemory>>) -> CPU {
        CPU {
            pc: 0,
            sp: 0xFD,
            ps: 0x24,
            a: 0,
            x: 0,
            y: 0,
            ram: ram,
            code_segment: (0, 0),
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.ram.borrow().get_word(0xFFFC);
        self.sp = 0xFF;
        self.ps = 0x24;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.code_segment = self.ram.borrow().get_code_segment();
        self.code_segment = (0, 0xFFFF);
    }

    fn set_flag(&mut self, flag: ProcessorFlag) {
        self.ps |= flag as u8;
    }

    fn reset_flag(&mut self, flag: ProcessorFlag) {
        self.ps &= !(flag as u8);
    }

    fn get_flag(&mut self, flag: ProcessorFlag) -> bool {
        (self.ps & (flag as u8)) != 0
    }

    fn set_or_reset_flag(&mut self, flag: ProcessorFlag, cond: bool) {
        if cond {
            self.set_flag(flag);
        } else {
            self.reset_flag(flag);
        }
    }

    fn carry(&mut self) -> u8 {
        if self.get_flag(ProcessorFlag::CarryFlag) {
            1
        } else {
            0
        }
    }

    fn push_u8(&mut self, val: u8) {
        self.ram
            .borrow_mut()
            .store_byte(self.sp as u16 + STACK_PAGE, val);
        self.sp -= 1;
    }

    fn push_u16(&mut self, val: u16) {
        let addr = self.sp as u16 + STACK_PAGE - 1;
        self.ram.borrow_mut().store_word(addr, val);
        self.sp -= 2;
    }

    fn pop_u8(&mut self) -> u8 {
        self.sp += 1;
        self.ram.borrow().get_byte(self.sp as u16 + STACK_PAGE)
    }

    fn pop_u16(&mut self) -> u16 {
        self.sp += 2;
        let addr = self.sp as u16 + STACK_PAGE - 1;
        self.ram.borrow().get_word(addr)
    }

    pub fn run_single_instruction(&mut self) -> Option<u8> {
        let (code_segment_start, code_segment_end) = self.code_segment;
        if self.pc >= code_segment_start && self.pc <= code_segment_end {
            let op = self.ram.borrow().get_byte(self.pc);
            let mut b0 = 0;
            let mut b1 = 0;
            if self.pc + 1 <= code_segment_end {
                b0 = self.ram.borrow().get_byte(self.pc + 1);
            }
            if self.pc + 2 <= code_segment_end {
                b1 = self.ram.borrow().get_byte(self.pc + 2);
            }
            //println!("{:X} {:X} {:X} {:X} \t\tA:{:X} X:{:X} Y:{:X} P:{:X} SP={:X}",self.pc, op, b0,b1,self.a, self.x, self.y, self.ps, self.sp);
            let (bytes, cycles) = self.execute_instruction(op, b0, b1);
            self.pc += bytes as u16;
            Some(cycles)
        } else {
            println!("CPU Stopped execution at {:X}", self.pc);
            None
        }
    }

    fn execute_instruction(&mut self, op: u8, b0: u8, b1: u8) -> (u8, u8) {
        match op {
            0x00 => (1, 7 + self.brk(b0, b1, AddressingMode::Implicit)),

            0x69 => (2, 2 + self.adc(b0, b1, AddressingMode::Immediate)),
            0x65 => (2, 3 + self.adc(b0, b1, AddressingMode::ZeroPage)),
            0x75 => (2, 4 + self.adc(b0, b1, AddressingMode::ZeroPageX)),
            0x6D => (3, 4 + self.adc(b0, b1, AddressingMode::Absolute)),
            0x7D => (3, 4 + self.adc(b0, b1, AddressingMode::AbsoluteX)),
            0x79 => (3, 4 + self.adc(b0, b1, AddressingMode::AbsoluteY)),
            0x61 => (2, 6 + self.adc(b0, b1, AddressingMode::IndexedIndirectX)),
            0x71 => (2, 5 + self.adc(b0, b1, AddressingMode::IndirectIndexedY)),

            0x29 => (2, 2 + self.and(b0, b1, AddressingMode::Immediate)),
            0x25 => (2, 3 + self.and(b0, b1, AddressingMode::ZeroPage)),
            0x35 => (2, 4 + self.and(b0, b1, AddressingMode::ZeroPageX)),
            0x2D => (3, 4 + self.and(b0, b1, AddressingMode::Absolute)),
            0x3D => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteX)),
            0x39 => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteY)),
            0x21 => (2, 6 + self.and(b0, b1, AddressingMode::IndexedIndirectX)),
            0x31 => (2, 5 + self.and(b0, b1, AddressingMode::IndirectIndexedY)),

            0x0A => (1, 2 + self.asl(b0, b1, AddressingMode::Accumulator)),
            0x06 => (2, 5 + self.asl(b0, b1, AddressingMode::ZeroPage)),
            0x16 => (2, 6 + self.asl(b0, b1, AddressingMode::ZeroPageX)),
            0x0E => (3, 6 + self.asl(b0, b1, AddressingMode::Absolute)),
            0x1E => (3, 7 + self.asl(b0, b1, AddressingMode::AbsoluteX)),

            0x90 => (2, 2 + self.bcc(b0, b1, AddressingMode::Relative)),
            0xB0 => (2, 2 + self.bcs(b0, b1, AddressingMode::Relative)),
            0xF0 => (2, 2 + self.beq(b0, b1, AddressingMode::Relative)),
            0x30 => (2, 2 + self.bmi(b0, b1, AddressingMode::Relative)),
            0xD0 => (2, 2 + self.bne(b0, b1, AddressingMode::Relative)),
            0x10 => (2, 2 + self.bpl(b0, b1, AddressingMode::Relative)),
            0x50 => (2, 2 + self.bvc(b0, b1, AddressingMode::Relative)),
            0x70 => (2, 2 + self.bvs(b0, b1, AddressingMode::Relative)),

            0x24 => (2, 3 + self.bit(b0, b1, AddressingMode::ZeroPage)),
            0x2C => (3, 5 + self.bit(b0, b1, AddressingMode::Absolute)),

            0x18 => (1, 2 + self.clc(b0, b1, AddressingMode::Implicit)),
            0xD8 => (1, 2 + self.cld(b0, b1, AddressingMode::Implicit)),
            0x58 => (1, 2 + self.cli(b0, b1, AddressingMode::Implicit)),
            0xB8 => (1, 2 + self.clv(b0, b1, AddressingMode::Implicit)),

            0xC9 => (2, 2 + self.cmp(b0, b1, AddressingMode::Immediate)),
            0xC5 => (2, 3 + self.cmp(b0, b1, AddressingMode::ZeroPage)),
            0xD5 => (2, 4 + self.cmp(b0, b1, AddressingMode::ZeroPageX)),
            0xCD => (3, 4 + self.cmp(b0, b1, AddressingMode::Absolute)),
            0xDD => (3, 4 + self.cmp(b0, b1, AddressingMode::AbsoluteX)),
            0xD9 => (3, 4 + self.cmp(b0, b1, AddressingMode::AbsoluteY)),
            0xC1 => (2, 6 + self.cmp(b0, b1, AddressingMode::IndexedIndirectX)),
            0xD1 => (2, 5 + self.cmp(b0, b1, AddressingMode::IndirectIndexedY)),

            0xE0 => (2, 2 + self.cpx(b0, b1, AddressingMode::Immediate)),
            0xE4 => (2, 3 + self.cpx(b0, b1, AddressingMode::ZeroPage)),
            0xEC => (3, 4 + self.cpx(b0, b1, AddressingMode::Absolute)),

            0xC0 => (2, 2 + self.cpy(b0, b1, AddressingMode::Immediate)),
            0xC4 => (2, 3 + self.cpy(b0, b1, AddressingMode::ZeroPage)),
            0xCC => (3, 4 + self.cpy(b0, b1, AddressingMode::Absolute)),

            0xC6 => (2, 5 + self.dec(b0, b1, AddressingMode::ZeroPage)),
            0xD6 => (2, 6 + self.dec(b0, b1, AddressingMode::ZeroPageX)),
            0xCE => (3, 6 + self.dec(b0, b1, AddressingMode::Absolute)),
            0xDE => (3, 7 + self.dec(b0, b1, AddressingMode::AbsoluteX)),

            0xCA => (1, 2 + self.dex(b0, b1, AddressingMode::Implicit)),
            0x88 => (1, 2 + self.dey(b0, b1, AddressingMode::Implicit)),

            0x49 => (2, 2 + self.eor(b0, b1, AddressingMode::Immediate)),
            0x45 => (2, 3 + self.eor(b0, b1, AddressingMode::ZeroPage)),
            0x55 => (2, 4 + self.eor(b0, b1, AddressingMode::ZeroPageX)),
            0x4D => (3, 4 + self.eor(b0, b1, AddressingMode::Absolute)),
            0x5D => (3, 4 + self.eor(b0, b1, AddressingMode::AbsoluteX)),
            0x59 => (3, 4 + self.eor(b0, b1, AddressingMode::AbsoluteY)),
            0x41 => (2, 6 + self.eor(b0, b1, AddressingMode::IndexedIndirectX)),
            0x51 => (2, 5 + self.eor(b0, b1, AddressingMode::IndirectIndexedY)),

            0xE6 => (2, 5 + self.inc(b0, b1, AddressingMode::ZeroPage)),
            0xF6 => (2, 6 + self.inc(b0, b1, AddressingMode::ZeroPageX)),
            0xEE => (3, 6 + self.inc(b0, b1, AddressingMode::Absolute)),
            0xFE => (3, 7 + self.inc(b0, b1, AddressingMode::AbsoluteX)),

            0xC8 => (1, 2 + self.iny(b0, b1, AddressingMode::Implicit)),
            0xE8 => (1, 2 + self.inx(b0, b1, AddressingMode::Implicit)),

            0x4C => (3, 3 + self.jmp(b0, b1, AddressingMode::Absolute)),
            0x6C => (3, 5 + self.jmp(b0, b1, AddressingMode::Indirect)),

            0x20 => (3, 6 + self.jsr(b0, b1, AddressingMode::Absolute)),

            0xA9 => (2, 2 + self.lda(b0, b1, AddressingMode::Immediate)),
            0xA5 => (2, 3 + self.lda(b0, b1, AddressingMode::ZeroPage)),
            0xB5 => (2, 4 + self.lda(b0, b1, AddressingMode::ZeroPageX)),
            0xAD => (3, 4 + self.lda(b0, b1, AddressingMode::Absolute)),
            0xBD => (3, 4 + self.lda(b0, b1, AddressingMode::AbsoluteX)),
            0xB9 => (3, 4 + self.lda(b0, b1, AddressingMode::AbsoluteY)),
            0xA1 => (2, 6 + self.lda(b0, b1, AddressingMode::IndexedIndirectX)),
            0xB1 => (2, 5 + self.lda(b0, b1, AddressingMode::IndirectIndexedY)),

            0xA2 => (2, 2 + self.ldx(b0, b1, AddressingMode::Immediate)),
            0xA6 => (2, 2 + self.ldx(b0, b1, AddressingMode::ZeroPage)),
            0xB6 => (2, 2 + self.ldx(b0, b1, AddressingMode::ZeroPageY)),
            0xAE => (3, 3 + self.ldx(b0, b1, AddressingMode::Absolute)),
            0xBE => (3, 3 + self.ldx(b0, b1, AddressingMode::AbsoluteY)),

            0xA0 => (2, 2 + self.ldy(b0, b1, AddressingMode::Immediate)),
            0xA4 => (2, 2 + self.ldy(b0, b1, AddressingMode::ZeroPage)),
            0xB4 => (2, 2 + self.ldy(b0, b1, AddressingMode::ZeroPageX)),
            0xAC => (3, 3 + self.ldy(b0, b1, AddressingMode::Absolute)),
            0xBC => (3, 3 + self.ldy(b0, b1, AddressingMode::AbsoluteX)),

            0x4A => (1, 2 + self.lsr(b0, b1, AddressingMode::Accumulator)),
            0x46 => (2, 5 + self.lsr(b0, b1, AddressingMode::ZeroPage)),
            0x56 => (2, 6 + self.lsr(b0, b1, AddressingMode::ZeroPageX)),
            0x4E => (3, 6 + self.lsr(b0, b1, AddressingMode::Absolute)),
            0x5E => (3, 7 + self.lsr(b0, b1, AddressingMode::AbsoluteX)),

            0xEA => (1, 2 + self.nop(b0, b1, AddressingMode::Implicit)),

            0x09 => (2, 2 + self.ora(b0, b1, AddressingMode::Immediate)),
            0x05 => (2, 3 + self.ora(b0, b1, AddressingMode::ZeroPage)),
            0x15 => (2, 4 + self.ora(b0, b1, AddressingMode::ZeroPageX)),
            0x0D => (3, 4 + self.ora(b0, b1, AddressingMode::Absolute)),
            0x1D => (3, 4 + self.ora(b0, b1, AddressingMode::AbsoluteX)),
            0x19 => (3, 4 + self.ora(b0, b1, AddressingMode::AbsoluteY)),
            0x01 => (2, 6 + self.ora(b0, b1, AddressingMode::IndexedIndirectX)),
            0x11 => (2, 5 + self.ora(b0, b1, AddressingMode::IndirectIndexedY)),

            0x48 => (1, 3 + self.pha(b0, b1, AddressingMode::Implicit)),
            0x08 => (1, 3 + self.php(b0, b1, AddressingMode::Implicit)),
            0x68 => (1, 3 + self.pla(b0, b1, AddressingMode::Implicit)),
            0x28 => (1, 3 + self.plp(b0, b1, AddressingMode::Implicit)),

            0x2A => (1, 2 + self.rol(b0, b1, AddressingMode::Accumulator)),
            0x26 => (2, 5 + self.rol(b0, b1, AddressingMode::ZeroPage)),
            0x36 => (2, 6 + self.rol(b0, b1, AddressingMode::ZeroPageX)),
            0x2E => (3, 6 + self.rol(b0, b1, AddressingMode::Absolute)),
            0x3E => (3, 7 + self.rol(b0, b1, AddressingMode::AbsoluteX)),

            0x6A => (1, 2 + self.ror(b0, b1, AddressingMode::Accumulator)),
            0x66 => (2, 5 + self.ror(b0, b1, AddressingMode::ZeroPage)),
            0x76 => (2, 6 + self.ror(b0, b1, AddressingMode::ZeroPageX)),
            0x6E => (3, 6 + self.ror(b0, b1, AddressingMode::Absolute)),
            0x7E => (3, 7 + self.ror(b0, b1, AddressingMode::AbsoluteX)),

            0x40 => (1, 6 + self.rti(b0, b1, AddressingMode::Implicit)),
            0x60 => (1, 6 + self.rts(b0, b1, AddressingMode::Implicit)),

            0xE9 => (2, 2 + self.sbc(b0, b1, AddressingMode::Immediate)),
            0xE5 => (2, 3 + self.sbc(b0, b1, AddressingMode::ZeroPage)),
            0xF5 => (2, 4 + self.sbc(b0, b1, AddressingMode::ZeroPageX)),
            0xED => (3, 4 + self.sbc(b0, b1, AddressingMode::Absolute)),
            0xFD => (3, 4 + self.sbc(b0, b1, AddressingMode::AbsoluteX)),
            0xF9 => (3, 4 + self.sbc(b0, b1, AddressingMode::AbsoluteY)),
            0xE1 => (2, 6 + self.sbc(b0, b1, AddressingMode::IndexedIndirectX)),
            0xF1 => (2, 5 + self.sbc(b0, b1, AddressingMode::IndirectIndexedY)),

            0x38 => (1, 2 + self.sec(b0, b1, AddressingMode::Implicit)),
            0xF8 => (1, 2 + self.sed(b0, b1, AddressingMode::Implicit)),
            0x78 => (1, 2 + self.sei(b0, b1, AddressingMode::Implicit)),

            0x85 => (2, 3 + self.sta(b0, b1, AddressingMode::ZeroPage)),
            0x95 => (2, 4 + self.sta(b0, b1, AddressingMode::ZeroPageX)),
            0x8D => (3, 4 + self.sta(b0, b1, AddressingMode::Absolute)),
            0x9D => (3, 5 + self.sta(b0, b1, AddressingMode::AbsoluteX)),
            0x99 => (3, 5 + self.sta(b0, b1, AddressingMode::AbsoluteY)),
            0x81 => (2, 6 + self.sta(b0, b1, AddressingMode::IndexedIndirectX)),
            0x91 => (2, 6 + self.sta(b0, b1, AddressingMode::IndirectIndexedY)),

            0x84 => (2, 3 + self.sty(b0, b1, AddressingMode::ZeroPage)),
            0x94 => (2, 4 + self.sty(b0, b1, AddressingMode::ZeroPageX)),
            0x8C => (3, 4 + self.sty(b0, b1, AddressingMode::Absolute)),

            0x86 => (2, 3 + self.stx(b0, b1, AddressingMode::ZeroPage)),
            0x96 => (2, 4 + self.stx(b0, b1, AddressingMode::ZeroPageY)),
            0x8E => (3, 4 + self.stx(b0, b1, AddressingMode::Absolute)),

            0xAA => (1, 2 + self.tax(b0, b1, AddressingMode::Implicit)),
            0xA8 => (1, 2 + self.tay(b0, b1, AddressingMode::Implicit)),
            0xBA => (1, 2 + self.tsx(b0, b1, AddressingMode::Implicit)),
            0x8A => (1, 2 + self.txa(b0, b1, AddressingMode::Implicit)),
            0x9A => (1, 2 + self.txs(b0, b1, AddressingMode::Implicit)),
            0x98 => (1, 2 + self.tya(b0, b1, AddressingMode::Implicit)),

            /*ILLEGAL OPPCODES */
            /*NOP*/
            0x1A => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0x3A => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0x5A => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0x7A => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0xDA => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0xFA => (1, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            /*DOP*/
            0x04 => (2, 3 + self.nop(b0, b1, AddressingMode::ZeroPage)),
            0x14 => (2, 4 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            0x34 => (2, 4 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            0x44 => (2, 3 + self.nop(b0, b1, AddressingMode::ZeroPage)),
            0x54 => (2, 4 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            0x64 => (2, 3 + self.nop(b0, b1, AddressingMode::ZeroPage)),
            0x74 => (2, 4 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            0x80 => (2, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0x82 => (2, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0x89 => (2, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0xC2 => (2, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0xD4 => (2, 4 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            0xE2 => (2, 2 + self.nop(b0, b1, AddressingMode::Immediate)),
            0xF4 => (2, 3 + self.nop(b0, b1, AddressingMode::ZeroPageX)),
            /*TOP*/
            0x0C => (3, 4 + self.nop(b0, b1, AddressingMode::Absolute)),
            0x1C => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),
            0x3C => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),
            0x5C => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),
            0x7C => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),
            0xDC => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),
            0xFC => (3, 4 + self.nop(b0, b1, AddressingMode::AbsoluteX)),

            0xA7 => (2, 3 + self.lax(b0, b1, AddressingMode::ZeroPage)),
            0xB7 => (2, 4 + self.lax(b0, b1, AddressingMode::ZeroPageY)),
            0xAF => (3, 4 + self.lax(b0, b1, AddressingMode::Absolute)),
            0xBF => (3, 4 + self.lax(b0, b1, AddressingMode::AbsoluteY)),
            0xA3 => (2, 6 + self.lax(b0, b1, AddressingMode::IndexedIndirectX)),
            0xB3 => (2, 5 + self.lax(b0, b1, AddressingMode::IndirectIndexedY)),

            0x87 => (2, 3 + self.aax(b0, b1, AddressingMode::ZeroPage)),
            0x97 => (2, 4 + self.aax(b0, b1, AddressingMode::ZeroPageY)),
            0x83 => (2, 6 + self.aax(b0, b1, AddressingMode::IndexedIndirectX)),
            0x8F => (3, 4 + self.aax(b0, b1, AddressingMode::Absolute)),

            0xEB => (2, 2 + self.sbc(b0, b1, AddressingMode::Immediate)),

            0xC7 => (2, 5 + self.dcp(b0, b1, AddressingMode::ZeroPage)),
            0xD7 => (2, 6 + self.dcp(b0, b1, AddressingMode::ZeroPageX)),
            0xCF => (3, 6 + self.dcp(b0, b1, AddressingMode::Absolute)),
            0xDF => (3, 7 + self.dcp(b0, b1, AddressingMode::AbsoluteX)),
            0xDB => (3, 7 + self.dcp(b0, b1, AddressingMode::AbsoluteY)),
            0xC3 => (2, 8 + self.dcp(b0, b1, AddressingMode::IndexedIndirectX)),
            0xD3 => (2, 8 + self.dcp(b0, b1, AddressingMode::IndirectIndexedY)),

            0xE7 => (2, 5 + self.isc(b0, b1, AddressingMode::ZeroPage)),
            0xF7 => (2, 6 + self.isc(b0, b1, AddressingMode::ZeroPageX)),
            0xEF => (3, 6 + self.isc(b0, b1, AddressingMode::Absolute)),
            0xFF => (3, 7 + self.isc(b0, b1, AddressingMode::AbsoluteX)),
            0xFB => (3, 7 + self.isc(b0, b1, AddressingMode::AbsoluteY)),
            0xE3 => (2, 8 + self.isc(b0, b1, AddressingMode::IndexedIndirectX)),
            0xF3 => (2, 8 + self.isc(b0, b1, AddressingMode::IndirectIndexedY)),

            0x07 => (2, 5 + self.slo(b0, b1, AddressingMode::ZeroPage)),
            0x17 => (2, 6 + self.slo(b0, b1, AddressingMode::ZeroPageX)),
            0x0F => (3, 6 + self.slo(b0, b1, AddressingMode::Absolute)),
            0x1F => (3, 7 + self.slo(b0, b1, AddressingMode::AbsoluteX)),
            0x1B => (3, 7 + self.slo(b0, b1, AddressingMode::AbsoluteY)),
            0x03 => (2, 8 + self.slo(b0, b1, AddressingMode::IndexedIndirectX)),
            0x13 => (2, 8 + self.slo(b0, b1, AddressingMode::IndirectIndexedY)),

            0x27 => (2, 5 + self.rla(b0, b1, AddressingMode::ZeroPage)),
            0x37 => (2, 6 + self.rla(b0, b1, AddressingMode::ZeroPageX)),
            0x2F => (3, 6 + self.rla(b0, b1, AddressingMode::Absolute)),
            0x3F => (3, 7 + self.rla(b0, b1, AddressingMode::AbsoluteX)),
            0x3B => (3, 7 + self.rla(b0, b1, AddressingMode::AbsoluteY)),
            0x23 => (2, 8 + self.rla(b0, b1, AddressingMode::IndexedIndirectX)),
            0x33 => (2, 8 + self.rla(b0, b1, AddressingMode::IndirectIndexedY)),

            0x47 => (2, 5 + self.sre(b0, b1, AddressingMode::ZeroPage)),
            0x57 => (2, 6 + self.sre(b0, b1, AddressingMode::ZeroPageX)),
            0x4F => (3, 6 + self.sre(b0, b1, AddressingMode::Absolute)),
            0x5F => (3, 7 + self.sre(b0, b1, AddressingMode::AbsoluteX)),
            0x5B => (3, 7 + self.sre(b0, b1, AddressingMode::AbsoluteY)),
            0x43 => (2, 8 + self.sre(b0, b1, AddressingMode::IndexedIndirectX)),
            0x53 => (2, 8 + self.sre(b0, b1, AddressingMode::IndirectIndexedY)),

            0x67 => (2, 5 + self.rra(b0, b1, AddressingMode::ZeroPage)),
            0x77 => (2, 6 + self.rra(b0, b1, AddressingMode::ZeroPageX)),
            0x6F => (3, 6 + self.rra(b0, b1, AddressingMode::Absolute)),
            0x7F => (3, 7 + self.rra(b0, b1, AddressingMode::AbsoluteX)),
            0x7B => (3, 7 + self.rra(b0, b1, AddressingMode::AbsoluteY)),
            0x63 => (2, 8 + self.rra(b0, b1, AddressingMode::IndexedIndirectX)),
            0x73 => (2, 8 + self.rra(b0, b1, AddressingMode::IndirectIndexedY)),

            _ => panic!("Unknown instruction {:#04x} at {:#X} ", op, self.pc),
        }
    }

    fn get_address_and_cycles(&self, b0: u8, b1: u8, mode: AddressingMode) -> (Address, u8) {
        let b0_u16 = b0 as u16;
        let b1_u16 = b1 as u16;
        let x_u16 = self.x as u16;
        let y_u16 = self.y as u16;
        let zero_page_x = (b0_u16 + x_u16) & 0xFF;
        let zero_page_x_hi = (b0_u16 + x_u16 + 1) & 0xFF;
        let zero_page_y = (b0_u16 + y_u16) & 0xFF;
        let mut add_cycles = 0;

        match mode {
            AddressingMode::Implicit => (Address::Implicit, 0),
            AddressingMode::Accumulator => (Address::Accumulator, 0),
            AddressingMode::Immediate => (Address::Immediate(b0), 0),
            AddressingMode::ZeroPage => (Address::RAM(b0_u16), 0),
            AddressingMode::ZeroPageX => (Address::RAM(zero_page_x), 0),
            AddressingMode::ZeroPageY => (Address::RAM(zero_page_y), 0),
            AddressingMode::Absolute => (Address::RAM(convert_2u8_to_u16(b0, b1)), 0),
            AddressingMode::AbsoluteX => {
                if b0_u16 + x_u16 > 0xFF {
                    add_cycles = 1
                }
                (Address::RAM(b0_u16 + x_u16 + (b1_u16 << 8)), add_cycles)
            }
            AddressingMode::AbsoluteY => {
                if b0_u16 + y_u16 > 0xFF {
                    add_cycles = 1
                }
                (Address::RAM(b0_u16 + y_u16 + (b1_u16 << 8)), add_cycles)
            }
            AddressingMode::IndexedIndirectX => {
                let indexed_indirect = convert_2u8_to_u16(
                    self.ram.borrow().get_byte(zero_page_x),
                    self.ram.borrow().get_byte(zero_page_x_hi),
                );
                (Address::RAM(indexed_indirect), 0)
            }
            AddressingMode::IndirectIndexedY => {
                let indirect = convert_2u8_to_u16(
                    self.ram.borrow().get_byte(b0_u16),
                    self.ram.borrow().get_byte((b0_u16 + 1) & 0xFF),
                );
                let indirect_indexed = indirect + y_u16;
                if indirect_indexed & 0xFF00 > indirect & 0xFF00 {
                    add_cycles = 1;
                }

                (Address::RAM(indirect_indexed), add_cycles)
            }
            AddressingMode::Relative => {
                let new_pc = (self.pc as i16 + (b0 as i8 as i16)) as u16;
                if new_pc & 0xFF00 != self.pc & 0xFF00 {
                    add_cycles = 1
                }
                (Address::Relative(new_pc), add_cycles)
            }
            AddressingMode::Indirect => {
                let indirect = if b0_u16 == 0xFF {
                    convert_2u8_to_u16(
                        self.ram.borrow().get_byte(b0_u16 + (b1_u16 << 8)),
                        self.ram.borrow().get_byte(b1_u16 << 8),
                    )
                } else {
                    convert_2u8_to_u16(
                        self.ram.borrow().get_byte(b0_u16 + (b1_u16 << 8)),
                        self.ram.borrow().get_byte(b0_u16 + (b1_u16 << 8) + 1),
                    )
                };
                (Address::RAM(indirect), 0)
            }
        }
    }

    fn load_from_address(&self, address: &Address) -> u8 {
        match address {
            Address::Implicit => panic!("load_from_address can't be used for implicit mode"),
            Address::Accumulator => self.a,
            Address::Immediate(i) => *i,
            Address::RAM(address) => self.ram.borrow().get_byte(*address),
            Address::Relative(_) => panic!("load_from_address can't be used for the Relative mode"),
        }
    }

    fn get_ram_address(&self, address: &Address) -> u16 {
        match address {
            Address::RAM(address) => *address,
            _ => panic!("Invalid address type {:?}", address),
        }
    }

    fn store_to_address(&mut self, address: &Address, byte: u8) {
        match address {
            Address::Implicit => panic!("store_to_address can't be used for implicit mode"),
            Address::Accumulator => self.a = byte,
            Address::Immediate(_) => panic!("Not possible to store in Immediate addressing"),
            Address::RAM(address) => self.ram.borrow_mut().store_byte(*address, byte),
            Address::Relative(_) => panic!("store_to_address can't be used for the Relative mode"),
        }
    }

    fn _adc(&mut self, m: u8) {
        let result = m as u16 + self.a as u16 + self.carry() as u16;
        self.set_or_reset_flag(
            ProcessorFlag::OverflowFlag,
            m & 0x80 == self.a & 0x80 && result & 0x80 != m as u16 & 0x80,
        );
        self.a = (result & 0x00FF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, (self.a as i8) < 0);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, result & 0xFF00 != 0);
    }

    fn adc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self._adc(m);
        cycles
    }

    fn sbc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self._adc(!m);
        cycles
    }

    fn asl(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&m_address);
        let old_bit_7 = m & 0x80;
        m = m << 1;
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&m_address, m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        cycles
    }

    fn and(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self.a &= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn ora(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self.a |= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn eor(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self.a ^= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn ror(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&m_address);
        let old_bit_0 = m & 0x1;
        m = m >> 1 | (self.carry() << 7);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&m_address, m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        cycles
    }

    fn rol(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&m_address);
        let old_bit_7 = m & 0x80;
        m = m << 1 | self.carry();
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&m_address, m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        cycles
    }

    fn lsr(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&m_address);
        let old_bit_0 = m & 1;
        m >>= 1;
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&m_address, m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        cycles
    }

    fn nop(&mut self, _b0: u8, _b1: u8, _mode: AddressingMode) -> u8 {
        0
    }

    fn jsr(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.push_u16(self.pc + 2);
        self.pc = self.get_ram_address(&m_address) - 3;
        cycles
    }

    fn rts(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.pc = self.pop_u16();
        cycles
    }

    fn rti(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_u8();
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
        self.pc = self.pop_u16() - 1;
        cycles
    }

    fn jmp(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.pc = self.get_ram_address(&m_address) - 3;
        cycles
    }

    fn sei(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.set_flag(ProcessorFlag::InterruptDisable);
        cycles
    }

    fn cld(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.reset_flag(ProcessorFlag::DecimalMode);
        cycles
    }

    fn cli(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.reset_flag(ProcessorFlag::InterruptDisable);
        cycles
    }

    fn clv(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.reset_flag(ProcessorFlag::OverflowFlag);
        cycles
    }

    fn lda(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.a = self.load_from_address(&m_address);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn ldx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.x = self.load_from_address(&m_address);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
        cycles
    }

    fn ldy(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.y = self.load_from_address(&m_address);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
        cycles
    }

    fn sta(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.store_to_address(&m_address, self.a);
        cycles
    }

    fn sty(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.store_to_address(&m_address, self.y);
        cycles
    }

    fn stx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.store_to_address(&m_address, self.x);
        cycles
    }

    fn txs(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.sp = self.x;
        cycles
    }

    fn tsx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.x = self.sp;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
        cycles
    }

    fn txa(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.a = self.x;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn tax(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.x = self.a;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
        cycles
    }

    fn tya(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.a = self.y;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn tay(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.y = self.a;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
        cycles
    }

    fn beq(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if self.get_flag(ProcessorFlag::ZeroFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bne(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if !self.get_flag(ProcessorFlag::ZeroFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bpl(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if !self.get_flag(ProcessorFlag::NegativeFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bvc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if !self.get_flag(ProcessorFlag::OverflowFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bvs(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if self.get_flag(ProcessorFlag::OverflowFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bmi(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if self.get_flag(ProcessorFlag::NegativeFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bcc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if !self.get_flag(ProcessorFlag::CarryFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bcs(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        if self.get_flag(ProcessorFlag::CarryFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _ => panic!("Unexpected address type in beq."),
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn bit(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&address);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a & m == 0);
        self.set_or_reset_flag(ProcessorFlag::OverflowFlag, m & (1 << 6) != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & (1 << 7) != 0);
        cycles
    }

    fn cmp(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&address);
        let result = (self.a as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.a >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
        cycles
    }

    fn cpx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&address);
        let result = (self.x as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.x >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
        cycles
    }

    fn cpy(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&address);
        let result = (self.y as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.y >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
        cycles
    }

    fn dey(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        if self.y == 0 {
            self.y = 0xFF;
        } else {
            self.y = self.y - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
        cycles
    }

    fn dec(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&address);
        if m == 0 {
            m = 0xFF;
        } else {
            m = m - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&address, m);
        cycles
    }

    fn dex(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        if self.x == 0 {
            self.x = 0xFF;
        } else {
            self.x = self.x - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
        cycles
    }

    fn brk(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.push_u16(self.pc + 1);
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_u8(ps);
        self.pc = self.ram.borrow().get_word(0xFFFE) - 1;
        cycles
    }

    pub fn nmi(&mut self) -> u8 {
        self.push_u16(self.pc);
        let mut ps = self.ps;
        ps &= !(ProcessorFlag::BFlagBit4 as u8);
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_u8(ps);
        self.pc = self.ram.borrow().get_word(0xFFFA);
        7
    }

    fn pha(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.push_u8(self.a);
        cycles
    }

    fn php(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_u8(ps);
        cycles
    }

    fn pla(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.a = self.pop_u8();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
        cycles
    }

    fn plp(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_u8();
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
        cycles
    }

    fn inc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&address);
        let result = m as u16 + 1;
        m = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(&address, m);
        cycles
    }

    fn inx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let result = self.x as u16 + 1;
        self.x = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
        cycles
    }

    fn iny(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let result = self.y as u16 + 1;
        self.y = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
        cycles
    }

    fn clc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.reset_flag(ProcessorFlag::CarryFlag);
        cycles
    }

    fn sec(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.set_flag(ProcessorFlag::CarryFlag);
        cycles
    }

    fn sed(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.set_flag(ProcessorFlag::DecimalMode);
        cycles
    }

    fn lax(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&address);
        self.a = m;
        self.x = m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        cycles
    }

    fn aax(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let result = self.a & self.x;
        self.store_to_address(&address, result);
        cycles
    }

    fn dcp(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.dec(b0, b1, mode);
        self.cmp(b0, b1, mode);
        cycles
    }

    fn isc(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.inc(b0, b1, mode);
        self.sbc(b0, b1, mode);
        cycles
    }

    fn slo(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.asl(b0, b1, mode);
        self.ora(b0, b1, mode);
        cycles
    }

    fn rla(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.rol(b0, b1, mode);
        self.and(b0, b1, mode);
        cycles
    }

    fn sre(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.lsr(b0, b1, mode);
        self.eor(b0, b1, mode);
        cycles
    }

    fn rra(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.ror(b0, b1, mode);
        self.adc(b0, b1, mode);
        cycles
    }
}
