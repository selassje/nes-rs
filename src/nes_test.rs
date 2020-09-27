use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{io::DumpFrame, io::io_test, nes::Nes, read_rom};

pub struct NesTest {
    nes : Nes,
    io_test : Rc<RefCell<io_test::IOTest>>,
    rom_path : String,
}

impl NesTest {
    pub fn new(rom_path: &str) -> Self {
        let io_test  = Rc::new(RefCell::new(io_test::IOTest::new(rom_path))); 
        let mut nes = Nes::new(io_test.clone());
        let nes_file = read_rom(rom_path);
        nes.load(&nes_file);
        NesTest {
            io_test,
            nes,
            rom_path : String::from(rom_path)
        }
    }

    pub fn run_for(&mut self, duration: Duration) {
        self.nes.run(Some(duration))
    }

    pub fn dump_frame(&self) {
        let path = self.rom_path.to_owned() + ".bmp";
        self.io_test.borrow().dump_frame(&path)
    }
}