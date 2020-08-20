use crate::memory::{Memory,RAM};
use crate::screen::{Screen, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::common::*;
use std::time::{Duration, Instant};
use std::thread::sleep;
use crate::keyboard::{KeyEvent};
use crate::mapper::{Mapper};
use std::sync::mpsc::{Sender, Receiver};
use crate::ppu::*;

const NTSC_FREQ_MHZ          : f32 = 1.79;
const NANOS_PER_CYCLE : u128 =  ((1.0/NTSC_FREQ_MHZ) * 1000.0) as u128;


type Keyboard = [bool; 16];

#[derive(Debug)]
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


enum ProcessorFlag {
    CarryFlag           = 0b00000001,
    ZeroFlag            = 0b00000010,
    InterruptDisable    = 0b00000100,
    DecimalMode         = 0b00001000,
    BreakCommand        = 0b00010000,
    OverflowFlag        = 0b00100000,
    NegativeFlag        = 0b01000000,
}

pub struct CPU
{
   pc           : u16,
   sp           : u8,
   ps           : u8,
   a            : u8,
   x            : u8,
   y            : u8,
   stack        : Vec<u16>,
   ram          : RAM,
   ppu          : PPU,
   keyboard     : Keyboard,
   keyboard_rx  : Receiver<KeyEvent>,
   audio_tx     : Sender<bool>,
   code_segment : (u16,u16)
}

impl<'a> CPU
{
    pub fn new(mapper : &'a mut Box<dyn Mapper>,
               screen_tx: Sender<Screen>, 
               keyboard_rx: Receiver<KeyEvent>,
               audio_tx: Sender<bool>) -> CPU
    {
        let mut ram = RAM::new();
        ram.store_bytes(mapper.get_rom_start(), &mapper.get_pgr_rom().to_vec());
        CPU
        {
            pc           : ram.get_2_bytes(0xFFFC),
            sp           : 0,
            ps           : 0,
            a            : 0,
            x            : 0,
            y            : 0,
            stack        : Vec::<u16>::new(),
            ram          : ram,
            ppu          : PPU::new(screen_tx, mapper.get_chr_rom().to_vec()),
            keyboard     : [false; 16],
            keyboard_rx  : keyboard_rx,
            audio_tx     : audio_tx,
            code_segment : (mapper.get_rom_start(), mapper.get_rom_start() - 1 + mapper.get_pgr_rom().len() as u16)
        }
    }

    fn set_flag(&mut self, flag : ProcessorFlag) {
        self.ps |= flag as u8;
    }

    fn reset_flag(&mut self, flag : ProcessorFlag) {
        self.ps &= !(flag as u8);
    }

    fn get_flag(&mut self, flag : ProcessorFlag) -> bool {
        (self.ps & (flag as u8)) != 0
    }

    fn set_or_reset_flag(&mut self, flag : ProcessorFlag, cond : bool) {
        if cond {self.set_flag(flag);}
        else    {self.reset_flag(flag);}
    }

    fn set_flag_if(&mut self, flag : ProcessorFlag, cond : bool) {
        if cond {self.set_flag(flag);}
    }

    fn carry(&mut self) -> u8 {
        if self.get_flag(ProcessorFlag::CarryFlag) { 1 }
        else                                       { 0 }
    }

 
    pub fn run(&mut self) {
        let (code_segment_start, code_segment_end) = self.code_segment;
  
        println!("PC int location {:#X} ROM Start {:#X}  ROM End {:#X}", self.pc, code_segment_start, code_segment_end);

        while self.pc >= code_segment_start &&  self.pc <= code_segment_end {
            let now = Instant::now();
            
            let op = self.ram.get_byte(self.pc);
            let mut b0 = 0;
            let mut b1 = 0;
            if  self.pc + 1 <= code_segment_end {
                b0 = self.ram.get_byte(self.pc + 1);
            }
            if  self.pc + 2 <= code_segment_end {
                b1 = self.ram.get_byte(self.pc + 2);
            }

            let (bytes, cycles) = self.execute_instruction(op, b0, b1);
            self.pc += bytes as u16;
            println!("Executed instruction {:#0x} bytes {} cycles {} pc {:X}",op, bytes, cycles, self.pc);
            let elapsed_time_ns  = now.elapsed().as_nanos();
            let required_time_ns = (cycles as u128) * NANOS_PER_CYCLE;
            if required_time_ns > elapsed_time_ns {
               sleep(Duration::from_nanos((required_time_ns - elapsed_time_ns) as u64));
            }
            self.ppu.render_frame();
        }
        println!("CPU Stopped execution")
    }

    fn execute_instruction(&mut self, op : u8, b0 : u8, b1 : u8 ) -> (u8, u8)  {

        match op {
            0x69 => (2, 2 + self.add_with_carry(b0, b1, AddressingMode::Immediate)),
            0x65 => (2, 3 + self.add_with_carry(b0, b1, AddressingMode::ZeroPage)),
            0x75 => (2, 4 + self.add_with_carry(b0, b1, AddressingMode::ZeroPageX)),
            0x6D => (3, 4 + self.add_with_carry(b0, b1, AddressingMode::Absolute)),
            0x7D => (3, 4 + self.add_with_carry(b0, b1, AddressingMode::AbsoluteX)),
            0x79 => (3, 4 + self.add_with_carry(b0, b1, AddressingMode::AbsoluteY)),
            0x61 => (2, 6 + self.add_with_carry(b0, b1, AddressingMode::IndexedIndirectX)),
            0x71 => (2, 5 + self.add_with_carry(b0, b1, AddressingMode::IndirectIndexedY)),

            0x29 => (2, 2 + self.and(b0, b1, AddressingMode::Immediate)),
            0x25 => (2, 3 + self.and(b0, b1, AddressingMode::ZeroPage)),
            0x35 => (2, 4 + self.and(b0, b1, AddressingMode::ZeroPageX)),
            0x2D => (3, 4 + self.and(b0, b1, AddressingMode::Absolute)),
            0x3D => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteX)),
            0x39 => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteY)),
            0x21 => (2, 6 + self.and(b0, b1, AddressingMode::IndexedIndirectX)),
            0x31 => (2, 5 + self.and(b0, b1, AddressingMode::IndirectIndexedY)),

            0x09 => (2, 2 + self.or(b0, b1, AddressingMode::Immediate)),
            0x05 => (2, 3 + self.or(b0, b1, AddressingMode::ZeroPage)),
            0x15 => (2, 4 + self.or(b0, b1, AddressingMode::ZeroPageX)),
            0x0D => (3, 4 + self.or(b0, b1, AddressingMode::Absolute)),
            0x1D => (3, 4 + self.or(b0, b1, AddressingMode::AbsoluteX)),
            0x19 => (3, 4 + self.or(b0, b1, AddressingMode::AbsoluteY)),
            0x01 => (2, 6 + self.or(b0, b1, AddressingMode::IndexedIndirectX)),
            0x11 => (2, 5 + self.or(b0, b1, AddressingMode::IndirectIndexedY)),


            0x4A => (1, 2 + self.lsr(b0, b1, AddressingMode::Accumulator)),
            0x46 => (2, 5 + self.lsr(b0, b1, AddressingMode::ZeroPage)),
            0x56 => (2, 6 + self.lsr(b0, b1, AddressingMode::ZeroPageX)),
            0x4E => (3, 6 + self.lsr(b0, b1, AddressingMode::Absolute)),
            0x5E => (3, 7 + self.lsr(b0, b1, AddressingMode::AbsoluteX)),

            0x20 => (3, 6 + self.jsr(b0, b1, AddressingMode::Absolute)),

            0x78 => (1, 2 + self.sei(b0, b1, AddressingMode::Implicit)),

            0xD8 => (1, 2 + self.cld(b0, b1, AddressingMode::Implicit)),

            0xA9 => (2, 2 + self.lda(b0, b1, AddressingMode::Immediate)),
            0xA5 => (2, 3 + self.lda(b0, b1, AddressingMode::ZeroPage)),
            0xB5 => (2, 4 + self.lda(b0, b1, AddressingMode::ZeroPageX)),
            0xAD => (3, 4 + self.lda(b0, b1, AddressingMode::Absolute)),
            0xBD => (3, 4 + self.lda(b0, b1, AddressingMode::AbsoluteX)),
            0xB9 => (3, 4 + self.lda(b0, b1, AddressingMode::AbsoluteY)),
            0xA1 => (2, 6 + self.lda(b0, b1, AddressingMode::IndexedIndirectX)),
            0xB1 => (2, 5 + self.lda(b0, b1, AddressingMode::IndirectIndexedY)),

            0xC9 => (2, 2 + self.cmp(b0, b1, AddressingMode::Immediate)),
            0xC5 => (2, 3 + self.cmp(b0, b1, AddressingMode::ZeroPage)),
            0xD5 => (2, 4 + self.cmp(b0, b1, AddressingMode::ZeroPageX)),
            0xCD => (3, 4 + self.cmp(b0, b1, AddressingMode::Absolute)),
            0xDD => (3, 4 + self.cmp(b0, b1, AddressingMode::AbsoluteX)),
            0xD9 => (3, 4 + self.cmp(b0, b1, AddressingMode::AbsoluteY)),
            0xC1 => (2, 6 + self.cmp(b0, b1, AddressingMode::IndexedIndirectX)),
            0xD1 => (2, 5 + self.cmp(b0, b1, AddressingMode::IndirectIndexedY)),

            0x85 => (2, 3 + self.sta(b0, b1, AddressingMode::ZeroPage)),
            0x95 => (2, 4 + self.sta(b0, b1, AddressingMode::ZeroPageX)),
            0x8D => (3, 4 + self.sta(b0, b1, AddressingMode::Absolute)),
            0x9D => (3, 5 + self.sta(b0, b1, AddressingMode::AbsoluteX)),
            0x99 => (3, 5 + self.sta(b0, b1, AddressingMode::AbsoluteY)),
            0x81 => (2, 6 + self.sta(b0, b1, AddressingMode::IndexedIndirectX)),
            0x91 => (2, 6 + self.sta(b0, b1, AddressingMode::IndirectIndexedY)),

            0xA2 => (2, 2 + self.ldx(b0, b1, AddressingMode::Immediate)),

            0x9A => (1, 2 + self.txs(b0, b1, AddressingMode::Implicit)),

            0xF0 => (2, 2 + self.beq(b0, b1, AddressingMode::Relative)),
            0xD0 => (2, 2 + self.ben(b0, b1, AddressingMode::Relative)),


            0x1A  => (1, 2),
        
            _    => panic!("Unknown instruction {:#04x}", op)
              
        }        
    }

    fn get_address_and_cycles(&self, b0: u8, b1: u8, mode: AddressingMode) -> (Address,u8) {
        let b0_u16 = b0 as u16;
        let b1_u16 = b1 as u16;
        let x_u16  = self.x as u16;
        let y_u16  = self.y as u16;
        let zero_page_x = (b0_u16 + x_u16) & 0xFF;
        let zero_page_y = (b0_u16 + y_u16) & 0xFF;
        let mut add_cycles = 0;
    
        match mode {
            AddressingMode::Implicit    => (Address::Implicit, 0),
            AddressingMode::Accumulator => (Address::Accumulator, 0),
            AddressingMode::Immediate   => (Address::Immediate(b0), 0),
            AddressingMode::ZeroPage    => (Address::RAM(b0_u16),0),
            AddressingMode::ZeroPageX   => (Address::RAM(zero_page_x),0),
            AddressingMode::ZeroPageY   => (Address::RAM(zero_page_y),0),
            AddressingMode::Absolute    => (Address::RAM(convert_2u8_to_u16(b1,b0)),0),
            AddressingMode::AbsoluteX   => {
                if b0_u16 + x_u16 > 0xFF {add_cycles = 1} 
                (Address::RAM(b0_u16 + x_u16 + (b1_u16<< 4)),add_cycles)
            }
            AddressingMode::AbsoluteY   => {
                if b0_u16 + y_u16 > 0xFF {add_cycles = 1} 
                (Address::RAM(b0_u16 + y_u16 + (b1_u16<< 4)),add_cycles)
            }
            AddressingMode::IndexedIndirectX   => {
                if zero_page_x == 0xFF {panic!("Invalid ZeroPageX address!")}
                let ind = convert_2u8_to_u16(self.ram.get_byte(zero_page_x),
                                             self.ram.get_byte(zero_page_x + 1));
                (Address::RAM(ind),0)
            }
            AddressingMode::IndirectIndexedY   =>  {
                let ind = self.ram.get_byte(b0_u16) as u16 + y_u16;
                if ind & 0xFF00 != 0 {add_cycles = 1}

                let ind = convert_2u8_to_u16(self.ram.get_byte(ind),
                                             self.ram.get_byte(ind + 1));
                (Address::RAM(ind), add_cycles)
            }
            AddressingMode::Relative   =>  {
                println!("Relative adressing signed offset {}", b0 as i8 as i16 );
                let new_pc = (self.pc as i16 + (b0 as i8 as i16)) as u16;
                if new_pc & 0xFF00 != self.pc & 0xFF00 {add_cycles = 1}
                (Address::Relative(new_pc), add_cycles)
            }
            _ => panic!("Invalid addresing mode {}",mode as u8),
        }
    }

    fn load_from_address(&self, address : &Address) -> u8 {
        match address {
            Address::Implicit => panic!("load_from_address can't be used for implicit mode"), 
            Address::Accumulator => self.a, 
            Address::Immediate(i) => *i, 
            Address::RAM(address) => self.ram.get_byte(*address),
            Address::Relative(_)  => panic!("load_from_address can't be used for the Relative mode"), 
        }
    }

    fn get_ram_address(&self, address : &Address) -> u16 {
        match address {
            Address::RAM(address) => *address,
            _ => panic!("Invalid address type {:?}",address),
        }
    }

    fn store_to_address(&mut self, address : &Address, byte : u8) {
        match address {
            Address::Implicit     => panic!("store_to_address can't be used for implicit mode"), 
            Address::Accumulator  => self.a = byte, 
            Address::Immediate(_) => panic!("Not possible to store in Immediate addressing"), 
            Address::RAM(address) => self.ram.store_byte(*address, byte),
            Address::Relative(_)  => panic!("store_to_address can't be used for the Relative mode"), 
        }
    }

    fn add_with_carry(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("add_with_carry");
        let a_i8  = self.a as i8;
        let c_u8 = self.carry();
        let c_i8 = c_u8 as i8;
        let c_i16 = c_i8 as i16;
        let c_u16 = c_u8 as u16;

        let (m_address, cycles)  = self.get_address_and_cycles(b0, b1, mode);
        let m_u8  = self.load_from_address(&m_address); 
        let m_u16 = m_u8 as u16;
        let m_i16 = m_u8 as i16;
        let m_i8  = m_u8 as i8;
           
        let result_u16 : u16 = m_u16 + c_u16 + (self.a as u16);
        let result_i16 : i16 = m_i16 + c_i16 + (self.a as i16);

        if  result_u16 > 255 {
            self.a = (result_i16 - 255) as u8;
            self.set_flag(ProcessorFlag::CarryFlag);
        }
        else {
            self.a = result_i16 as u8;
            self.reset_flag(ProcessorFlag::CarryFlag);
        }
        if  self. a == 0 {self.set_flag(ProcessorFlag::ZeroFlag);}
        else             {self.reset_flag(ProcessorFlag::ZeroFlag);}

        if result_i16 < 0 {self.set_flag(ProcessorFlag::NegativeFlag);}
        else              {self.reset_flag(ProcessorFlag::NegativeFlag);}

        self.reset_flag(ProcessorFlag::OverflowFlag);
        if m_i8 > 0 && a_i8 > 0 && (self.a as i8) < 0 {self.set_flag(ProcessorFlag::OverflowFlag);}
        if m_i8 < 0 && a_i8 < 0 && (self.a as i8) > 0 {self.set_flag(ProcessorFlag::OverflowFlag);}
        cycles
    }

    fn and(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("and {:?}", mode);
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self.a &= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, (self.a as i8) < 0);
        cycles        
    }

    fn or(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("or");
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let m = self.load_from_address(&m_address);
        self.a |= m;
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, self.a == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, (self.a as i8) < 0);
        cycles        
    }

     fn lsr(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("lsr {:?}", mode);
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let mut m = self.load_from_address(&m_address);
        m >>= 1;
        self.store_to_address(&m_address, m);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m & 0x80 == 1);
        self.set_or_reset_flag(ProcessorFlag::ZeroFlag, m == 0);
        self.set_or_reset_flag(ProcessorFlag::NegativeFlag, (m as i8) < 0);
        cycles        
    }

    fn jsr(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.stack.push(self.pc + 3);
        let raw_address = self.get_ram_address(&m_address);
        println!("jsr raw {:?} b0 {} b1 {}", raw_address,b0,b1);
        self.pc = self.get_ram_address(&m_address) - 2;
        cycles        
    }

    fn sei(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.set_flag(ProcessorFlag::InterruptDisable);
        println!("sei");
        cycles        
    }

    fn cld(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.reset_flag(ProcessorFlag::DecimalMode);
        println!("cld");
        cycles        
    }

    fn lda(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("lda {:?}", mode);
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let  m = self.load_from_address(&m_address);
        self.a = m;
        cycles
    }

    fn ldx(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("ldx {:?}", mode);
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let  m = self.load_from_address(&m_address);
        self.x = m;
        cycles
    }

    fn sta(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("sta {:?}", mode);
        let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.store_to_address(&m_address,self.a);
        cycles
    }

    fn txs(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("txs {:?}", mode);
        let (_, cycles) = self.get_address_and_cycles(b0, b1, mode);
        self.sp = self.x;
        cycles
    }

    fn beq(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("beq {:?}", mode);
        if self.get_flag(ProcessorFlag::ZeroFlag) {
            println!("Performing branch");
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _  => panic!("Unexpected address type in beq.")
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn ben(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("ben {:?}", mode);
        if !self.get_flag(ProcessorFlag::ZeroFlag) {
            let (m_address, cycles) = self.get_address_and_cycles(b0, b1, mode);
            let new_pc = match m_address {
                Address::Relative(new_pc) => new_pc,
                _  => panic!("Unexpected address type in beq.")
            };
            self.pc = new_pc;
            return cycles + 1;
        }
        0
    }

    fn cmp(&mut self, b0: u8, b1: u8, mode: AddressingMode) -> u8 {
        println!("cmp {:?}", mode);
        let (address, cycles) = self.get_address_and_cycles(b0, b1, mode);
        let  m = self.load_from_address(&address);
        if self.a == m {
            self.set_flag(ProcessorFlag::ZeroFlag)
        }
        if self.a >= m {
            self.set_flag(ProcessorFlag::CarryFlag)
        }
        if self.a < m {
            self.set_flag(ProcessorFlag::NegativeFlag)
        }
        cycles
    }
}

