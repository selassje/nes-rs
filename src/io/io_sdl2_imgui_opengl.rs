mod gui_builder;
mod keyboard_shortcuts;
mod keycode_to_sdl2_scancode;

use std::collections::HashMap;
use std::default::Default;
use std::iter::FromIterator;

use super::io_internal;
use crate::common;
use crate::io;

use gl::types::*;
use io::SampleFormat;

const SAMPLING_RATE: usize = 44100;
const INITIAL_SAMPLE_BUCKET_SIZE: f32 =
    (common::DEFAULT_FPS as f32 * common::CPU_CYCLES_PER_FRAME as f32) / SAMPLING_RATE as f32;
const BUFFER_SIZE: usize = 2000;

const DISPLAY_SCALING: usize = 2;
const DISPLAY_WIDTH: usize = DISPLAY_SCALING * io::FRAME_WIDTH;
const DISPLAY_HEIGHT: usize = DISPLAY_SCALING * io::FRAME_HEIGHT;
const MENU_BAR_HEIGHT: usize = 18;

struct SampleBuffer {
    size: usize,
    sum: f32,
    bucket_size: f32,
    target_bucket_size: f32,
    buffer: [io::SampleFormat; BUFFER_SIZE],
}

impl SampleBuffer {
    fn add(&mut self, sample: io::SampleFormat) {
        if 1.0 + self.bucket_size >= self.target_bucket_size && self.size < BUFFER_SIZE {
            let bucket_diff = self.target_bucket_size - self.bucket_size;
            let bucket_diff_comp = 1.0 - bucket_diff;
            self.sum += bucket_diff * sample;
            let target_sample = self.sum / self.target_bucket_size.floor();
            self.buffer[self.size] = target_sample;
            self.size += 1;
            self.sum = bucket_diff_comp * sample;
            self.bucket_size = bucket_diff_comp;
        } else {
            self.sum += sample;
            self.bucket_size += 1.0;
        }
    }

    fn reset(&mut self, fps: u16) {
        self.size = 0;
        self.sum = 0.0;
        self.bucket_size = 0.0;
        self.target_bucket_size =
            (fps as f32 * common::CPU_CYCLES_PER_FRAME as f32) / SAMPLING_RATE as f32;
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MenuBarItem {
    LoadNesFile,
    Quit,
    PowerCycle,
    Pause,
    SpeedNormal,
    SpeedDouble,
    SpeedHalf,
    SpeedIncrease,
    SpeedDecrease,
    None,
}

pub struct IOSdl2ImGuiOpenGl {
    io_internal: io_internal::IOInternal,
    sample_buffer: SampleBuffer,
    maybe_audio_queue: Option<sdl2::audio::AudioQueue<io::SampleFormat>>,
    events: sdl2::EventPump,
    keyboard_state: HashMap<sdl2::keyboard::Scancode, bool>,
    imgui: imgui::Context,
    imgui_sdl2: imgui_sdl2::ImguiSdl2,
    window: sdl2::video::Window,
    renderer: imgui_opengl_renderer::Renderer,
    _gl_context: sdl2::video::GLContext,
    gui_builder: gui_builder::GuiBuilder,
    keyboard_shortcuts: keyboard_shortcuts::KeyboardShortcuts,
}

impl IOSdl2ImGuiOpenGl {
    pub fn new(title: &str) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let mut maybe_audio_queue = None;
        if let Ok(sdl_audio) = sdl_context.audio() {
            let desired_spec = sdl2::audio::AudioSpecDesired {
                freq: Some(SAMPLING_RATE as i32),
                channels: Some(1),
                samples: Some(BUFFER_SIZE as u16),
            };
            let audio_queue = sdl_audio.open_queue(None, &desired_spec).unwrap();
            audio_queue.resume();
            maybe_audio_queue = Some(audio_queue);
        }

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
                MENU_BAR_HEIGHT as u32 + DISPLAY_HEIGHT as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let _gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");

        gl::load_with(|s| video_subsys.gl_get_proc_address(s) as _);

        let _ = video_subsys.gl_set_swap_interval(0);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui
            .io_mut()
            .config_flags
            .set(imgui::ConfigFlags::NAV_ENABLE_KEYBOARD, true);

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

        let fonts = gui_builder::prepare_fonts(&mut imgui);

        let events = sdl_context.event_pump().unwrap();
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsys.gl_get_proc_address(s) as _
        });

        let mut emulation_texture: GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut emulation_texture);
            gl::BindTexture(gl::TEXTURE_2D, emulation_texture);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, io::FRAME_WIDTH as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        let gui_builder =
            gui_builder::GuiBuilder::new(imgui::TextureId::from(emulation_texture as usize), fonts);

        IOSdl2ImGuiOpenGl {
            io_internal: io_internal::IOInternal::new(),
            sample_buffer: SampleBuffer {
                size: 0,
                sum: 0.0,
                bucket_size: 0.0,
                buffer: [0.0; BUFFER_SIZE],
                target_bucket_size: INITIAL_SAMPLE_BUCKET_SIZE,
            },
            maybe_audio_queue: maybe_audio_queue,
            events,
            keyboard_state: HashMap::new(),
            imgui,
            window,
            imgui_sdl2,
            renderer,
            _gl_context,
            gui_builder,
            keyboard_shortcuts: Default::default(),
        }
    }

    fn update_io_state(&mut self, io_state: &mut io::IOState) {
        io_state.quit = self.is_menu_bar_item_selected(MenuBarItem::Quit);
        io_state.power_cycle = self.is_menu_bar_item_selected(MenuBarItem::PowerCycle);
        self.gui_builder.choose_nes_file = self.is_menu_bar_item_selected(MenuBarItem::LoadNesFile);
        io_state.speed = None;
        if self.is_menu_bar_item_selected(MenuBarItem::SpeedIncrease) {
            io_state.speed = Some(io::Speed::Increase);
        }
        if self.is_menu_bar_item_selected(MenuBarItem::SpeedDecrease) {
            io_state.speed = Some(io::Speed::Decrease);
        }
        if self.is_menu_bar_item_selected(MenuBarItem::SpeedNormal) {
            io_state.speed = Some(io::Speed::Normal)
        }
        if self.is_menu_bar_item_selected(MenuBarItem::SpeedDouble) {
            io_state.speed = Some(io::Speed::Double)
        }
        if self
            .gui_builder
            .is_menu_bar_item_selected(MenuBarItem::SpeedHalf)
        {
            io_state.speed = Some(io::Speed::Half)
        }

        if self.is_menu_bar_item_selected(MenuBarItem::Pause) {
            io_state.pause = !self.gui_builder.paused;
        } else {
            io_state.pause = self.gui_builder.paused;
        }
        io_state.pause |= self.gui_builder.choose_nes_file;
    }

    fn check_for_keyboard_shortcuts(
        event: &sdl2::event::Event,
        keyboard_shortcuts: &mut keyboard_shortcuts::KeyboardShortcuts,
    ) {
        use sdl2::keyboard::Scancode;
        match *event {
            sdl2::event::Event::KeyDown {
                scancode, keymod, ..
            } => {
                if let Some(scancode) = scancode {
                    if scancode == Scancode::Escape {
                        keyboard_shortcuts.update(scancode, keymod)
                    }
                }
            }
            _ => {}
        }
    }

    fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.keyboard_shortcuts.is_menu_bar_item_selected(item)
            || self.gui_builder.is_menu_bar_item_selected(item)
    }
}

impl io::IO for IOSdl2ImGuiOpenGl {
    fn present_frame(&mut self, control: io::IOControl) -> io::IOState {
        let mut io_state: io::IOState = Default::default();
        self.gui_builder.prepare_for_new_frame(control.pause);
        self.keyboard_shortcuts = Default::default();

        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        for event in self.events.poll_iter() {
            Self::check_for_keyboard_shortcuts(&event, &mut self.keyboard_shortcuts);
            match event {
                sdl2::event::Event::Window { win_event, .. } => {
                    io_state.quit = win_event == sdl2::event::WindowEvent::Close
                }
                _ => {}
            };
            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
        }

        if let Some(ref audio_queue) = self.maybe_audio_queue {
            if control.pause {
                audio_queue.pause();
            } else {
                audio_queue.resume();
                while audio_queue.size() as usize
                    > std::mem::size_of::<SampleFormat>() * self.sample_buffer.size * 10
                {
                }
                audio_queue.queue(&self.sample_buffer.buffer[..self.sample_buffer.size]);
                self.sample_buffer.reset(control.target_fps);
            }
        }
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB8 as _,
                io::FRAME_WIDTH as _,
                io::FRAME_HEIGHT as _,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.io_internal.get_pixels_slice().as_ptr() as _,
            );
        };

        self.imgui_sdl2.prepare_frame(
            self.imgui.io_mut(),
            &self.window,
            &self.events.mouse_state(),
        );

        let mut ui = self.imgui.frame();
        //ui.show_demo_window(&mut true);
        self.imgui_sdl2.prepare_render(&ui, &self.window);

        self.gui_builder
            .build(control.current_fps, control.target_fps, &mut ui);

        io_state.load_nes_file = self.gui_builder.rom_path.take();

        self.renderer.render(ui);
        self.update_io_state(&mut io_state);
        self.window.gl_swap_window();

        io_state
    }

    fn is_audio_available(&self) -> bool {
        self.maybe_audio_queue.is_some()
    }
}

impl io::VideoAccess for IOSdl2ImGuiOpenGl {
    fn set_pixel(&mut self, x: usize, y: usize, color: io::RgbColor) {
        self.io_internal.set_pixel(x, y, color);
    }
}

impl io::AudioAccess for IOSdl2ImGuiOpenGl {
    fn add_sample(&mut self, sample: io::SampleFormat) {
        self.sample_buffer.add(sample);
    }
}

impl io::KeyboardAccess for IOSdl2ImGuiOpenGl {
    fn is_key_pressed(&self, key: io::KeyCode) -> bool {
        let sdl2_scancode = keycode_to_sdl2_scancode::keycode_to_sdl2_scancode(key);
        let key_state = self.keyboard_state.get(&sdl2_scancode);
        *key_state.unwrap_or(&false)
    }
}
