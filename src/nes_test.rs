use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{io::io_test, nes::Nes, read_rom};

pub struct NesTest {
    nes: Nes,
    pub io_test: Rc<RefCell<io_test::IOTest>>,
}

impl NesTest {
    pub fn new(rom_path: &str) -> Self {
        let io_test = Rc::new(RefCell::new(io_test::IOTest::new(rom_path)));
        let mut nes = Nes::new(io_test.clone());
        let nes_file = read_rom(rom_path);
        nes.load(&nes_file);
        NesTest {
            io_test,
            nes,
        }
    }

    pub fn run_for(&mut self, duration: Duration) {
        self.nes.run(Some(duration))
    }

    pub fn dump_frame(&self,path: &str) {
        self.io_test.borrow().dump_frame(&path)
    }
}
