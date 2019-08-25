use crate::memory::{RAM};

pub struct CPU<'a>
{
   v_regs : [u8; 16],
   i_reg  : u16,
   pc     : u16,
   sp     : u8,
   d_reg  : u8,
   t_reg  : u8,
   ram    : &'a RAM
}

impl<'a> CPU<'a>
{
    pub fn new(ram : &'a RAM) -> CPU<'a>
    {
        CPU
        {
            v_regs : [0;16],
            i_reg  : 0,
            pc     : 0x200,
            sp     : 0,
            d_reg  : 0,
            t_reg  : 0,
            ram    : ram,
        }
    }

    pub fn run(&mut self, ins_count : u16) {

        while self.pc - 0x200  < (ins_count - 1) {
            let mut ins = [0 as u8;4];
            ins[0] = self.ram.get_byte(self.pc as usize) >> 4;
            ins[1] = self.ram.get_byte(self.pc as usize) & 0x0F;
            ins[2] = self.ram.get_byte(self.pc as usize + 1) >> 4;
            ins[3] = self.ram.get_byte(self.pc as usize + 1) & 0x0F;

            self.execute_instruction(ins);
            self.pc += 2;
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

    fn clear_screen(&mut self) {
        println!("Clear screen");
    }

    fn ret(&mut self) {
         println!("return");
    }

    fn jump(&mut self, a1: u8, a2: u8, a3:u8 ) {
        println!("jump");
    }

    fn jump_v0(&mut self, a1: u8, a2: u8, a3:u8 ) {
        println!("jump v0");
    }

    fn call(&mut self, a1: u8, a2: u8, a3:u8 ) {
        println!("call");
    }

    fn skip_if(&mut self, x: u8, k1: u8, k2: u8) {
        println!("skip_if");
    }

    fn skip_ifnot(&mut self, x: u8, k1: u8, k2: u8) {
        println!("skip_ifnot");
    }

    fn skip_ifregs(&mut self, x: u8, y: u8) {
        println!("skip_ifregs");
    }

    fn skip_ifnotregs(&mut self, x: u8, y: u8) {
        println!("skip_ifnotregs");
    }

    fn skip_ifkey(&mut self, x: u8) {
        println!("skip_ifkey");
    }
    
    fn skip_ifnotkey(&mut self, x: u8) {
        println!("skip_ifnotkey");
    }


    fn load(&mut self, x: u8, k1: u8, k2: u8) {
        println!("load");
    }

    fn load_registers_at_i(&mut self, x: u8) {
        println!("load_registers_at_i");
    }

    fn add(&mut self, x: u8, k1: u8, k2: u8) {
        println!("add");
    }

    fn add_i(&mut self, x: u8) {
        println!("add_i");
    }

    fn add_with_carry(&mut self, x: u8, y: u8) {
        println!("add_with_carry");
    }  


    fn store(&mut self, x: u8, y: u8) {
        println!("store");
    }

    fn store_or(&mut self, x: u8, y: u8) {
        println!("store_or");
    }

    fn store_and(&mut self, x: u8, y: u8) {
        println!("store_and");
    }

    fn store_xor(&mut self, x: u8, y: u8) {
        println!("store_xor");
    }

    fn store_digits(&mut self, x: u8) {
        println!("store_digits");
    }

    fn store_registers_at_i(&mut self, x: u8) {
        println!("store_registers_at_i");
    }

    fn sub(&mut self, x: u8, y: u8) {
        println!("sub");
    }

    fn shr(&mut self, x: u8) {
        println!("shr");
    }

    fn subn(&mut self, x: u8, y: u8) {
        println!("subn");
    }

    fn shl(&mut self, x: u8) {
        println!("shl");
    }

    fn set_i(&mut self, a1: u8, a2: u8, a3:u8 ) {
        println!("set_i");
    }

    fn set_i_sprite(&mut self, x: u8 ) {
        println!("set_i_srpite");
    }


    fn set_dt(&mut self, x: u8) {
        println!("setdt");
    }

    fn load_dt(&mut self, x: u8) {
        println!("load");
    }

    fn set_st(&mut self, x: u8) {
        println!("set_st");
    }

    fn rand_and(&mut self, x: u8, k1: u8, k2: u8) {
        println!("random_and");
    }

    fn disp(&mut self, x: u8, y: u8, n: u8) {
        println!("disp");
    }

    fn wait_for_key(&mut self, x: u8) {
        println!("wait_for_key");
    }
}
