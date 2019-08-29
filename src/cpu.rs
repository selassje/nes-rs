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

pub const MILLISECONDS_PER_CYCLE : u64 = 200 /  60 ;

type Keyboard = [bool; 16];

pub struct CPU<'a>
{
   v_regs       : [u8; 16],
   i_reg        : u16,
   pc           : u16,
   stack        : LinkedList<u16>,
   d_reg        : u8,
   t_reg        : u8,
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
            v_regs      : [0;16],
            i_reg       : 0,
            pc          : 0x200,
            stack       : LinkedList::<u16>::new(),
            d_reg       : 0,
            t_reg       : 0,
            ram         : ram,
            screen_tx   : screen_tx,
            screen      : [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            rand        : thread_rng(),
            keyboard    : [false; 16],
            keyboard_rx : keyboard_rx,
            audio_tx    : audio_tx,
        }
    }

    pub fn run(&mut self, ins_count : u16) {

        println!("PC start location {} end location {}", self.pc, self.pc +  (ins_count - 1));

        while self.pc - 0x200  < (ins_count - 1) {
            let now = Instant::now();

            if self.pc % 2 == 1 {
                panic!("PC ar wrong location {}", self.pc);
            }

            let mut ins = [0 as u8;4];
            ins[0] = self.ram.get_byte(self.pc) >> 4;
            ins[1] = self.ram.get_byte(self.pc) & 0x0F;
            ins[2] = self.ram.get_byte(self.pc + 1) >> 4;
            ins[3] = self.ram.get_byte(self.pc + 1) & 0x0F;

            self.execute_instruction(ins);
            self.pc += 2;
    
            while let Ok(key_event) = self.keyboard_rx.try_recv() { 
                match key_event {
                    KeyEvent::KeyDown(key) => {
                        self.keyboard[key as usize] = true;
                    },
                    KeyEvent::KeyUp(key) => {
                        self.keyboard[key as usize] = false;
                    } 
                }
            }
            if self.d_reg > 0 { self.d_reg -= 1; }

            if self.t_reg > 0 {
                self.t_reg -= 1; 
                if self.t_reg == 0 {
                      self.audio_tx.send(true).unwrap(); 
                }
            }

            let elapsed_milliseconds  = now.elapsed().as_millis() as u64;
            if MILLISECONDS_PER_CYCLE > elapsed_milliseconds {
               sleep(Duration::from_millis(MILLISECONDS_PER_CYCLE - elapsed_milliseconds));
            }
        }
        println!("CPU Stopped execution")
    }

    fn execute_instruction(&mut self, ins : [u8; 4]) {
            match (ins[0], ins[1], ins[2], ins[3])
            {
                (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
                (0x0, 0x0, 0xE, 0xE) => self.ret(),
                (0x0, 0x0, 0x0, 0x0) => (),
                (0x1, _, _, _) => self.jump(ins[1],ins[2],ins[3]),
                (0x2, _, _, _) => self.call(ins[1],ins[2],ins[3]),
                (0x3, _, _, _) => self.skip_if(ins[1],ins[2],ins[3]),
                (0x4, _, _, _) => self.skip_ifnot(ins[1],ins[2],ins[3]),
                (0x5, _, _, 0x0) => self.skip_ifregs(ins[1],ins[2]),
                (0x6, _, _, _) => self.load(ins[1],ins[2],ins[3]),
                (0x7, _, _, _) => self.add(ins[1],ins[2],ins[3]),
                (0x8, _, _, 0x0) => self.store(ins[1],ins[2]),
                (0x8, _, _, 0x1) => self.store_or(ins[1],ins[2]),
                (0x8, _, _, 0x2) => self.store_and(ins[1],ins[2]),
                (0x8, _, _, 0x3) => self.store_xor(ins[1],ins[2]),
                (0x8, _, _, 0x4) => self.add_with_carry(ins[1],ins[2]),
                (0x8, _, _, 0x5) => self.sub(ins[1],ins[2]),
                (0x8, _, _, 0x6) => self.shr(ins[1]),
                (0x8, _, _, 0x7) => self.subn(ins[1],ins[2]),
                (0x8, _, _, 0xE) => self.shl(ins[1]),
                (0x9, _, _, 0x0) => self.skip_ifnotregs(ins[1],ins[2]),
                (0xA, _, _, _) =>   self.set_i(ins[1],ins[2],ins[3]),
                (0xB, _, _, _) =>   self.jump_v0(ins[1],ins[2],ins[3]),
                (0xC, _, _, _) =>   self.rand_and(ins[1],ins[2],ins[3]),
                (0xD, _, _, _) =>   self.disp(ins[1],ins[2],ins[3]),
                (0xE, _, 0x9, 0xE) => self.skip_ifkey(ins[1]),
                (0xE, _, 0xA, 0x1) => self.skip_ifnotkey(ins[1]),
                (0xF, _, 0x0, 0x7) => self.load_dt(ins[1]),
                (0xF, _, 0x0, 0xA) => self.wait_for_key(ins[1]),
                (0xF, _, 0x1, 0x5) => self.set_dt(ins[1]),
                (0xF, _, 0x1, 0x8) => self.set_st(ins[1]),
                (0xF, _, 0x1, 0xE) => self.add_i(ins[1]),
                (0xF, _, 0x2, 0x9) => self.set_i_sprite(ins[1]),
                (0xF, _, 0x3, 0x3) => self.store_digits(ins[1]),
                (0xF, _, 0x5, 0x5) => self.store_registers_at_i(ins[1]),
                (0xF, _, 0x6, 0x5) => self.load_registers_at_i(ins[1]),
                (_,_,_,_) => println!("Unknown instruction {} {} {} {}",ins[0],ins[1],ins[2],ins[3]),
            }
    }
/*
00E0 - CLS
Clear the display. */
    fn clear_screen(&mut self) {
        //println!("Clear display");
        self.screen =  [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        self.screen_tx.send(self.screen).unwrap();
    }
/*
00EE - RET
Return from a subroutine.

The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
*/
    fn ret(&mut self) {
        // println!("return");
         self.pc = self.stack.pop_back().unwrap();
    }
/*1nnn - JP addr
Jump to location nnn.

The interpreter sets the program counter to nnn.
*/

    fn jump(&mut self, n1: u8, n2: u8, n3:u8 ) {
        //println!("jump {} {} {} ", n1 ,n2 ,n3);
        self.pc = convert_3n_to_u16(n1, n2, n3);
        self.pc -= 2;
    }
/*Bnnn - JP V0, addr
Jump to location nnn + V0.

The program counter is set to nnn plus the value of V0.*/

    fn jump_v0(&mut self, n1: u8, n2: u8, n3:u8 ) {
        //println!("jump v0");
        self.jump(n1, n2, n3);
        self.pc += self.v_regs[0x0] as u16;
    }
/*
2nnn - CALL addr
Call subroutine at nnn.

The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
*/

    fn call(&mut self, n1: u8, n2: u8, n3:u8 ) {
        self.stack.push_back(self.pc);
        self.pc = convert_3n_to_u16(n1, n2, n3);
        self.pc -= 2;
       // println!("call");
    }
/*
    Skip next instruction if Vx = kk.

    The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
*/
    fn skip_if(&mut self, x: u8, k1: u8, k2: u8) {
        let byte  = convert_2n_to_u8(k1, k2);
        if self.v_regs[x as usize] == byte {
            self.pc += 2;
        }
       // println!("skip_if {} {} k1 {} k2 {}", self.v_regs[x as usize], byte,k1,k2);
    }
/*
    Skip next instruction if Vx != kk.

    The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
*/
    fn skip_ifnot(&mut self, x: u8, k1: u8, k2: u8) {
        if self.v_regs[x as usize] != convert_2n_to_u8(k1, k2) {
            self.pc += 2;
        }
       // println!("skip_ifnot");
    }
/* 
    Skip next instruction if Vx = Vy.

    The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
*/
    fn skip_ifregs(&mut self, x: u8, y: u8) {
        //println!("skip_ifregs");
        if self.v_regs[x as usize] == self.v_regs[y as usize] {
            self.pc += 2;
        }
    }

/* 
    Skip next instruction if Vx != Vy.

    The interpreter compares register Vx to register Vy, and if they are not equal, increments the program counter by 2.1
*/
    fn skip_ifnotregs(&mut self, x: u8, y: u8) {
        //println!("skip_ifnotregs");
        if self.v_regs[x as usize] != self.v_regs[y as usize] {
            self.pc += 2;
        }
    }
/*Ex9E - SKP Vx
Skip next instruction if key with the value of Vx is pressed.

Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.*/    
    fn skip_ifkey(&mut self, x: u8) {
        let vx = self.v_regs[x as usize];
        assert!(vx < 16);
        if self.keyboard[vx as usize] {
            self.pc += 2;
        }
    }

/*
ExA1 - SKNP Vx
Skip next instruction if key with the value of Vx is not pressed.

Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
*/
    fn skip_ifnotkey(&mut self, x: u8) {
        let vx = self.v_regs[x as usize];
        assert!(vx < 16);
        if !self.keyboard[vx as usize] {
            self.pc += 2;
        }
    }

/*
x0A - LD Vx, K
Wait for a key press, store the value of the key in Vx.

All execution stops until a key is pressed, then the value of that key is stored in Vx.
*/
    fn wait_for_key(&mut self, x: u8) {
        println!("wait for key");
        let key_down : u8;
        'wait: loop {
            if let Ok(key_event) = self.keyboard_rx.try_recv() {
                match key_event {
                    KeyEvent::KeyDown(key) => {self.keyboard[key as usize] = true; key_down = key; break 'wait},
                    KeyEvent::KeyUp(key) =>  self.keyboard[key as usize] = false
                }
            }
        }
        self.v_regs[x as usize] = key_down;
        println!("wait_for finished");
    }

/*
Set Vx = kk.

The interpreter puts the value kk into register Vx.
*/
    fn load(&mut self, x: u8, k1: u8, k2: u8) {
        let byte = convert_2n_to_u8(k1, k2);
        self.v_regs[x as usize] = byte;
        //println!("load {} to v{}", byte,x);
    }
/*
x65 - LD Vx, [I]
Read registers V0 through Vx from memory starting at location I.

The interpreter reads values from memory starting at location I into registers V0 through Vx.
*/

    fn load_registers_at_i(&mut self, x: u8) {
        //println!("load_registers_at_i up to v{} from {:#06x}",x, self.i_reg);
        for i in 0..(x+1) {
            self.v_regs[i as usize] = self.ram.get_byte(self.i_reg + (i as u16));
        }
    }

/*
    Set Vx = Vx + kk.

    Adds the value kk to the value of register Vx, then stores the result in Vx.
*/
    fn add(&mut self, x: u8, k1: u8, k2: u8) {
        let vx = self.v_regs[x as usize];
        let byte = convert_2n_to_u8(k1, k2);
        self.v_regs[x as usize] = ((byte as u16) + (vx as u16)) as u8;
        //println!("add {}({} {}) to v({}){}; result {}", byte,k1,k2,x,vx,self.v_regs[x as usize]);
    }
/*
    Fx1E - ADD I, Vx
    Set I = I + Vx.

    The values of I and Vx are added, and the results are stored in I.
*/

    fn add_i(&mut self, x: u8) {
        //println!("add_i");
        self.i_reg += self.v_regs[x as usize] as u16;
    }
/*
Set Vx = Vx + Vy, set VF = carry.

The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
*/
    fn add_with_carry(&mut self, x: u8, y: u8) {
        //println!("add_with_carry");
        let result : u16 = (self.v_regs[x as usize] as u16) + (self.v_regs[y as usize] as u16);
        if result > 255 {
            self.v_regs[0xF] = 1;
        }
        else {
            self.v_regs[0xF] = 0;
        }
        self.v_regs[x as usize] = (0x00FF & result) as u8;
    }  
/*
Set Vx = Vy.

Stores the value of register Vy in register Vx.
*/
    fn store(&mut self, x: u8, y: u8) {
        //println!("store");
        self.v_regs[x as usize] = self.v_regs[y as usize];
    }
/*
Set Vx = Vx OR Vy.

Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0. 
*/
    fn store_or(&mut self, x: u8, y: u8) {
        //println!("store_or");
        self.v_regs[x as usize] |= self.v_regs[y as usize];
    }
/*
Set Vx = Vx AND Vy.

Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0. 
*/

    fn store_and(&mut self, x: u8, y: u8) {
        //println!("store_and");
        self.v_regs[x as usize] &= self.v_regs[y as usize];
    }
/*
Set Vx = Vx XOR Vy.

Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0. 
*/
    fn store_xor(&mut self, x: u8, y: u8) {
        //println!("store_xor");
        self.v_regs[x as usize] ^= self.v_regs[y as usize];
    }

/*
Fx33 - LD B, Vx
Store BCD representation of Vx in memory locations I, I+1, and I+2.

The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
*/
    fn store_digits(&mut self, x: u8) {
        
        let mut v_x = self.v_regs[x as usize];
        let hundreds =  v_x / 100;
        v_x -= hundreds * 100;
        let tens =  v_x / 10;
        v_x -= tens * 10;
        let ones = v_x;

        self.ram.store_byte(self.i_reg, hundreds);
        self.ram.store_byte(self.i_reg + 1, tens);
        self.ram.store_byte(self.i_reg + 2 , ones);

        //println!("store_digits {} {} {} of v{}({}) at {:#06x}",hundreds,tens, ones,x,self.v_regs[x as usize],self.i_reg);
    }
/*
Fx55 - LD [I], Vx
Store registers V0 through Vx in memory starting at location I.

The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
*/
    fn store_registers_at_i(&mut self, x: u8) {
        //println!("store_registers_at_i up to v{} at {:#06x}",x, self.i_reg);
        for i in 0..(x + 1) {
            let address : u16 = self.i_reg + (i as u16);
            self.ram.store_byte(address , self.v_regs[i as usize]);
        }
    }

/*
8xy5 - SUB Vx, Vy
Set Vx = Vx - Vy, set VF = NOT borrow.

If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
*/
    fn sub(&mut self, x: u8, y: u8) {
        //println!("sub");
        if self.v_regs[x as usize] > self.v_regs[y as usize] {
            self.v_regs[x as usize] -= self.v_regs[y as usize];
            self.v_regs[0xF] = 1;
        } else {
            self.v_regs[x as usize] = self.v_regs[y as usize] - self.v_regs[x as usize];
            self.v_regs[0xF] = 0;
        }
    }

/*
8xy7 - SUBN Vx, Vy
Set Vx = Vy - Vx, set VF = NOT borrow.

If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
*/

    fn subn(&mut self, x: u8, y: u8) {
        //println!("subn");
        self.sub(x,y);
    }

/*8xy6 - SHR Vx {, Vy}
Set Vx = Vx SHR 1.

If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
*/
    fn shr(&mut self, x: u8) {
        //println!("shr");
        self.v_regs[0xF] = self.v_regs[ x as usize ] & 0x1;
        self.v_regs[x as usize] >>= 1;
    }

/*
8xyE - SHL Vx {, Vy}
Set Vx = Vx SHL 1.

If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
*/
    fn shl(&mut self, x: u8) {
        //println!("shl");
        self.v_regs[0xF] = self.v_regs[ x as usize ] & 0x80;
        self.v_regs[x as usize] <<= 1;
    }

/*nnn - LD I, addr
Set I = nnn.

The value of register I is set to nnn.
*/

    fn set_i(&mut self, n1: u8, n2: u8, n3: u8 ) {
        
        self.i_reg = convert_3n_to_u16(n1, n2, n3);
        //println!("set_i {:#06x} ", self.i_reg);
    }
/*
Fx29 - LD F, Vx
Set I = location of sprite for digit Vx.

The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
*/
    fn set_i_sprite(&mut self, x: u8 ) {
        self.i_reg = (self.v_regs[x as usize] * 5) as u16;
        //println!("set_i_sprite v{}({}) i {:#06x}",x, self.v_regs[x as usize], self.i_reg);
    }

/*
Fx15 - LD DT, Vx
Set delay timer = Vx.

DT is set equal to the value of Vx.
*/
    fn set_dt(&mut self, x: u8) {
       //println!("set_dt {} ", self.v_regs[x as usize]);
        self.d_reg = self.v_regs[x as usize];
    }
/*
Fx07 - LD Vx, DT
Set Vx = delay timer value.

The value of DT is placed into Vx.
*/

    fn load_dt(&mut self, x: u8) {
        self.v_regs[x as usize] = self.d_reg;
        // println!("load_dt {} to v{}",self.d_reg, x);
    }

/*
Fx18 - LD ST, Vx
Set sound timer = Vx.

ST is set equal to the value of Vx.
*/
    fn set_st(&mut self, x: u8) {
        //println!("set_st");
        self.t_reg = self.v_regs[x as usize];
    }

/*
Cxkk - RND Vx, byte
Set Vx = random byte AND kk.

The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
*/

    fn rand_and(&mut self, x: u8, k1: u8, k2: u8) {
        //println!("random_and");
        let r = self.rand.gen_range(0, 256) as u8;
        let kk = convert_2n_to_u8(k1, k2);
        self.v_regs[x as usize] = kk & r;
    }
/*
Dxyn - DRW Vx, Vy, nibble
Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.

The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
*/

    fn disp(&mut self, x: u8, y: u8, n: u8) {
        self.v_regs[0xF] = 0;
        let org_x  = self.v_regs[x as usize] as usize; 
        let org_y  = self.v_regs[y as usize] as usize;
        for i in 0..n {
            let byte = self.ram.get_byte(self.i_reg + (i as u16));      
            let disp_y = (org_y + (i as usize) ) % DISPLAY_HEIGHT;
            for b in 0 .. 8{
                let bit = get_bit(byte, 7 - b as u8);
                let disp_x = (org_x + (b as usize) ) % DISPLAY_WIDTH;
                if  self.screen[disp_x][disp_y] && bit {
                    self.v_regs[0xF] = 1;
                } 
                self.screen[disp_x][disp_y] = ! (bit == self.screen[disp_x][disp_y]);
            }
        }
        self.screen_tx.send(self.screen).unwrap();
    }
}
