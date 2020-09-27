use crate::io::io_internal::IOInternal;
use crate::io::{AudioAccess, DumpFrame, KeyboardAccess, RgbColor, VideoAccess, IO};

pub struct IOTest {
    io_internal: IOInternal,
}

impl IOTest {
    pub fn new(_: &str) -> Self {
        IOTest {
            io_internal: IOInternal::new(),
        }
    }
}

impl IO for IOTest {
    fn present_frame(&mut self) {}
}

impl DumpFrame for IOTest {
    fn dump_frame(&self, path: &str) {
        self.io_internal.dump_frame(path);
    }
}

impl AudioAccess for IOTest {
    fn add_sample(&mut self, _: crate::io::SampleFormat) {}
}

impl VideoAccess for IOTest {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.io_internal.set_pixel(x, y, color);
    }
}

impl KeyboardAccess for IOTest {
    fn is_key_pressed(&self, _: crate::io::KeyCode) -> bool {
        false
    }
}
