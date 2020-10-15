use std::collections::HashMap;
use std::iter::FromIterator;

use crate::{
    common,
    io::{
        AudioAccess, KeyCode, KeyboardAccess, RgbColor, SampleFormat, VideoAccess, FRAME_HEIGHT,
        FRAME_WIDTH, IO,
    },
};

use pixels::Color;
use sdl2::{
    audio::AudioQueue, audio::AudioSpecDesired, keyboard::Scancode, pixels, rect::Rect,
    render::Canvas, render::TextureQuery, rwops::RWops, ttf::Sdl2TtfContext, video::Window,
    EventPump,
};

use super::{io_internal::IOInternal, IOControl, IOState};

const SAMPLE_RATE: usize = 44100;
const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / (common::FPS);
const SAMPLE_BUCKET_SIZE: f32 =
    (common::FPS * common::CPU_CYCLES_PER_FRAME) as f32 / SAMPLE_RATE as f32;
const BUFFER_SIZE: usize = SAMPLES_PER_FRAME;

const DISPLAY_SCALING: i16 = 2;

struct SampleBuffer {
    samples_ignored: usize,
    index: usize,
    total: u16,
    extra: u16,
    sum: f32,
    buffer: [SampleFormat; BUFFER_SIZE],
}

impl SampleBuffer {
    fn add(&mut self, sample: SampleFormat) {
        const SAMPLE_BUCKET_SIZE_INT: usize = SAMPLE_BUCKET_SIZE as usize;
        const FRACTION_LEFT: f32 = SAMPLE_BUCKET_SIZE - SAMPLE_BUCKET_SIZE_INT as f32;
        const FRACTION_RIGHT: f32 = 1.0 - FRACTION_LEFT;
        self.total += 1;
        if self.index < BUFFER_SIZE {
            if self.samples_ignored == SAMPLE_BUCKET_SIZE_INT && self.index % 2 == 0 {
                self.buffer[self.index] =
                    (self.sum + FRACTION_LEFT * sample) / (SAMPLE_BUCKET_SIZE_INT + 1) as f32;
                self.index = self.index + 1;
                self.sum = FRACTION_RIGHT * sample;
            } else if self.samples_ignored == SAMPLE_BUCKET_SIZE_INT - 1 && self.index % 2 == 1 {
                self.buffer[self.index] = (self.sum + sample) / (SAMPLE_BUCKET_SIZE_INT + 1) as f32;
                self.index += 1;
                self.sum = 0.0;
                self.samples_ignored += 1;
            } else {
                self.sum += sample;
            }
        } else {
            self.extra += 1;
        }
        self.samples_ignored = (self.samples_ignored + 1) % (SAMPLE_BUCKET_SIZE_INT + 1);
    }

    fn reset(&mut self) {
        self.index = 0;
        self.total = 0;
        self.extra = 0;
        self.samples_ignored = 0;
        self.sum = 0.0;
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
            samples: Some(SAMPLES_PER_FRAME as u16),
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

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();
        let events = sdl_context.event_pump().unwrap();

        IOSdl2 {
            io_internal: IOInternal::new(),
            sample_buffer: SampleBuffer {
                index: 0,
                total: 0,
                extra: 0,
                samples_ignored: 0,
                sum: 0.0,
                buffer: [0.0; BUFFER_SIZE],
            },
            audio_queue,
            canvas,
            events,
            keyboard_state: HashMap::new(),
            ttf_context: sdl2::ttf::init().unwrap(),
        }
    }

    fn draw_fps(&mut self, fps: u8) {
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
        for (x, col) in self.io_internal.get_pixel_iter().enumerate() {
            for (y, color) in col.iter().enumerate() {
                let x = (x * (DISPLAY_SCALING as usize)) as i32;
                let y = (y * (DISPLAY_SCALING as usize)) as i32;
                let rect = Rect::new(x, y, DISPLAY_SCALING as u32, DISPLAY_SCALING as u32);
                let _ = self.canvas.set_draw_color(*color);
                let _ = self.canvas.draw_rect(rect);
            }
        }
        self.draw_fps(control.fps);
        self.canvas.present();

        self.audio_queue
            .queue(&self.sample_buffer.buffer[..self.sample_buffer.index]);

        // println!(
        //     "Total samples {} current {} extra {} samples_per_frame {} interopolation {}",
        //     self.sample_buffer.total,
        //     self.sample_buffer.index,
        //     self.sample_buffer.extra,
        //     SAMPLES_PER_FRAME,
        //     SAMPLE_BUCKET_SIZE,
        // );
        self.sample_buffer.reset();

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
