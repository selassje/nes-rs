use std::collections::HashMap;
use std::iter::FromIterator;

use crate::{
    common,
    io::{
        AudioAccess, IOControl, IOState, KeyCode, KeyboardAccess, RgbColor, SampleFormat,
        VideoAccess, FRAME_HEIGHT, FRAME_WIDTH, IO, PIXEL_SIZE,
    },
};

use sdl2::{
    audio::AudioQueue, audio::AudioSpecDesired, keyboard::Scancode, pixels::Color,
    pixels::PixelFormatEnum, rect::Rect, render::Canvas, render::TextureQuery, rwops::RWops,
    ttf::Sdl2TtfContext, video::Window, EventPump,
};

use super::io_internal::IOInternal;

const SAMPLE_RATE: usize = 44100;
const SAMPLE_RATE_ADJ: usize = (SAMPLE_RATE as f32 * 1.0101) as usize;
const INITIAL_SAMPLE_BUCKET_SIZE: f32 =
    (common::FPS * common::CPU_CYCLES_PER_FRAME) as f32 / SAMPLE_RATE_ADJ as f32;
const BUFFER_SIZE: usize = 10000;

const DISPLAY_SCALING: i16 = 2;

struct SampleBuffer {
    index: usize,
    sum: f32,
    bucket_size: f32,
    target_bucket_size: f32,
    buffer: [SampleFormat; BUFFER_SIZE],
}

impl SampleBuffer {
    fn add(&mut self, sample: SampleFormat) {
        if 1.0 + self.bucket_size >= self.target_bucket_size {
            let bucket_diff = self.target_bucket_size - self.bucket_size;
            assert!(bucket_diff >= 0.0 && bucket_diff <= 1.0);
            let bucket_diff_comp = 1.0 - bucket_diff;
            self.sum += bucket_diff * sample;
            let target_sample = self.sum / self.target_bucket_size.floor();
            self.buffer[self.index] = target_sample;
            self.index += 1;
            self.sum = bucket_diff_comp * sample;
            self.bucket_size = bucket_diff_comp;
        } else {
            self.sum += sample;
            self.bucket_size += 1.0;
        }
    }

    fn reset(&mut self, fps: u16) {
        self.index = 0;
        self.sum = 0.0;
        self.bucket_size = 0.0;
        self.target_bucket_size =
            (fps as usize * common::CPU_CYCLES_PER_FRAME) as f32 / SAMPLE_RATE_ADJ as f32;
    }
}

pub struct IOSdl2 {
    io_internal: IOInternal,
    sample_buffer: SampleBuffer,
    audio_queue: AudioQueue<SampleFormat>,
    events: EventPump,
    canvas: Canvas<Window>,
    keyboard_state: HashMap<Scancode, bool>,
    ttf_context: Sdl2TtfContext,
}

fn keycode_to_sdl2_scancode(key: KeyCode) -> Scancode {
    match key {
        KeyCode::Q => Scancode::Q,
        KeyCode::E => Scancode::E,
        KeyCode::C => Scancode::C,
        KeyCode::Space => Scancode::Space,
        KeyCode::W => Scancode::W,
        KeyCode::S => Scancode::S,
        KeyCode::A => Scancode::A,
        KeyCode::D => Scancode::D,
        KeyCode::Kp4 => Scancode::Kp4,
        KeyCode::Kp5 => Scancode::Kp5,
        KeyCode::Kp6 => Scancode::Kp6,
        KeyCode::KpPlus => Scancode::KpPlus,
        KeyCode::Up => Scancode::Up,
        KeyCode::Down => Scancode::Down,
        KeyCode::Left => Scancode::Left,
        KeyCode::Right => Scancode::Right,
    }
}

impl IOSdl2 {
    pub fn new(title: &str) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(BUFFER_SIZE as u16),
        };

        let audio_queue: AudioQueue<SampleFormat> =
            sdl_audio.open_queue(None, &desired_spec).unwrap();

        audio_queue.resume();

        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                &format!("nes-rs: {}", title),
                (FRAME_WIDTH as u32) * (DISPLAY_SCALING as u32),
                (FRAME_HEIGHT as u32) * (DISPLAY_SCALING as u32),
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.present();
        let events = sdl_context.event_pump().unwrap();

        IOSdl2 {
            io_internal: IOInternal::new(),
            sample_buffer: SampleBuffer {
                index: 0,
                sum: 0.0,
                bucket_size: 0.0,
                buffer: [0.0; BUFFER_SIZE],
                target_bucket_size: INITIAL_SAMPLE_BUCKET_SIZE,
            },
            audio_queue,
            canvas,
            events,
            keyboard_state: HashMap::new(),
            ttf_context: sdl2::ttf::init().unwrap(),
        }
    }

    fn draw_fps(&mut self, fps: u16) {
        let font_data = include_bytes!("../../res/OpenSans-Regular.ttf");
        let r = RWops::from_bytes(font_data).unwrap();
        let mut font = self.ttf_context.load_font_from_rwops(r, 14).unwrap();
        font.set_style(sdl2::ttf::FontStyle::BOLD);
        let texture_creator = self.canvas.texture_creator();
        let surface = font
            .render(&format!("FPS {}", fps))
            .blended(Color::RGBA(255, 255, 255, 255))
            .map_err(|e| e.to_string())
            .unwrap();

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        let x = DISPLAY_SCALING * FRAME_WIDTH as i16 - width as i16;
        let target = Rect::new(x as i32, 0, width, height as u32);
        let _ = self.canvas.copy(&texture, None, Some(target));
    }
}

impl IO for IOSdl2 {
    fn present_frame(&mut self, control: IOControl) -> IOState {
        let mut io_state: IOState = Default::default();
        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        io_state.quit = *self.keyboard_state.get(&Scancode::Escape).unwrap();
        self.events.pump_events();
        let texture_creator = self.canvas.texture_creator();
        let mut streaming_texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                FRAME_WIDTH as u32,
                FRAME_HEIGHT as u32,
            )
            .unwrap();

        let _ = streaming_texture.update(
            Rect::new(0, 0, FRAME_WIDTH as u32, FRAME_HEIGHT as u32),
            self.io_internal.get_pixels_slice(),
            FRAME_WIDTH * PIXEL_SIZE,
        );

        let _ = self.canvas.copy(&streaming_texture, None, None);
        self.draw_fps(control.fps);
        self.canvas.present();

        self.audio_queue
            .queue(&self.sample_buffer.buffer[..self.sample_buffer.index]);

        self.sample_buffer.reset(control.fps);

        io_state
    }
}

impl VideoAccess for IOSdl2 {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.io_internal.set_pixel(x, y, color);
    }
}

impl AudioAccess for IOSdl2 {
    fn add_sample(&mut self, sample: SampleFormat) {
        self.sample_buffer.add(sample);
    }
}

impl KeyboardAccess for IOSdl2 {
    fn is_key_pressed(&self, key: crate::io::KeyCode) -> bool {
        let sdl2_scancode = keycode_to_sdl2_scancode(key);
        let key_state = self.keyboard_state.get(&sdl2_scancode);
        *key_state.unwrap_or(&false)
    }
}
