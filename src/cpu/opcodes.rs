use super::AddressingMode;
use super::AddressingMode::*;
use super::CPU;

type Instruction = fn(&mut CPU);
#[derive(Copy, Clone)]
pub struct OpCode(pub Instruction, pub AddressingMode, pub u8, pub u8);

pub type OpCodes = [Option<OpCode>; 256];

macro_rules! fill_opcodes {
    ($(($op:expr,$ins:ident,$mode:expr,$bytes:expr,$cycles:expr)),*) => {{
        let mut opcodes: OpCodes = [None; 256];
        $(
        opcodes[$op] = Some(OpCode(CPU::$ins, $mode, $bytes, $cycles));
        )*
        opcodes
    }};
}


pub fn get_opcodes() -> OpCodes {
    fill_opcodes!(
        /*BRK*/
        (0x00, brk, Implicit, 1, 7),
         /*ADC*/
        (0x69, adc, Immediate, 2, 2),
        (0x65, adc, ZeroPage, 2, 3),
        (0x75, adc, ZeroPageX, 2, 4),
        (0x6D, adc, Absolute, 3, 4),
        (0x7D, adc, AbsoluteX, 3, 4),
        (0x79, adc, AbsoluteY, 3, 5),
        (0x61, adc, IndexedIndirectX, 2, 6),
        (0x71, adc, IndirectIndexedY, 2, 5),
        /*AND*/
        (0x29, and, Immediate, 2, 2),
        (0x25, and, ZeroPage, 2, 3),
        (0x35, and, ZeroPageX, 2, 4),
        (0x2D, and, Absolute, 3, 4),
        (0x3D, and, AbsoluteX, 3, 4),
        (0x39, and, AbsoluteY, 3, 4),
        (0x21, and, IndexedIndirectX, 2, 6),
        (0x31, and, IndirectIndexedY, 2, 5),
         /*ASL*/
        (0x0A, asl, Accumulator, 1, 2),
        (0x06, asl, ZeroPage, 2, 5),
        (0x16, asl, ZeroPageX, 2, 6),
        (0x0E, asl, Absolute, 3, 6),
        (0x1E, asl, AbsoluteX, 3, 7),
        /*BRANCH*/
        (0x90, bcc, Relative, 2, 2),
        (0xB0, bcs, Relative, 2, 2),
        (0xF0, beq, Relative, 2, 2),
        (0x30, bmi, Relative, 2, 2),
        (0xD0, bne, Relative, 2, 2),
        (0x10, bpl, Relative, 2, 2),
        (0x50, bvc, Relative, 2, 2),
        (0x70, bvs, Relative, 2, 2),
        /*BIT*/
        (0x24, bit, ZeroPage, 2, 3),
        (0x2C, bit, Absolute, 3, 5),
        /*CLEAR FLAGS*/
        (0x18, clc, Implicit, 1, 2),
        (0xD8, cld, Implicit, 1, 2),
        (0x58, cli, Implicit, 1, 2),
        (0xB8, clv, Implicit, 1, 2),
        /*CMP*/
        (0xC9, cmp, Immediate, 2, 2),
        (0xC5, cmp, ZeroPage, 2, 3),
        (0xD5, cmp, ZeroPageX, 2, 4),
        (0xCD, cmp, Absolute, 3, 4),
        (0xDD, cmp, AbsoluteX, 3, 4),
        (0xD9, cmp, AbsoluteY, 3, 4),
        (0xC1, cmp, IndexedIndirectX, 2, 6),
        (0xD1, cmp, IndirectIndexedY, 2, 5),
        /*CPX*/
        (0xE0, cpx, Immediate, 2, 2),
        (0xE4, cpx, ZeroPage, 2, 3),
        (0xEC, cpx, Absolute, 3, 4),
        /*CPY*/
        (0xC0, cpy, Immediate, 2, 2),
        (0xC4, cpy, ZeroPage, 2, 3),
        (0xCC, cpy, Absolute, 3, 4),
        /*DEC*/
        (0xC6, dec, ZeroPage, 2, 5),
        (0xD6, dec, ZeroPageX, 2, 6),
        (0xCE, dec, Absolute, 3, 6),
        (0xDE, dec, AbsoluteX, 3, 7),
        (0xCA, dex, Implicit, 1, 2),
        (0x88, dey, Implicit, 1, 2),
         /*EOR*/
        (0x49, eor, Immediate, 2, 2),
        (0x45, eor, ZeroPage, 2, 3),
        (0x55, eor, ZeroPageX, 2, 4),
        (0x4D, eor, Absolute, 3, 4),
        (0x5D, eor, AbsoluteX, 3, 4),
        (0x59, eor, AbsoluteY, 3, 4),
        (0x41, eor, IndexedIndirectX, 2, 6),
        (0x51, eor, IndirectIndexedY, 2, 5),
        /*INC*/
        (0xE6, inc, ZeroPage, 2, 5),
        (0xF6, inc, ZeroPageX, 2, 6),
        (0xEE, inc, Absolute, 3, 6),
        (0xFE, inc, AbsoluteX, 3, 7),
        (0xE8, inx, Implicit, 1, 2),
        (0xC8, iny, Implicit, 1, 2),
        /*JMP*/
        (0x4C, jmp, Absolute, 3, 3),
        (0x6C, jmp, Indirect, 3, 5),
        /*JSR*/
        (0x20, jsr, Absolute, 3, 6),
        /*LDA*/
        (0xA9, lda, Immediate, 2, 2),
        (0xA5, lda, ZeroPage, 2, 3),
        (0xB5, lda, ZeroPageX, 2, 4),
        (0xAD, lda, Absolute, 3, 4),
        (0xBD, lda, AbsoluteX, 3, 4),
        (0xB9, lda, AbsoluteY, 3, 4),
        (0xA1, lda, IndexedIndirectX, 2, 6),
        (0xB1, lda, IndirectIndexedY, 2, 5),
        /*LDX*/
        (0xA2, ldx, Immediate, 2, 2),
        (0xA6, ldx, ZeroPage, 2, 2),
        (0xB6, ldx, ZeroPageY, 2, 2),
        (0xAE, ldx, Absolute, 3, 3),
        (0xBE, ldx, AbsoluteY, 3, 3),
        /*LDY*/
        (0xA0, ldy, Immediate, 2, 2),
        (0xA4, ldy, ZeroPage, 2, 2),
        (0xB4, ldy, ZeroPageX, 2, 2),
        (0xAC, ldy, Absolute, 3, 3),
        (0xBC, ldy, AbsoluteX, 3, 3),
        /*LSR*/
        (0x4A, lsr, Accumulator, 1, 2),
        (0x46, lsr, ZeroPage, 2, 5),
        (0x56, lsr, ZeroPageX, 2, 6),
        (0x4E, lsr, Absolute, 3, 6),
        (0x5E, lsr, AbsoluteX, 3, 7),
        /*NOP*/
        (0xEA, nop, Implicit, 1, 2),
        /*ORA*/
        (0x09, ora, Immediate, 2, 2),
        (0x05, ora, ZeroPage, 2, 3),
        (0x15, ora, ZeroPageX, 2, 4),
        (0x0D, ora, Absolute, 3, 4),
        (0x1D, ora, AbsoluteX, 3, 4),
        (0x19, ora, AbsoluteY, 3, 4),
        (0x01, ora, IndexedIndirectX, 2, 6),
        (0x11, ora, IndirectIndexedY, 2, 5),
        /*PUSH-PULL*/
        (0x48, pha, Implicit, 1, 3),
        (0x08, php, Implicit, 1, 3),
        (0x68, pla, Implicit, 1, 3),
        (0x28, plp, Implicit, 1, 3),
        /*ROL*/
        (0x2A, rol, Accumulator, 1, 2),
        (0x26, rol, ZeroPage, 2, 5),
        (0x36, rol, ZeroPageX, 2, 6),
        (0x2E, rol, Absolute, 3, 6),
        (0x3E, rol, AbsoluteX, 3, 7),
        /*ROR*/
        (0x6A, ror, Accumulator, 1, 2),
        (0x66, ror, ZeroPage, 2, 5),
        (0x76, ror, ZeroPageX, 2, 6),
        (0x6E, ror, Absolute, 3, 6),
        (0x7E, ror, AbsoluteX, 3, 7),
        /*RTI*/
        (0x40, rti, Implicit, 1, 6),
        /*RTS*/
        (0x60, rts, Implicit, 1, 6),
        /*SBC*/
        (0xE9, sbc, Immediate, 2, 2),
        (0xE5, sbc, ZeroPage, 2, 3),
        (0xF5, sbc, ZeroPageX, 2, 4),
        (0xED, sbc, Absolute, 3, 4),
        (0xFD, sbc, AbsoluteX, 3, 4),
        (0xF9, sbc, AbsoluteY, 3, 4),
        (0xE1, sbc, IndexedIndirectX, 2, 6),
        (0xF1, sbc, IndirectIndexedY, 2, 5),
        /*SET CLEARS*/
        (0x38, sec, Implicit, 1, 2),
        (0xF8, sed, Implicit, 1, 2),
        (0x78, sei, Implicit, 1, 2),
        /*STA*/
        (0x85, sta, ZeroPage, 2, 3),
        (0x95, sta, ZeroPageX, 2, 4),
        (0x8D, sta, Absolute, 3, 4),
        (0x9D, sta, AbsoluteX, 3, 5),
        (0x99, sta, AbsoluteY, 3, 5),
        (0x81, sta, IndexedIndirectX, 2, 6),
        (0x91, sta, IndirectIndexedY, 2, 6),
        /*STY*/
        (0x84, sty, ZeroPage, 2, 3),
        (0x94, sty, ZeroPageX, 2, 4),
        (0x8C, sty, Absolute, 3, 4),
        /*STX*/
        (0x86, stx, ZeroPage, 2, 3),
        (0x96, stx, ZeroPageY, 2, 4),
        (0x8E, stx, Absolute, 3, 4),
        /*TRANSFER*/
        (0xAA, tax, Implicit, 1, 2),
        (0xA8, tay, Implicit, 1, 2),
        (0xBA, tsx, Implicit, 1, 2),
        (0x8A, txa, Implicit, 1, 2),
        (0x9A, txs, Implicit, 1, 2),
        (0x98, tya, Implicit, 1, 2),
        /*ILLEGAL OPPCODES */
        (0x87, aax, ZeroPage, 2, 3),
        (0x97, aax, ZeroPageY, 2, 4),
        (0x8F, aax, Absolute, 3, 4),
        (0x83, aax, IndexedIndirectX, 2, 6),
        (0xC7, dcp, ZeroPage, 2, 5),
        (0xD7, dcp, ZeroPageX, 2, 6),
        (0xCF, dcp, Absolute, 3, 6),
        (0xDF, dcp, AbsoluteX, 3, 7),
        (0xDB, dcp, AbsoluteY, 3, 7),
        (0xC3, dcp, IndexedIndirectX, 2, 8),
        (0xD3, dcp, IndirectIndexedY, 2, 8),
        (0xE7, isc, ZeroPage, 2, 5),
        (0xF7, isc, ZeroPageX, 2, 6),
        (0xEF, isc, Absolute, 3, 6),
        (0xFF, isc, AbsoluteX, 3, 7),
        (0xFB, isc, AbsoluteY, 3, 7),
        (0xE3, isc, IndexedIndirectX, 2, 8),
        (0xF3, isc, IndirectIndexedY, 2, 8),
        (0xA7, lax, ZeroPage, 2, 3),
        (0xB7, lax, ZeroPageY, 2, 4),
        (0xAF, lax, Absolute, 3, 4),
        (0xBF, lax, AbsoluteY, 3, 4),
        (0xA3, lax, IndexedIndirectX, 2, 6),
        (0xB3, lax, IndirectIndexedY, 2, 5),
        /*NOP*/
        (0x1A, nop, Implicit, 1, 2),
        (0x3A, nop, Implicit, 1, 2),
        (0x5A, nop, Implicit, 1, 2),
        (0x7A, nop, Implicit, 1, 2),
        (0xDA, nop, Implicit, 1, 2),
        (0xFA, nop, Implicit, 1, 2),
        /*DOP*/
        (0x04, nop, ZeroPage, 2, 3),
        (0x14, nop, ZeroPageX, 2, 4),
        (0x34, nop, ZeroPageX, 2, 4),
        (0x44, nop, ZeroPage, 2, 3),
        (0x54, nop, ZeroPageX, 2, 4),
        (0x64, nop, ZeroPage, 2, 3),
        (0x74, nop, ZeroPageX, 2, 4),
        (0x80, nop, Implicit, 2, 2),
        (0x82, nop, Implicit, 2, 2),
        (0x89, nop, Implicit, 2, 2),
        (0xC2, nop, Implicit, 2, 2),
        (0xD4, nop, ZeroPageX, 2, 4),
        (0xE2, nop, Implicit, 2, 2),
        (0xF4, nop, ZeroPageX, 2, 4),
        /*TOP*/
        (0x0C, nop, Absolute, 3, 4),
        (0x1C, nop, AbsoluteX, 3, 4),
        (0x3C, nop, AbsoluteX, 3, 4),
        (0x5C, nop, AbsoluteX, 3, 4),
        (0x7C, nop, AbsoluteX, 3, 4),
        (0xDC, nop, AbsoluteX, 3, 4),
        (0xFC, nop, AbsoluteX, 3, 4),
        (0x27, rla, ZeroPage, 2, 5),
        (0x37, rla, ZeroPageX, 2, 6),
        (0x2F, rla, Absolute, 3, 6),
        (0x3F, rla, AbsoluteX, 3, 7),
        (0x3B, rla, AbsoluteY, 3, 7),
        (0x23, rla, IndexedIndirectX, 2, 8),
        (0x33, rla, IndirectIndexedY, 2, 8),
        (0x67, rra, ZeroPage, 2, 5),
        (0x77, rra, ZeroPageX, 2, 6),
        (0x6F, rra, Absolute, 3, 6),
        (0x7F, rra, AbsoluteX, 3, 7),
        (0x7B, rra, AbsoluteY, 3, 7),
        (0x63, rra, IndexedIndirectX, 2, 8),
        (0x73, rra, IndirectIndexedY, 2, 8),
        (0xEB, sbc, Immediate, 2, 2),
        (0x07, slo, ZeroPage, 2, 5),
        (0x17, slo, ZeroPageX, 2, 6),
        (0x0F, slo, Absolute, 3, 6),
        (0x1F, slo, AbsoluteX, 3, 7),
        (0x1B, slo, AbsoluteY, 3, 7),
        (0x03, slo, IndexedIndirectX, 2, 8),
        (0x13, slo, IndirectIndexedY, 2, 8),
        (0x47, sre, ZeroPage, 2, 5),
        (0x57, sre, ZeroPageX, 2, 6),
        (0x4F, sre, Absolute, 3, 6),
        (0x5F, sre, AbsoluteX, 3, 7),
        (0x5B, sre, AbsoluteY, 3, 7),
        (0x43, sre, IndexedIndirectX, 2, 8),
        (0x53, sre, IndirectIndexedY, 2, 8)
    )
}
