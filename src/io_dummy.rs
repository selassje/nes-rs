use sdl2::{
    pixels::{self, PixelFormatEnum},
    rect::Rect,
};

use crate::io::{
    AudioAccess, Frame, KeyboardAccess, RgbColor, VideoAccess, FRAME_HEIGHT, FRAME_WIDTH, IO,
};

pub struct IODummy {
    frame: Frame,
}

impl IODummy {
    pub fn new(_: &str) -> Self {
        IODummy {
            frame: [[(255, 255, 255); FRAME_HEIGHT]; FRAME_WIDTH],
        }
    }
}

impl IO for IODummy {
    fn present_frame(&mut self) {}

    fn dump_frame(&self, path: &str) {
        let mut bitmap = sdl2::surface::Surface::new(
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        for (x, col) in self.frame.iter().enumerate() {
            for (y, color) in col.iter().enumerate() {
                let (r, g, b) = *color;
                let pixel_color = pixels::Color::RGB(r, g, b);
                let _ = bitmap.fill_rect(Rect::new(x as i32, y as i32, 1, 1), pixel_color);
            }
        }
        let _ = bitmap.save_bmp(path);
    }
}

impl AudioAccess for IODummy {
    fn add_sample(&mut self, _: crate::io::SampleFormat) {}
}

impl VideoAccess for IODummy {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.frame[x][y] = color;
    }
}

impl KeyboardAccess for IODummy {
    fn is_key_pressed(&self, _: crate::io::KeyCode) -> bool {
        false
    }
}
