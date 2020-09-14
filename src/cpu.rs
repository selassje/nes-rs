mod opcodes;

use self::AddressingMode::*;
use crate::common::*;
use crate::memory::CpuMemory;
use opcodes::{get_opcodes, OpCode, OpCodes};
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
    cycles: u128,
    cycles_next: u16,
    opcode_next: u8,
    operand_1: u8,
    operand_2: u8,
    address: Address,
    ram: Rc<RefCell<dyn CpuMemory>>,
    code_segment: (u16, u16),
    opcodes: OpCodes,
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
            cycles: 0,
            cycles_next: 0,
            opcode_next: 0,
            operand_1: 0,
            operand_2: 0,
            ram: ram,
            code_segment: (0, 0),
            address: Address::Implicit,
            opcodes: get_opcodes(),
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0xC000;
        self.pc = self.ram.borrow().get_word(0xFFFC);
        self.sp = 0xFD;
        self.ps = 0x24;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.cycles = 0;
        self.cycles_next = 0;
        self.opcode_next = 0;
        self.operand_1 = 0;
        self.operand_2 = 0;
        self.code_segment = (0, 0xFFFF);
        self.address = Address::Implicit;
    }

    fn set_flag(&mut self, flag: ProcessorFlag) {
        self.ps |= flag as u8;
    }

    fn reset_flag(&mut self, flag: ProcessorFlag) {
        self.ps &= !(flag as u8);
    }

    fn get_flag(&self, flag: ProcessorFlag) -> bool {
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

    pub fn fetch_next_instruction(&mut self) -> u16 {
        let op = self.ram.borrow().get_byte(self.pc);
        let (_, code_segment_end) = self.code_segment;
        if let Some(opcode) = self.opcodes[op as usize] {
            if self.pc + 1 <= code_segment_end {
                self.operand_1 = self.ram.borrow().get_byte(self.pc + 1);
            }
            if self.pc + 2 <= code_segment_end {
                self.operand_2 = self.ram.borrow().get_byte(self.pc + 2);
            }
            let (address, extra_cycles) =
                self.get_address_and_extra_cycles(self.operand_1, self.operand_2, opcode.mode);
            self.address = address;
            self.cycles_next = (opcode.base_cycles + extra_cycles) as u16; 
            self.opcode_next = op;
            self.cycles_next
        } else {
            panic!("Unknown instruction {:#04x} at {:#X} ", op, self.pc);
        }
    }

    pub fn run_next_instruction(&mut self)  {
        //println!("{:X} {:X} {:X} {:X} \t\tA:{:X} X:{:X} Y:{:X} P:{:X} SP={:X}",self.pc, self.opcode_next, self.operand_1, self.operand_2,self.a, self.x, self.y, self.ps, self.sp);
        let opcode = self.opcodes[self.opcode_next as usize].unwrap();
        (opcode.instruction)(self);
        self.pc += opcode.mode.get_bytes() as u16;
        let cycles_left = std::u128::MAX - self.cycles;
        if cycles_left < self.cycles_next as u128 {
            self.cycles = self.cycles_next as u128 - cycles_left;
        } else {
            self.cycles += self.cycles_next as u128;
        }
    }

    fn get_address_and_extra_cycles(&self, b0: u8, b1: u8, mode: AddressingMode) -> (Address, u8) {
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

    fn load_from_address(&self) -> u8 {
        match &self.address {
            Address::Implicit => panic!("load_from_address can't be used for implicit mode"),
            Address::Accumulator => self.a,
            Address::Immediate(i) => *i,
            Address::RAM(address) => self.ram.borrow().get_byte(*address),
            Address::Relative(_) => panic!("load_from_address can't be used for the Relative mode"),
        }
    }

    fn get_ram_address(&self) -> u16 {
        match &self.address {
            Address::RAM(address) => *address,
            _ => panic!("Invalid address type {:?}", self.address),
        }
    }

    fn store_to_address(&mut self, byte: u8) {
        match &self.address {
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

    fn adc(&mut self) {
        let m = self.load_from_address();
        self._adc(m);
    }

    fn sbc(&mut self) {
        let m = self.load_from_address();
        self._adc(!m);
    }

    fn asl(&mut self) {
        let mut m = self.load_from_address();
        let old_bit_7 = m & 0x80;
        m = m << 1;
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
    }

    fn and(&mut self) {
        let m = self.load_from_address();
        self.a &= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ora(&mut self) {
        let m = self.load_from_address();
        self.a |= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn eor(&mut self) {
        let m = self.load_from_address();
        self.a ^= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ror(&mut self) {
        let mut m = self.load_from_address();
        let old_bit_0 = m & 0x1;
        m = m >> 1 | (self.carry() << 7);
        self.store_to_address(m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
    }

    fn rol(&mut self) {
        let mut m = self.load_from_address();
        let old_bit_7 = m & 0x80;
        m = m << 1 | self.carry();
        self.store_to_address(m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_7 != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
    }

    fn lsr(&mut self) {
        let mut m = self.load_from_address();
        let old_bit_0 = m & 1;
        m >>= 1;
        self.store_to_address(m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, old_bit_0 == 1);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
    }

    fn nop(&mut self) {}

    fn jsr(&mut self) {
        self.push_u16(self.pc + 2);
        self.pc = self.get_ram_address() - 3;
    }

    fn rts(&mut self) {
        self.pc = self.pop_u16();
    }

    fn rti(&mut self) {
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_u8();
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
        self.pc = self.pop_u16() - 1;
    }

    fn jmp(&mut self) {
        self.pc = self.get_ram_address() - 3;
    }

    fn sei(&mut self) {
        self.set_flag(ProcessorFlag::InterruptDisable);
    }

    fn cld(&mut self) {
        self.reset_flag(ProcessorFlag::DecimalMode);
    }

    fn cli(&mut self) {
        self.reset_flag(ProcessorFlag::InterruptDisable);
    }

    fn clv(&mut self) {
        self.reset_flag(ProcessorFlag::OverflowFlag);
    }

    fn lda(&mut self) {
        self.a = self.load_from_address();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn ldx(&mut self) {
        self.x = self.load_from_address();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn ldy(&mut self) {
        self.y = self.load_from_address();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn sta(&mut self) {
        self.store_to_address(self.a);
    }

    fn sty(&mut self) {
        self.store_to_address(self.y);
    }

    fn stx(&mut self) {
        self.store_to_address(self.x);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
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

    fn beq(&mut self) {
        self.branch_if(self.check_condition_for_beq());
    }

    fn check_condition_for_bne(&self) -> bool {
        !self.check_condition_for_beq()
    }

    fn bne(&mut self) {
        self.branch_if(self.check_condition_for_bne());
    }

    fn check_condition_for_bmi(&self) -> bool {
        self.get_flag(ProcessorFlag::NegativeFlag)
    }

    fn bmi(&mut self) {
        self.branch_if(self.check_condition_for_bmi());
    }

    fn check_condition_for_bpl(&self) -> bool {
        !self.check_condition_for_bmi()
    }

    fn bpl(&mut self) {
        self.branch_if(self.check_condition_for_bpl());
    }

    fn check_condition_for_bvs(&self) -> bool {
        self.get_flag(ProcessorFlag::OverflowFlag)
    }

    fn bvs(&mut self) {
        self.branch_if(self.check_condition_for_bvs());
    }

    fn check_condition_for_bvc(&self) -> bool {
        !self.check_condition_for_bvs()
    }

    fn bvc(&mut self) {
        self.branch_if(self.check_condition_for_bvc());
    }

    fn check_condition_for_bcs(&self) -> bool {
        self.get_flag(ProcessorFlag::CarryFlag)
    }

    fn bcs(&mut self) {
        self.branch_if(self.check_condition_for_bcs());
    }

    fn check_condition_for_bcc(&self) -> bool {
        !self.check_condition_for_bcs()
    }

    fn bcc(&mut self) {
        self.branch_if(self.check_condition_for_bcc());
    }

    fn bit(&mut self) {
        let m = self.load_from_address();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a & m == 0);
        self.set_or_reset_flag(ProcessorFlag::OverflowFlag, m & (1 << 6) != 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & (1 << 7) != 0);
    }

    fn cmp(&mut self) {
        let m = self.load_from_address();
        let result = (self.a as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.a >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn cpx(&mut self) {
        let m = self.load_from_address();
        let result = (self.x as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.x >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn cpy(&mut self) {
        let m = self.load_from_address();
        let result = (self.y as u16 - m as u16) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == m);
        self.set_or_reset_flag(ProcessorFlag::CarryFlag, self.y >= m);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, result & 0x80 != 0);
    }

    fn dey(&mut self) {
        if self.y == 0 {
            self.y = 0xFF;
        } else {
            self.y = self.y - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn dec(&mut self) {
        let mut m = self.load_from_address();
        if m == 0 {
            m = 0xFF;
        } else {
            m = m - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(m);
    }

    fn dex(&mut self) {
        if self.x == 0 {
            self.x = 0xFF;
        } else {
            self.x = self.x - 1;
        }
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn brk(&mut self) {
        self.push_u16(self.pc + 1);
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_u8(ps);
        self.pc = self.ram.borrow().get_word(0xFFFE) - 1;
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

    fn pha(&mut self) {
        self.push_u8(self.a);
    }

    fn php(&mut self) {
        let mut ps = self.ps;
        ps |= ProcessorFlag::BFlagBit4 as u8;
        ps |= ProcessorFlag::BFlagBit5 as u8;
        self.push_u8(ps);
    }

    fn pla(&mut self) {
        self.a = self.pop_u8();
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.a & 0x80 != 0);
    }

    fn plp(&mut self) {
        let b_flag_mask = ProcessorFlag::BFlagBit4 as u8 | ProcessorFlag::BFlagBit5 as u8;
        let b_flag_bits = self.ps & b_flag_mask;
        self.ps = self.pop_u8();
        self.ps &= !b_flag_mask;
        self.ps |= b_flag_bits;
    }

    fn inc(&mut self) {
        let mut m = self.load_from_address();
        let result = m as u16 + 1;
        m = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
        self.store_to_address(m);
    }

    fn inx(&mut self) {
        let result = self.x as u16 + 1;
        self.x = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.x == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.x & 0x80 != 0);
    }

    fn iny(&mut self) {
        let result = self.y as u16 + 1;
        self.y = (result & 0xFF) as u8;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.y == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, self.y & 0x80 != 0);
    }

    fn clc(&mut self) {
        self.reset_flag(ProcessorFlag::CarryFlag);
    }

    fn sec(&mut self) {
        self.set_flag(ProcessorFlag::CarryFlag);
    }

    fn sed(&mut self) {
        self.set_flag(ProcessorFlag::DecimalMode);
    }

    fn lax(&mut self) {
        let m = self.load_from_address();
        self.a = m;
        self.x = m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, m & 0x80 != 0);
    }

    fn aax(&mut self) {
        let result = self.a & self.x;
        self.store_to_address(result);
    }

    fn dcp(&mut self) {
        self.dec();
        self.cmp();
    }

    fn isc(&mut self) {
        self.inc();
        self.sbc();
    }

    fn slo(&mut self) {
        self.asl();
        self.ora();
    }

    fn rla(&mut self) {
        self.rol();
        self.and();
    }

    fn sre(&mut self) {
        self.lsr();
        self.eor();
    }

    fn rra(&mut self) {
        self.ror();
        self.adc();
    }
}
