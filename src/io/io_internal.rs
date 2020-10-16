use sdl2::{
    pixels::{self, PixelFormatEnum},
    rect::Rect,
};

use super::{RgbColor, VideoAccess, FRAME_HEIGHT, FRAME_WIDTH, PIXEL_SIZE};

const FRAME_SIZE: usize = FRAME_HEIGHT * FRAME_WIDTH * PIXEL_SIZE;
type Frame2 = [u8; FRAME_SIZE];
pub(super) struct IOInternal {
    frame: Frame2,
}

impl IOInternal {
    pub fn new() -> Self {
        IOInternal {
            frame: [0; FRAME_SIZE],
        }
    }

    pub fn get_pixels_slice(&self) -> &[u8] {
        &self.frame
    }
    pub(super) fn dump_frame(&self, path: &str) {
        let mut bitmap = sdl2::surface::Surface::new(
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        for x in 0..FRAME_WIDTH {
            for y in 0..FRAME_HEIGHT {
                let index = y * PIXEL_SIZE * FRAME_WIDTH + x * PIXEL_SIZE;
                let pixel_color = pixels::Color::RGB(
                    self.frame[index],
                    self.frame[index + 1],
                    self.frame[index + 2],
                );
                let _ = bitmap.fill_rect(Rect::new(x as i32, y as i32, 1, 1), pixel_color);
            }
        }
        let _ = bitmap.save_bmp(path);
    }
}

impl VideoAccess for IOInternal {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        let (r, g, b) = color;
        let index = y * PIXEL_SIZE * FRAME_WIDTH + x * PIXEL_SIZE;
        self.frame[index] = r;
        self.frame[index + 1] = g;
        self.frame[index + 2] = b;
    }
}
