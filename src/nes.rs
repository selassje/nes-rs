use crate::cpu::{CPU};
use crate::ppu::{PPU};
use crate::apu::{APU};
use crate::keyboard::KeyboardController;
use crate::controllers::Controllers;
use crate::nes_format_reader::NesFile;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes<'a> {
    cpu : Option<CPU<'a>>,
    ppu : Option<RefCell<PPU>>,
    apu : Option<Rc<RefCell<APU>>>,
    controllers : Option<Rc<RefCell<Controllers>>>
}


impl<'a> Nes<'a> {

    pub fn new(nes_file : &NesFile) -> Self {
        let mapper = nes_file.create_mapper();
        let controller_1 = KeyboardController::get_default_keyboard_controller_player1();
        let controller_2 = KeyboardController::get_default_keyboard_controller_player2();
     
        let controllers = Rc::new(RefCell::new(Controllers::new(Box::new(controller_1), Box::new(controller_2))));
                                                     
        //let ppu = ;
        let apu = Rc::new(RefCell::new(APU::new()));
        //let cpu = Rc::new(RefCell::new(CPU::new(mapper, ppu, apu.clone(), controllers.clone())));
        let mut nesTmp = Nes {
                cpu : None,
                ppu : Some(RefCell::new(PPU::new(mapper.get_chr_rom().to_vec(),nes_file.get_mirroring()))),
                apu : None,
                controllers : None,
        };


        nesTmp.cpu = Some(CPU::new(mapper, nesTmp.ppu.as_ref().unwrap(), apu.clone(), controllers.clone()));

        nesTmp
    }

    pub fn run_cpu_instruction(&mut self) {
        self.cpu.as_mut().unwrap().run_next_instruction();
    }

    pub fn run(&mut self) {
        self.cpu.as_mut().unwrap().run2();
    }
}