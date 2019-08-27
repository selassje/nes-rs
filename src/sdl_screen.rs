use crate::screen::{ScreenController, Screen, DISPLAY_HEIGHT, DISPLAY_WIDTH};

use std::sync::mpsc::{Sender};

pub struct SdlScreenController {
    sink : Sender<Screen>,
}

impl SdlScreenController {

    pub fn new(sink : Sender<Screen>) -> SdlScreenController
    {
        SdlScreenController {
            sink: sink,
        }
    }
}

impl ScreenController for SdlScreenController {
    fn clear(&self) {
        let screen : Screen = [[true; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        self.sink.send(screen).unwrap();
    }

    fn draw(&self, screen: Screen) {
        self.sink.send(screen).unwrap();
    }
}
