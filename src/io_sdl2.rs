use std::collections::HashMap;
use std::iter::FromIterator;

use crate::{
    common,
    io::{
        AudioAccess, Frame, KeyCode, KeyboardAccess, RgbColor, SampleFormat, VideoAccess,
        FRAME_HEIGHT, FRAME_WIDTH, IO,
    },
};

use sdl2::{
    audio::AudioQueue, audio::AudioSpecDesired, keyboard::Scancode, pixels, rect::Rect,
    render::Canvas, video::Window, EventPump,
};

const SAMPLE_RATE: usize = 41100;
const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / common::FPS;
const SAMPLE_INTERPOLATION: usize = common::CPU_CYCLES_PER_FRAME / SAMPLES_PER_FRAME;
const DISPLAY_SCALING: i16 = 2;

struct SampleBuffer {
    samples_ignored: usize,
    index: usize,
    buffer: [SampleFormat; SAMPLES_PER_FRAME],
}

impl SampleBuffer {
    fn add(&mut self, sample: SampleFormat) {
        if self.samples_ignored == 0 {
            self.buffer[self.index] = sample;
            self.index = (self.index + 1) % SAMPLES_PER_FRAME;
            if self.index == SAMPLES_PER_FRAME {
                self.index = 0;
            }
        }
        self.samples_ignored = (self.samples_ignored + 1) % SAMPLE_INTERPOLATION;
    }
}

pub struct IOSdl2 {
    frame: Frame,
    sample_buffer: SampleBuffer,
    audio_queue: AudioQueue<SampleFormat>,
    events: EventPump,
    canvas: Canvas<Window>,
    keyboard_state: HashMap<Scancode, bool>,
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
            //.present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();

        let events = sdl_context.event_pump().unwrap();

        IOSdl2 {
            frame: [[(255, 255, 255); FRAME_HEIGHT]; FRAME_WIDTH],
            sample_buffer: SampleBuffer {
                index: 0,
                samples_ignored: 0,
                buffer: [0; SAMPLES_PER_FRAME],
            },
            audio_queue,
            canvas,
            events,
            keyboard_state: HashMap::new(),
        }
    }
}

impl IO for IOSdl2 {
    fn present_frame(&mut self) {
        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        self.events.pump_events();
        for (x, col) in self.frame.iter().enumerate() {
            for (y, color) in col.iter().enumerate() {
                let x = (x * (DISPLAY_SCALING as usize)) as i32;
                let y = (y * (DISPLAY_SCALING as usize)) as i32;
                let rect = Rect::new(x, y, DISPLAY_SCALING as u32, DISPLAY_SCALING as u32);
                let _ = self.canvas.set_draw_color(*color);
                let _ = self.canvas.draw_rect(rect);
            }
        }
        self.canvas.present();
        self.audio_queue.queue(&self.sample_buffer.buffer[..]);
    }

    fn dump_frame(&self, _: &str) {}
}

impl VideoAccess for IOSdl2 {
    fn set_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        self.frame[x][y] = color;
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
