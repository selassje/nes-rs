use std::collections::HashMap;
use std::iter::FromIterator;

extern crate gl;

use crate::{
    common,
    io::{
        AudioAccess, IOControl, IOState, KeyCode, KeyboardAccess, RgbColor, SampleFormat,
        VideoAccess, FRAME_HEIGHT, FRAME_WIDTH, IO,
    },
};

use gl::types::*;

use super::io_internal::IOInternal;

use imgui::{im_str, Context, Image, MenuItem, TextureId, Ui};
use imgui_sdl2::ImguiSdl2;
use sdl2::{
    audio::AudioQueue, audio::AudioSpecDesired, keyboard::Scancode, ttf::Sdl2TtfContext,
    video::GLContext, video::Window, EventPump,
};

const SAMPLE_RATE: usize = 44100;
const SAMPLE_RATE_ADJ: usize = (SAMPLE_RATE as f32 * 1.0000) as usize;
const INITIAL_SAMPLE_BUCKET_SIZE: f32 =
    (common::FPS * common::CPU_CYCLES_PER_FRAME) as f32 / SAMPLE_RATE_ADJ as f32;
const BUFFER_SIZE: usize = 10000;

const DISPLAY_SCALING: usize = 2;
const DISPLAY_WIDTH: usize = DISPLAY_SCALING * FRAME_WIDTH;
const DISPLAY_HEIGHT: usize = DISPLAY_SCALING * FRAME_HEIGHT;
const MENU_BAR_HEIGHT: usize = 20;

struct SampleBuffer {
    index: usize,
    sum: f32,
    bucket_size: f32,
    target_bucket_size: f32,
    buffer: [SampleFormat; BUFFER_SIZE],
}

impl SampleBuffer {
    fn add(&mut self, sample: SampleFormat) {
        if 1.0 + self.bucket_size >= self.target_bucket_size && self.index < BUFFER_SIZE {
            let bucket_diff = self.target_bucket_size - self.bucket_size;
            //assert!(bucket_diff >= 0.0 && bucket_diff <= 1.0);
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
    keyboard_state: HashMap<Scancode, bool>,
    ttf_context: Sdl2TtfContext,
    imgui: Context,
    imgui_sdl2: ImguiSdl2,
    window: Window,
    renderer: imgui_opengl_renderer::Renderer,
    _gl_context: GLContext,
    emulation_texture: GLuint,
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
        {
            let gl_attr = video_subsys.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 0);
        }

        let window = video_subsys
            .window(
                title,
                DISPLAY_WIDTH as _,
                (DISPLAY_HEIGHT + MENU_BAR_HEIGHT) as _,
            )
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let _gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");

        gl::load_with(|s| video_subsys.gl_get_proc_address(s) as _);

        let _ = video_subsys.gl_set_swap_interval(0);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

        let events = sdl_context.event_pump().unwrap();
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsys.gl_get_proc_address(s) as _
        });

        let mut emulation_texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut emulation_texture);
            gl::BindTexture(gl::TEXTURE_2D, emulation_texture);
            gl::PixelStorei(gl::PACK_ROW_LENGTH, FRAME_WIDTH as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
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
            events,
            keyboard_state: HashMap::new(),
            ttf_context: sdl2::ttf::init().unwrap(),
            imgui,
            window,
            imgui_sdl2,
            renderer,
            _gl_context,
            emulation_texture,
        }
    }

    fn prepare_menu_bar(ui: &mut Ui) {
        let styles = ui.push_style_vars(&[
            imgui::StyleVar::WindowRounding(0.0),
            imgui::StyleVar::WindowBorderSize(0.0),
            imgui::StyleVar::WindowPadding([0.0, 0.0]),
        ]);
        if let Some(menu_bar_token) = ui.begin_main_menu_bar() {
            if let Some(menu_token) = ui.begin_menu(im_str!("File"), true) {
                MenuItem::new(im_str!("Load Rom"))
                    .selected(false)
                    .enabled(true)
                    .build(ui);
                menu_token.end(ui);
            }
            menu_bar_token.end(ui);
        } else {
            panic!("Could not render main_menu bar");
        }
        styles.pop(ui);
    }

    fn prepare_emulation_texture(emulation_texture: TextureId, ui: &mut Ui) {
        let styles = ui.push_style_vars(&[
            imgui::StyleVar::WindowRounding(0.0),
            imgui::StyleVar::WindowBorderSize(0.0),
            imgui::StyleVar::WindowPadding([0.0, 0.0]),
        ]);

        let window = imgui::Window::new(im_str!("emulation"))
            .scrollable(false)
            .no_decoration()
            .position([0.0, MENU_BAR_HEIGHT as _], imgui::Condition::Always)
            .size(
                [DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _],
                imgui::Condition::Always,
            );
        if let Some(window_token) = window.begin(ui) {
            Image::new(emulation_texture, [DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _]).build(ui);
            window_token.end(ui);
        }
        styles.pop(&ui);
    }

    fn draw_fps(&mut self, fps: u16) {
        //   V  let font_data = include_bytes!("../../res/OpenSans-Regular.ttf");
        //     let r = RWops::from_bytes(font_data).unwrap();
        //     let mut font = self.ttf_context.load_font_from_rwops(r, 14).unwrap();
        //     font.set_style(sdl2::ttf::FontStyle::BOLD);
        //     let texture_creator = self.canvas.texture_creator();
        //     let surface = font
        //         .render(&format!("FPS {}", fps))
        //         .blended(Color::RGBA(255, 255, 255, 255))
        //         .map_err(|e| e.to_string())
        //         .unwrap();

        //     let texture = texture_creator
        //         .create_texture_from_surface(&surface)
        //         .map_err(|e| e.to_string())
        //         .unwrap();
        //     let TextureQuery { width, height, .. } = texture.query();
        //     let x = DISPLAY_SCALING * FRAME_WIDTH as i16 - width as i16;
        //     let target = Rect::new(x as i32, 0, width, height as u32);
        //     // let _ = self.canvas.copy(&texture, None, Some(target));
    }
}

impl IO for IOSdl2 {
    fn present_frame(&mut self, control: IOControl) -> IOState {
        let mut io_state: IOState = Default::default();
        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        io_state.quit = *self.keyboard_state.get(&Scancode::Escape).unwrap();
        io_state.reset = *self.keyboard_state.get(&Scancode::R).unwrap();

        for event in self.events.poll_iter() {
            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
            if self.imgui_sdl2.ignore_event(&event) {
                continue;
            }
        }

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB8 as _,
                FRAME_WIDTH as _,
                FRAME_HEIGHT as _,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.io_internal.get_pixels_slice().as_ptr() as _,
            );
        };

        self.draw_fps(control.fps);

        self.imgui_sdl2.prepare_frame(
            self.imgui.io_mut(),
            &self.window,
            &self.events.mouse_state(),
        );

        let mut ui = self.imgui.frame();

        Self::prepare_menu_bar(&mut ui);
        Self::prepare_emulation_texture(TextureId::from(self.emulation_texture as usize), &mut ui);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.imgui_sdl2.prepare_render(&ui, &self.window);
        self.renderer.render(ui);
        self.window.gl_swap_window();

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
