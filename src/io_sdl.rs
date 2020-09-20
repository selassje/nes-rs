extern crate sdl2;

use crate::{
    screen::{Screen, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    NesSettings,
};
use pixels::PixelFormatEnum;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

use lazy_static::lazy_static;
use sdl2::keyboard::Scancode;
use sdl2::pixels;
use sdl2::rect::Rect;

use circular_queue::CircularQueue;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::Mutex;
use std::time::{Duration, Instant};
pub type SampleFormat = u8;

const FPS: u128 = 200;
const FRAME_DURATION: u128 = Duration::from_secs(1).as_nanos() / FPS;

pub static mut SCREEN: Screen = [[(255, 255, 255); DISPLAY_HEIGHT]; DISPLAY_WIDTH];
pub const BUFFER_SIZE: usize = 2048;
pub const SAMPLE_RATE: usize = 41100;
const ENQEUES_PER_SECOND: u128 = (SAMPLE_RATE / BUFFER_SIZE) as u128;
const SOUND_FRAME_DURATION: u128 = Duration::from_secs(1).as_nanos() / ENQEUES_PER_SECOND;

pub struct SampleBuffer {
    pub index: usize,
    pub buffer: [SampleFormat; BUFFER_SIZE],
}

lazy_static! {
    pub static ref KEYBOARD: Mutex<HashMap<Scancode, bool>> = Mutex::new(HashMap::new());
    pub static ref SAMPLES_QUEUE: Mutex<CircularQueue<SampleFormat>> =
        Mutex::new(CircularQueue::with_capacity(BUFFER_SIZE));
    pub static ref SAMPLE_BUFFER: Mutex<SampleBuffer> = Mutex::new(SampleBuffer {
        index: 0,
        buffer: [0; BUFFER_SIZE]
    });
}

pub fn get_key_status(key: Scancode) -> bool {
    if let Some(status) = KEYBOARD.lock().unwrap().get(&key) {
        *status
    } else {
        false
    }
}

pub struct IOSdl {
    title: String,
}

impl IOSdl {
    pub fn new(title: String) -> IOSdl {
        IOSdl { title }
    }
    pub fn run(&self, settings: NesSettings) {
        const DISPLAY_SCALING: i16 = 2;
        let start_time = Instant::now();

        let sdl_context = sdl2::init().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(BUFFER_SIZE as u16),
        };

        let apu_audio: AudioQueue<SampleFormat> =
            sdl_audio.open_queue(None, &desired_spec).unwrap();

        if settings.enable_sound {
            apu_audio.resume();
        }
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                &format!("nes-rs: {}", self.title),
                (DISPLAY_WIDTH as u32) * (DISPLAY_SCALING as u32),
                (DISPLAY_HEIGHT as u32) * (DISPLAY_SCALING as u32),
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            //.present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();

        let mut events = sdl_context.event_pump().unwrap();
        let mut frame_timer = Instant::now();
        let mut sound_timer = Instant::now();
        while settings.duration == None || start_time.elapsed() < settings.duration.unwrap() {
            *KEYBOARD.lock().unwrap() = HashMap::from_iter(events.keyboard_state().scancodes());
            events.pump_events();
            canvas.clear();

            unsafe {
                if frame_timer.elapsed().as_nanos() > FRAME_DURATION {
                    for (x, col) in SCREEN.iter().enumerate() {
                        for (y, color) in col.iter().enumerate() {
                            let x = (x * (DISPLAY_SCALING as usize)) as i32;
                            let y = (y * (DISPLAY_SCALING as usize)) as i32;
                            let rect =
                                Rect::new(x, y, DISPLAY_SCALING as u32, DISPLAY_SCALING as u32);
                            let _ = canvas.set_draw_color(*color);
                            let _ = canvas.draw_rect(rect);
                        }
                    }
                    canvas.present();
                    frame_timer = Instant::now();
                }

                let mut buffer = SAMPLE_BUFFER.lock().unwrap();
                if sound_timer.elapsed().as_nanos() > SOUND_FRAME_DURATION {
                    if buffer.index == BUFFER_SIZE {
                        apu_audio.queue(&buffer.buffer[..]);
                        buffer.index = 0;
                        sound_timer = Instant::now();
                    }
                }
            };
        }
    }

    pub fn dump_frame(path: &str) {
        let mut bitmap = sdl2::surface::Surface::new(
            DISPLAY_WIDTH as u32,
            DISPLAY_HEIGHT as u32,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        unsafe {
            for (x, col) in SCREEN.iter().enumerate() {
                for (y, color) in col.iter().enumerate() {
                    let (r, g, b) = *color;
                    let pixel_color = pixels::Color::RGB(r, g, b);
                    let _ = bitmap.fill_rect(Rect::new(x as i32, y as i32, 1, 1), pixel_color);
                }
            }
        }
        let _ = bitmap.save_bmp(path);
    }
}
