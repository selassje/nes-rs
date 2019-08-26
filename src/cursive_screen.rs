use cursive::CbSink;
use crate::screen::{ScreenController, Screen};
use crate::display::ScreenView;

pub struct CursiveScreenController {
    sink : CbSink,
}

impl CursiveScreenController {

    pub fn new(sink : CbSink) -> CursiveScreenController
    {
        CursiveScreenController {
            sink: sink,
        }
    }
}

impl ScreenController for CursiveScreenController {
    fn clear(&self) {
        self.sink.send(Box::new(|s| {
             s.call_on_id("display", |view : &mut ScreenView |{
                 for (_, col) in view.screen.iter_mut().enumerate() {
                     for (_, b) in col.iter_mut().enumerate() {
                        *b = false;
                     }
                 }
            });}
        )).unwrap();
    }

    fn draw(&self, screen: Screen) {
          self.sink.send(Box::new(move |s| {
             s.call_on_id("display", |view : &mut ScreenView |{
                 for (x, col) in view.screen.iter_mut().enumerate() {
                     for (y, b) in col.iter_mut().enumerate() {
                        *b = screen[x][y];
                     }
                 }
            });}
        )).unwrap();
    }
}

