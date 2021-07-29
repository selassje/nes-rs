mod audio_sample_buffer;
mod gui_builder;
mod keyboard_shortcuts;

use std::default::Default;
use std::iter::FromIterator;
use std::{borrow::BorrowMut, collections::HashMap};

use super::{io_internal, VideoSizeControl};
use crate::{controllers, io};

use gl::types::*;
use sdl2::image::ImageRWops;

const MENU_BAR_HEIGHT: u32 = 18;
const MIN_WINDOW_WIDTH: u32 = 360;

type Size = [f32; 2];

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
    AudioEnabled,
    VolumeIncrease,
    VolumeDecrease,
    VideoSizeNormal,
    VideoSizeDouble,
    VideoSizeTriple,
    VideoSizeQuadrupal,
    VideoSizeFullScreen,
    ControllersSetup,
    None,
}

pub struct IOSdl2ImGuiOpenGl {
    io_internal: io_internal::IOInternal,
    sample_buffer: audio_sample_buffer::AudioSampleBuffer,
    maybe_audio_queue: Option<sdl2::audio::AudioQueue<io::AudioSampleFormat>>,
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
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let mut maybe_audio_queue = None;
        if let Ok(sdl_audio) = sdl_context.audio() {
            let desired_spec = sdl2::audio::AudioSpecDesired {
                freq: Some(audio_sample_buffer::SAMPLING_RATE as i32),
                channels: Some(1),
                samples: Some(audio_sample_buffer::BUFFER_SIZE as u16),
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

        let [video_width, video_height]: [u32; 2] = io::VideoSizeControl::Double.into();
        let mut window = video_subsys
            .window("NES-RS", video_width, MENU_BAR_HEIGHT as u32 + video_height)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        window.set_icon(
            sdl2::rwops::RWops::from_bytes(include_bytes!("../../res/icon/Nintendo-gray-icon.png"))
                .unwrap()
                .load_png()
                .unwrap(),
        );

        let _gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");

        gl::load_with(|s| video_subsys.gl_get_proc_address(s) as _);

        let _ = video_subsys.gl_set_swap_interval(0);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

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
            sample_buffer: audio_sample_buffer::AudioSampleBuffer::new(),
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
        let io_common = self.gui_builder.get_io_common();

        io_state.common.choose_nes_file = self.is_menu_bar_item_selected(MenuBarItem::LoadNesFile);
        io_state.common.volume = io_common.volume;

        io_state.quit |= self.is_menu_bar_item_selected(MenuBarItem::Quit);
        io_state.power_cycle = self.is_menu_bar_item_selected(MenuBarItem::PowerCycle);
        io_state.load_nes_file = self.gui_builder.get_rom_path();

        io_state.speed = None;
        {
            let mut set_speed_selection = |item: MenuBarItem, speed: io::Speed| {
                if self.is_menu_bar_item_selected(item) {
                    io_state.speed = Some(speed)
                }
            };
            set_speed_selection(MenuBarItem::SpeedIncrease, io::Speed::Increase);
            set_speed_selection(MenuBarItem::SpeedDecrease, io::Speed::Decrease);
            set_speed_selection(MenuBarItem::SpeedNormal, io::Speed::Normal);
            set_speed_selection(MenuBarItem::SpeedHalf, io::Speed::Half);
            set_speed_selection(MenuBarItem::SpeedDouble, io::Speed::Double);
        }
        io_state.audio_volume_control = None;
        {
            let mut set_volume_control =
                |item: MenuBarItem, volume_control: io::AudioVolumeControl| {
                    if self.is_menu_bar_item_selected(item) {
                        io_state.audio_volume_control = Some(volume_control)
                    }
                };
            set_volume_control(
                MenuBarItem::VolumeIncrease,
                io::AudioVolumeControl::Increase,
            );
            set_volume_control(
                MenuBarItem::VolumeDecrease,
                io::AudioVolumeControl::Decrease,
            );
        }
        io_state.common.video_size = io_common.video_size;
        {
            let mut set_video_size_selection =
                |item: MenuBarItem, video_size: io::VideoSizeControl| {
                    if self.is_menu_bar_item_selected(item) {
                        io_state.common.video_size = video_size;
                    }
                };
            set_video_size_selection(MenuBarItem::VideoSizeDouble, io::VideoSizeControl::Double);
            set_video_size_selection(MenuBarItem::VideoSizeTriple, io::VideoSizeControl::Triple);
            set_video_size_selection(
                MenuBarItem::VideoSizeQuadrupal,
                io::VideoSizeControl::Quadrupal,
            );
            set_video_size_selection(
                MenuBarItem::VideoSizeFullScreen,
                io::VideoSizeControl::FullScreen,
            );
        }
        let toggle = |item: MenuBarItem, value: bool| {
            if self.is_menu_bar_item_selected(item) {
                !value
            } else {
                value
            }
        };

        io_state.common.pause = toggle(MenuBarItem::Pause, io_common.pause);
        io_state.common.audio_enabled = toggle(MenuBarItem::AudioEnabled, io_common.audio_enabled);
        io_state.common.controllers_setup =
            toggle(MenuBarItem::ControllersSetup, io_common.controllers_setup);
        io_state.common.pause |= io_state.common.choose_nes_file;
        io_state.common.controller_configs = io_common.controller_configs;
    }

    fn check_for_keyboard_shortcuts(
        event: &sdl2::event::Event,
        keyboard_shortcuts: &mut keyboard_shortcuts::KeyboardShortcuts,
    ) {
        match *event {
            sdl2::event::Event::KeyDown {
                scancode, keymod, ..
            } => {
                if let Some(scancode) = scancode {
                    keyboard_shortcuts.update(scancode, keymod)
                }
            }
            _ => {}
        }
    }

    fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.keyboard_shortcuts.is_menu_bar_item_selected(item)
            || self.gui_builder.is_menu_bar_item_selected(item)
    }

    fn set_window_tile(&mut self, control: &io::IOControl) {
        if let Some(ref title) = control.title {
            self.window.borrow_mut().set_title(title).unwrap();
        }
    }

    fn set_window_size_and_get_video_size(&mut self, control: &io::IOControl) -> Size {
        if control.common.video_size != VideoSizeControl::FullScreen {
            let [video_width, video_height]: [u32; 2] = control.common.video_size.into();
            self.window
                .borrow_mut()
                .set_fullscreen(sdl2::video::FullscreenType::Off)
                .unwrap();
            self.window
                .borrow_mut()
                .set_size(
                    std::cmp::max(video_width, MIN_WINDOW_WIDTH),
                    video_height + MENU_BAR_HEIGHT as u32,
                )
                .unwrap();
            [video_width as f32, video_height as f32]
        } else {
            self.window
                .borrow_mut()
                .set_fullscreen(sdl2::video::FullscreenType::Desktop)
                .unwrap();
            let display_mode = self.window.display_mode().unwrap();
            [display_mode.w as f32, display_mode.h as f32]
        }
    }
}

impl io::IO for IOSdl2ImGuiOpenGl {
    fn present_frame(&mut self, control: io::IOControl) -> io::IOState {
        let mut io_state: io::IOState = Default::default();
        self.set_window_tile(&control);
        let video_size = self.set_window_size_and_get_video_size(&control);
        self.gui_builder
            .prepare_for_new_frame(control.clone(), video_size);
        self.keyboard_shortcuts = Default::default();

        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        for event in self.events.poll_iter() {
            if self.gui_builder.is_key_selection_pending() {
                self.gui_builder.try_get_key_selection(&event);
            } else {
                Self::check_for_keyboard_shortcuts(&event, &mut self.keyboard_shortcuts);
            }
            match event {
                sdl2::event::Event::Window { win_event, .. } => {
                    io_state.quit = win_event == sdl2::event::WindowEvent::Close
                }
                _ => {}
            };
            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
        }

        if let Some(ref audio_queue) = self.maybe_audio_queue {
            if control.common.pause {
                audio_queue.pause();
            } else {
                audio_queue.resume();
                while audio_queue.size() as usize > self.sample_buffer.get_byte_size() * 10 {}
                audio_queue.queue(&self.sample_buffer.get_samples());
                let volume = if control.common.audio_enabled {
                    control.common.volume as f32 / 100.0
                } else {
                    0.0
                };
                self.sample_buffer.reset(control.target_fps, volume);
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
        self.imgui_sdl2.prepare_render(&ui, &self.window);

        self.gui_builder.build(&mut ui);

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
    fn add_sample(&mut self, sample: io::AudioSampleFormat) {
        self.sample_buffer.add(sample);
    }
}

impl io::ControllerAccess for IOSdl2ImGuiOpenGl {
    fn is_button_pressed(
        &self,
        controller_id: controllers::ControllerId,
        button: controllers::Button,
    ) -> bool {
        let sdl2_scancode = self.gui_builder.get_io_common().controller_configs
            [controller_id as usize]
            .mapping[button as usize]
            .key;
        let key_state = self.keyboard_state.get(&sdl2_scancode);
        *key_state.unwrap_or(&false)
    }
}
