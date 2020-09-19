use super::AddressingMode;
use super::AddressingMode::*;
use super::CPU;

pub type Instruction = fn(&mut CPU);
#[derive(Copy, Clone)]
pub(super) struct OpCode {
    pub(super) instruction: Instruction,
    pub(super) mode: AddressingMode,
    pub(super) base_cycles: u8,
    pub(super) extra_cycle_on_page_crossing: bool,
}

pub(super) type OpCodes = [Option<OpCode>; 256];

macro_rules! fill_opcodes {
    ($(($op:expr,$ins:ident,$mode:expr,$cycles:expr $(,$optional:expr)?)),*) => {{
        let mut opcodes: OpCodes = [None; 256];
        $(
        let mut extra_cycle_on_page_crossing = false;
        $(extra_cycle_on_page_crossing = $optional;
        )?

        opcodes[$op] = Some(OpCode{instruction:CPU::$ins, mode: $mode,base_cycles: $cycles,extra_cycle_on_page_crossing});
        )*
        opcodes
    }};
}
#[allow(unused_assignments, unused_mut)]
pub(super) fn get_opcodes() -> OpCodes {
    fill_opcodes!(
        /*BRK*/
        (0x00, brk, Implicit, 7),
        /*ADC*/
        (0x69, adc, Immediate, 2),
        (0x65, adc, ZeroPage, 3),
        (0x75, adc, ZeroPageX, 4),
        (0x6D, adc, Absolute, 4),
        (0x7D, adc, AbsoluteX, 4, true),
        (0x79, adc, AbsoluteY, 4, true),
        (0x61, adc, IndexedIndirectX, 6),
        (0x71, adc, IndirectIndexedY, 5, true),
        /*AND*/
        (0x29, and, Immediate, 2),
        (0x25, and, ZeroPage, 3),
        (0x35, and, ZeroPageX, 4),
        (0x2D, and, Absolute, 4),
        (0x3D, and, AbsoluteX, 4, true),
        (0x39, and, AbsoluteY, 4, true),
        (0x21, and, IndexedIndirectX, 6),
        (0x31, and, IndirectIndexedY, 5, true),
        /*ASL*/
        (0x0A, asl, Accumulator, 2),
        (0x06, asl, ZeroPage, 5),
        (0x16, asl, ZeroPageX, 6),
        (0x0E, asl, Absolute, 6),
        (0x1E, asl, AbsoluteX, 7),
        /*BRANCH*/
        (0x90, bcc, Relative, 2),
        (0xB0, bcs, Relative, 2),
        (0xF0, beq, Relative, 2),
        (0x30, bmi, Relative, 2),
        (0xD0, bne, Relative, 2),
        (0x10, bpl, Relative, 2),
        (0x50, bvc, Relative, 2),
        (0x70, bvs, Relative, 2),
        /*BIT*/
        (0x24, bit, ZeroPage, 3),
        (0x2C, bit, Absolute, 4),
        /*CLEAR FLAGS*/
        (0x18, clc, Implicit, 2),
        (0xD8, cld, Implicit, 2),
        (0x58, cli, Implicit, 2),
        (0xB8, clv, Implicit, 2),
        /*CMP*/
        (0xC9, cmp, Immediate, 2),
        (0xC5, cmp, ZeroPage, 3),
        (0xD5, cmp, ZeroPageX, 4),
        (0xCD, cmp, Absolute, 4),
        (0xDD, cmp, AbsoluteX, 4, true),
        (0xD9, cmp, AbsoluteY, 4, true),
        (0xC1, cmp, IndexedIndirectX, 6),
        (0xD1, cmp, IndirectIndexedY, 5, true),
        /*CPX*/
        (0xE0, cpx, Immediate, 2),
        (0xE4, cpx, ZeroPage, 3),
        (0xEC, cpx, Absolute, 4),
        /*CPY*/
        (0xC0, cpy, Immediate, 2),
        (0xC4, cpy, ZeroPage, 3),
        (0xCC, cpy, Absolute, 4),
        /*DEC*/
        (0xC6, dec, ZeroPage, 5),
        (0xD6, dec, ZeroPageX, 6),
        (0xCE, dec, Absolute, 6),
        (0xDE, dec, AbsoluteX, 7),
        (0xCA, dex, Implicit, 2),
        (0x88, dey, Implicit, 2),
        /*EOR*/
        (0x49, eor, Immediate, 2),
        (0x45, eor, ZeroPage, 3),
        (0x55, eor, ZeroPageX, 4),
        (0x4D, eor, Absolute, 4),
        (0x5D, eor, AbsoluteX, 4, true),
        (0x59, eor, AbsoluteY, 4, true),
        (0x41, eor, IndexedIndirectX, 6),
        (0x51, eor, IndirectIndexedY, 5, true),
        /*INC*/
        (0xE6, inc, ZeroPage, 5),
        (0xF6, inc, ZeroPageX, 6),
        (0xEE, inc, Absolute, 6),
        (0xFE, inc, AbsoluteX, 7),
        (0xE8, inx, Implicit, 2),
        (0xC8, iny, Implicit, 2),
        /*JMP*/
        (0x4C, jmp, Absolute, 3),
        (0x6C, jmp, Indirect, 5),
        /*JSR*/
        (0x20, jsr, Absolute, 6),
        /*LDA*/
        (0xA9, lda, Immediate, 2),
        (0xA5, lda, ZeroPage, 3),
        (0xB5, lda, ZeroPageX, 4),
        (0xAD, lda, Absolute, 4),
        (0xBD, lda, AbsoluteX, 4, true),
        (0xB9, lda, AbsoluteY, 4, true),
        (0xA1, lda, IndexedIndirectX, 6),
        (0xB1, lda, IndirectIndexedY, 5, true),
        /*LDX*/
        (0xA2, ldx, Immediate, 2),
        (0xA6, ldx, ZeroPage, 3),
        (0xB6, ldx, ZeroPageY, 4),
        (0xAE, ldx, Absolute, 4),
        (0xBE, ldx, AbsoluteY, 4, true),
        /*LDY*/
        (0xA0, ldy, Immediate, 2),
        (0xA4, ldy, ZeroPage, 3),
        (0xB4, ldy, ZeroPageX, 4),
        (0xAC, ldy, Absolute, 4),
        (0xBC, ldy, AbsoluteX, 4, true),
        /*LSR*/
        (0x4A, lsr, Accumulator, 2),
        (0x46, lsr, ZeroPage, 5),
        (0x56, lsr, ZeroPageX, 6),
        (0x4E, lsr, Absolute, 6),
        (0x5E, lsr, AbsoluteX, 7),
        /*NOP*/
        (0xEA, nop, Implicit, 2),
        /*ORA*/
        (0x09, ora, Immediate, 2),
        (0x05, ora, ZeroPage, 3),
        (0x15, ora, ZeroPageX, 4),
        (0x0D, ora, Absolute, 4),
        (0x1D, ora, AbsoluteX, 4, true),
        (0x19, ora, AbsoluteY, 4, true),
        (0x01, ora, IndexedIndirectX, 6),
        (0x11, ora, IndirectIndexedY, 5, true),
        /*PUSH-PULL*/
        (0x48, pha, Implicit, 3),
        (0x08, php, Implicit, 3),
        (0x68, pla, Implicit, 4),
        (0x28, plp, Implicit, 4),
        /*ROL*/
        (0x2A, rol, Accumulator, 2),
        (0x26, rol, ZeroPage, 5),
        (0x36, rol, ZeroPageX, 6),
        (0x2E, rol, Absolute, 6),
        (0x3E, rol, AbsoluteX, 7),
        /*ROR*/
        (0x6A, ror, Accumulator, 2),
        (0x66, ror, ZeroPage, 5),
        (0x76, ror, ZeroPageX, 6),
        (0x6E, ror, Absolute, 6),
        (0x7E, ror, AbsoluteX, 7),
        /*RTI*/
        (0x40, rti, Implicit, 6),
        /*RTS*/
        (0x60, rts, Implicit, 6),
        /*SBC*/
        (0xE9, sbc, Immediate, 2),
        (0xE5, sbc, ZeroPage, 3),
        (0xF5, sbc, ZeroPageX, 4),
        (0xED, sbc, Absolute, 4),
        (0xFD, sbc, AbsoluteX, 4, true),
        (0xF9, sbc, AbsoluteY, 4, true),
        (0xE1, sbc, IndexedIndirectX, 6),
        (0xF1, sbc, IndirectIndexedY, 5, true),
        /*SET CLEARS*/
        (0x38, sec, Implicit, 2),
        (0xF8, sed, Implicit, 2),
        (0x78, sei, Implicit, 2),
        /*STA*/
        (0x85, sta, ZeroPage, 3),
        (0x95, sta, ZeroPageX, 4),
        (0x8D, sta, Absolute, 4),
        (0x9D, sta, AbsoluteX, 5),
        (0x99, sta, AbsoluteY, 5),
        (0x81, sta, IndexedIndirectX, 6),
        (0x91, sta, IndirectIndexedY, 6),
        /*STY*/
        (0x84, sty, ZeroPage, 3),
        (0x94, sty, ZeroPageX, 4),
        (0x8C, sty, Absolute, 4),
        /*STX*/
        (0x86, stx, ZeroPage, 3),
        (0x96, stx, ZeroPageY, 4),
        (0x8E, stx, Absolute, 4),
        /*TRANSFER*/
        (0xAA, tax, Implicit, 2),
        (0xA8, tay, Implicit, 2),
        (0xBA, tsx, Implicit, 2),
        (0x8A, txa, Implicit, 2),
        (0x9A, txs, Implicit, 2),
        (0x98, tya, Implicit, 2),
        /*ILLEGAL OPPCODES */
        (0x87, aax, ZeroPage, 3),
        (0x97, aax, ZeroPageY, 4),
        (0x8F, aax, Absolute, 4),
        (0x83, aax, IndexedIndirectX, 6),
        (0x4B, alr, Immediate, 2),
        (0x2B, anc, Immediate, 2),
        (0x0B, anc, Immediate, 2),
        (0x6B, arr, Immediate, 2),
        (0x9F, axa, AbsoluteY, 5),
        (0x93, axa, ZeroPageY, 6),
        (0xC7, dcp, ZeroPage, 5),
        (0xD7, dcp, ZeroPageX, 6),
        (0xCF, dcp, Absolute, 6),
        (0xDF, dcp, AbsoluteX, 7),
        (0xDB, dcp, AbsoluteY, 7),
        (0xC3, dcp, IndexedIndirectX, 8),
        (0xD3, dcp, IndirectIndexedY, 8),
        (0xE7, isc, ZeroPage, 5),
        (0xF7, isc, ZeroPageX, 6),
        (0xEF, isc, Absolute, 6),
        (0xFF, isc, AbsoluteX, 7),
        (0xFB, isc, AbsoluteY, 7),
        (0xE3, isc, IndexedIndirectX, 8),
        (0xF3, isc, IndirectIndexedY, 8),
        (0xA7, lax, ZeroPage, 3),
        (0xB7, lax, ZeroPageY, 4),
        (0xAF, lax, Absolute, 4),
        (0xBF, lax, AbsoluteY, 4, true),
        (0xA3, lax, IndexedIndirectX, 6),
        (0xB3, lax, IndirectIndexedY, 5, true),
        (0xBB, las, AbsoluteY, 4, true),
        /*NOP*/
        (0x1A, nop, Implicit, 2),
        (0x3A, nop, Implicit, 2),
        (0x5A, nop, Implicit, 2),
        (0x7A, nop, Implicit, 2),
        (0xDA, nop, Implicit, 2),
        (0xFA, nop, Implicit, 2),
        /*DOP*/
        (0x04, nop, ZeroPage, 3),
        (0x14, nop, ZeroPageX, 4),
        (0x34, nop, ZeroPageX, 4),
        (0x44, nop, ZeroPage, 3),
        (0x54, nop, ZeroPageX, 4),
        (0x64, nop, ZeroPage, 3),
        (0x74, nop, ZeroPageX, 4),
        (0x80, nop, Immediate, 2),
        (0x82, nop, Immediate, 2),
        (0x89, nop, Immediate, 2),
        (0xC2, nop, Immediate, 2),
        (0xD4, nop, ZeroPageX, 4),
        (0xE2, nop, Immediate, 2),
        (0xF4, nop, ZeroPageX, 4),
        /*TOP*/
        (0x0C, nop, Absolute, 4, true),
        (0x1C, nop, AbsoluteX, 4, true),
        (0x3C, nop, AbsoluteX, 4, true),
        (0x5C, nop, AbsoluteX, 4, true),
        (0x7C, nop, AbsoluteX, 4, true),
        (0xDC, nop, AbsoluteX, 4, true),
        (0xFC, nop, AbsoluteX, 4, true),
        (0xAB, oal, Immediate, 2),
        (0x27, rla, ZeroPage, 5),
        (0x37, rla, ZeroPageX, 6),
        (0x2F, rla, Absolute, 6),
        (0x3F, rla, AbsoluteX, 7),
        (0x3B, rla, AbsoluteY, 7),
        (0x23, rla, IndexedIndirectX, 8),
        (0x33, rla, IndirectIndexedY, 8),
        (0x67, rra, ZeroPage, 5),
        (0x77, rra, ZeroPageX, 6),
        (0x6F, rra, Absolute, 6),
        (0x7F, rra, AbsoluteX, 7),
        (0x7B, rra, AbsoluteY, 7),
        (0x63, rra, IndexedIndirectX, 8),
        (0x73, rra, IndirectIndexedY, 8),
        (0xCB, sax, Immediate, 2),
        (0x9C, say, AbsoluteX, 5),
        (0xEB, sbc, Immediate, 2),
        (0x07, slo, ZeroPage, 5),
        (0x17, slo, ZeroPageX, 6),
        (0x0F, slo, Absolute, 6),
        (0x1F, slo, AbsoluteX, 7),
        (0x1B, slo, AbsoluteY, 7),
        (0x03, slo, IndexedIndirectX, 8),
        (0x13, slo, IndirectIndexedY, 8),
        (0x47, sre, ZeroPage, 5),
        (0x57, sre, ZeroPageX, 6),
        (0x4F, sre, Absolute, 6),
        (0x5F, sre, AbsoluteX, 7),
        (0x5B, sre, AbsoluteY, 7),
        (0x43, sre, IndexedIndirectX, 8),
        (0x53, sre, IndirectIndexedY, 8),
        (0x9B, tas, AbsoluteY, 5),
        (0x8B, xaa, Immediate, 2),
        (0x9E, xas, AbsoluteY, 5)
    )
}
