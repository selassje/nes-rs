use self::Button::*;
use crate::cpu_controllers::*;
use std::cell::Cell;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
        match value {
            0 => A,
            1 => B,
            2 => Select,
            3 => Start,
            4 => Up,
            5 => Down,
            6 => Left,
            7 => Right,
            _ => panic!("Can't cast {} to Button", value),
        }
    }
}

pub trait Controller {
    fn is_button_pressed(&self, button: Button) -> u8;
}

pub struct Controllers {
    controller_1: Box<dyn Controller>,
    controller_2: Box<dyn Controller>,
    strobe: bool,
    next_controller_button: Cell<u8>,
}

impl Controllers {
    pub fn new(controller_1: Box<dyn Controller>, controller_2: Box<dyn Controller>) -> Self {
        Controllers {
            controller_1: controller_1,
            controller_2: controller_2,
            strobe: true,
            next_controller_button: Cell::new(A as u8)
        }
    }
}

impl ReadInputPorts for Controllers {
    fn read(&self, port: InputPort) -> u8 {
        let (mut controller1_output, mut controller2_output) = (1, 1);
        if self.next_controller_button.get() < 8 {
            controller1_output = self
                .controller_1
                .is_button_pressed(self.next_controller_button.get().into());
            controller2_output = self
                .controller_2
                .is_button_pressed(self.next_controller_button.get().into());
            if !self.strobe {
                self.next_controller_button.set(self.next_controller_button.get() + 1);
            }
        }
        match port {
            InputPort::Controller1 => controller1_output | 0x40,
            InputPort::Controller2 => controller2_output | 0x40,
        }
    }
}

impl WriteOutputPorts for Controllers {
    fn write(&mut self, port: OutputPort, value: u8) {
        assert!(port == OutputPort::Controllers1And2);
        self.strobe = (1 & value) != 0;
        if self.strobe {
            self.next_controller_button.set(A as u8);
        }
    }
}

impl ControllerPortsAccess for Controllers {

}