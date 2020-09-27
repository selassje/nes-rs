use std::slice::Iter;

use sdl2::{
    pixels::{self, PixelFormatEnum},
    rect::Rect,
};

use super::{DumpFrame, RgbColor, VideoAccess, FRAME_HEIGHT, FRAME_WIDTH};

type Frame = [[RgbColor; FRAME_HEIGHT]; FRAME_WIDTH];
pub(super) struct IOInternal {
    frame: Frame,
}

impl IOInternal {
    pub fn new() -> Self {
        IOInternal {
            frame: [[(255, 255, 255); FRAME_HEIGHT]; FRAME_WIDTH],
        }
    }

    pub fn get_pixel_iter(&self) -> Iter<[RgbColor; FRAME_HEIGHT]> {
        self.frame.iter()
    }
}

impl DumpFrame for IOInternal {
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

impl VideoAccess for IOInternal {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.frame[x][y] = color;
    }
}
