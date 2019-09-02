use crate::memory::{RAM};
use crate::screen::{Screen, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use std::collections::LinkedList;
use crate::utils::*;
use rand::rngs::{ThreadRng};
use rand::{Rng, thread_rng};
use std::time::{Duration, Instant};
use std::thread::sleep;
use crate::keyboard::{KeyEvent};
use std::sync::mpsc::{Sender, Receiver};

const NTSC_FREQ_MHZ          : f32 = 1.79;
const NANOS_PER_CYCLE : u128 =  ((1.0/NTSC_FREQ_MHZ) * 1000.0) as u128;
pub const PC_START : u16 = 0x0100;

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
    IndexedIndirect,
    IndirectIndexed,
}

enum Address {
    Accumulator,
    Immediate(u8),
    RAM(u16),
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

pub struct CPU<'a>
{
   pc           : u16,
   sp           : u8,
   ps           : u8,
   a            : u8,
   x            : u8,
   y            : u8,
   stack        : LinkedList<u16>,
   ram          : &'a mut RAM,
   screen_tx    : Sender<Screen>,
   screen       : Screen,
   rand         : ThreadRng,
   keyboard     : Keyboard,
   keyboard_rx  : Receiver<KeyEvent>,
   audio_tx     : Sender<bool>,
}

impl<'a> CPU<'a>
{
    pub fn new(ram : &'a mut RAM, 
               screen_tx: Sender<Screen>, 
               keyboard_rx: Receiver<KeyEvent>,
               audio_tx: Sender<bool>) -> CPU<'a>
    {
        CPU
        {
            pc           : PC_START,
            sp           : 0,
            ps           : 0,
            a            : 0,
            x            : 0,
            y            : 0,
            stack       : LinkedList::<u16>::new(),
            ram         : ram,
            screen_tx   : screen_tx,
            screen      : [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            rand        : thread_rng(),
            keyboard    : [false; 16],
            keyboard_rx : keyboard_rx,
            audio_tx    : audio_tx,
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

 
    pub fn run(&mut self, rom_size : u16) {

        println!("PC start location {} end location {}", self.pc, self.pc +  (rom_size - 1));

        while self.pc - PC_START  < rom_size - 1 {
            let now = Instant::now();
            
            let op = self.ram.get_byte(self.pc);
            let mut b0 = 0;
            let mut b1 = 0;
            if  self.pc - PC_START + 1  < rom_size - 1 {
                b0 = self.ram.get_byte(self.pc + 1);
            }
            if  self.pc - PC_START + 2  < rom_size - 1 {
                b1 = self.ram.get_byte(self.pc + 2);
            }

            let (bytes, cycles) = self.execute_instruction(op, b0, b1);
            println!("Executed instruction {:#0x} bytes {} cycles {}",op, bytes, cycles);
            self.pc += bytes as u16;
            let elapsed_time_ns  = now.elapsed().as_nanos();
            let required_time_ns = (cycles as u128) * NANOS_PER_CYCLE;
            if required_time_ns > elapsed_time_ns {
               sleep(Duration::from_nanos((required_time_ns - elapsed_time_ns) as u64));
            }
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
            0x61 => (2, 6 + self.add_with_carry(b0, b1, AddressingMode::IndexedIndirect)),
            0x71 => (2, 5 + self.add_with_carry(b0, b1, AddressingMode::IndirectIndexed)),

            0x29 => (2, 2 + self.and(b0, b1, AddressingMode::Immediate)),
            0x25 => (2, 3 + self.and(b0, b1, AddressingMode::ZeroPage)),
            0x35 => (2, 4 + self.and(b0, b1, AddressingMode::ZeroPageX)),
            0x2D => (3, 4 + self.and(b0, b1, AddressingMode::Absolute)),
            0x3D => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteX)),
            0x39 => (3, 4 + self.and(b0, b1, AddressingMode::AbsoluteY)),
            0x21 => (2, 6 + self.and(b0, b1, AddressingMode::IndexedIndirect)),
            0x31 => (2, 5 + self.and(b0, b1, AddressingMode::IndirectIndexed)),

            0x09 => (2, 2 + self.or(b0, b1, AddressingMode::Immediate)),
            0x05 => (2, 3 + self.or(b0, b1, AddressingMode::ZeroPage)),
            0x15 => (2, 4 + self.or(b0, b1, AddressingMode::ZeroPageX)),
            0x0D => (3, 4 + self.or(b0, b1, AddressingMode::Absolute)),
            0x1D => (3, 4 + self.or(b0, b1, AddressingMode::AbsoluteX)),
            0x19 => (3, 4 + self.or(b0, b1, AddressingMode::AbsoluteY)),
            0x01 => (2, 6 + self.or(b0, b1, AddressingMode::IndexedIndirect)),
            0x11 => (2, 5 + self.or(b0, b1, AddressingMode::IndirectIndexed)),


            0x4A => (1, 2 + self.lsr(b0, b1, AddressingMode::Accumulator)),
            0x46 => (2, 5 + self.lsr(b0, b1, AddressingMode::ZeroPage)),
            0x56 => (2, 6 + self.lsr(b0, b1, AddressingMode::ZeroPageX)),
            0x4E => (3, 6 + self.lsr(b0, b1, AddressingMode::Absolute)),
            0x5E => (3, 7 + self.lsr(b0, b1, AddressingMode::AbsoluteX)),

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
            AddressingMode::Accumulator => (Address::Accumulator, 0),
            AddressingMode::Immediate   => (Address::Immediate(b0), 0),
            AddressingMode::ZeroPage    => (Address::RAM(b0_u16),0),
            AddressingMode::ZeroPageX   => (Address::RAM(zero_page_x),0),
            AddressingMode::ZeroPageY   => (Address::RAM(zero_page_y),0),
            AddressingMode::Absolute    => (Address::RAM(b0_u16 + (b1_u16 << 4)),0),
            AddressingMode::AbsoluteX   => {
                if b0_u16 + x_u16 > 0xFF {add_cycles = 1} 
                (Address::RAM(b0_u16 + x_u16 + (b1_u16 << 4)),add_cycles)
            }
            AddressingMode::AbsoluteY   => {
                if b0_u16 + y_u16 > 0xFF {add_cycles = 1} 
                (Address::RAM(b0_u16 + y_u16 + (b1_u16 << 4)),add_cycles)
            }
            AddressingMode::IndexedIndirect   => {
                if zero_page_x == 0xFF {panic!("Invalid ZeroPageX address!")}
                let ind = convert_2u8_to_u16(self.ram.get_byte(zero_page_x),
                                             self.ram.get_byte(zero_page_x + 1));
                (Address::RAM(ind),0)
            }
            AddressingMode::IndirectIndexed   =>  {
                let ind = self.ram.get_byte(b0_u16) as u16 + y_u16;
                if ind & 0xFF00 != 0 {add_cycles = 1}

                let ind = convert_2u8_to_u16(self.ram.get_byte(ind),
                                             self.ram.get_byte(ind + 1));
                (Address::RAM(ind), add_cycles)
            }
            _ => panic!("Invalid addresing mode {}",mode as u8),
        }
    }

    fn load_from_address(&self, address : &Address) -> u8 {
        match address {
            Address::Accumulator => self.a, 
            Address::Immediate(i) => *i, 
            Address::RAM(address) => self.ram.get_byte(*address)
        }
    }

    fn store_to_address(&mut self, address : &Address, byte : u8) {
        match address {
            Address::Accumulator  => self.a = byte, 
            Address::Immediate(i) => panic!("Not possible to store in Immediate addressing"), 
            Address::RAM(address) => self.ram.store_byte(*address, byte)
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
        println!("and");
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
}

