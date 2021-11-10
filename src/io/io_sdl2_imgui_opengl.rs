mod audio_sample_buffer;
mod gui;
mod keyboard_shortcuts;

use std::default::Default;
use std::{borrow::BorrowMut, collections::HashMap};

use self::gui::VideoSizeControl;

use super::{io_internal, FRAME_HEIGHT, FRAME_WIDTH};
use crate::{controllers, io};

use gl::types::*;
use sdl2::image::ImageRWops;

const MENU_BAR_HEIGHT: u32 = 18;
const MIN_WINDOW_WIDTH: u32 = 360;

type Size = [f32; 2];

#[derive(Copy, Clone, PartialEq)]
pub enum MenuBarItem {
    LoadNesFile,
    SaveState,
    LoadState,
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
    Count,
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
    gui: gui::Gui,
    cancel: bool,
    is_video_size_change_pending: bool,
    keyboard_shortcuts: keyboard_shortcuts::KeyboardShortcuts,
    render_frame: bool,
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
            if cfg!(target_arch = "wasm32") {
                gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
                gl_attr.set_context_version(3, 0);
            } else {
                gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
                gl_attr.set_context_version(4, 3);
            };
        }
        let [video_width, video_height]: [u32; 2] = gui::VideoSizeControl::Double.into();
        let mut window = video_subsys
            .window("NES-RS", video_width, MENU_BAR_HEIGHT as u32 + video_height)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        window.set_icon(
            sdl2::rwops::RWops::from_bytes(include_bytes!("../../static/favicon.ico"))
                .unwrap()
                .load_png()
                .unwrap(),
        );

        let _gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");

        gl::load_with(|s| video_subsys.gl_get_proc_address(s) as _);

        #[cfg(not(target_os = "emscripten"))]
        let _ = video_subsys.gl_set_swap_interval(0);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        imgui
            .io_mut()
            .config_flags
            .set(imgui::ConfigFlags::NAV_ENABLE_KEYBOARD, true);
        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

        let fonts = gui::prepare_fonts(&mut imgui);

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

        let gui_builder = gui::Gui::new(imgui::TextureId::from(emulation_texture as usize), fonts);

        IOSdl2ImGuiOpenGl {
            io_internal: io_internal::IOInternal::new(),
            sample_buffer: audio_sample_buffer::AudioSampleBuffer::new(),
            maybe_audio_queue,
            events,
            keyboard_state: HashMap::new(),
            imgui,
            window,
            imgui_sdl2,
            renderer,
            _gl_context,
            gui: gui_builder,
            keyboard_shortcuts: Default::default(),
            cancel: false,
            is_video_size_change_pending: false,
            render_frame: false,
        }
    }

    fn update_io_state(&mut self, io_state: &mut io::IOState) {
        io_state.quit |= self.is_menu_bar_item_selected(MenuBarItem::Quit);
        io_state.power_cycle = self.is_menu_bar_item_selected(MenuBarItem::PowerCycle);
        io_state.load_nes_file = self.gui.get_rom_path();
        io_state.save_state = self.gui.get_save_state_path();
        io_state.load_state = self.gui.get_load_state_path();

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

        let audio_volume = self.gui.audio_volume;

        if self.is_menu_bar_item_selected(MenuBarItem::VolumeIncrease) {
            self.gui.audio_volume = std::cmp::min(100, audio_volume + 5);
        }

        if self.is_menu_bar_item_selected(MenuBarItem::VolumeDecrease) {
            self.gui.audio_volume = std::cmp::max(0, audio_volume as i32 - 5) as u8
        }

        {
            #[cfg(target_os = "emscripten")]
            if self.gui.video_size_control == VideoSizeControl::FullScreen
                && self.window.fullscreen_state() != sdl2::video::FullscreenType::Desktop
            {
                self.cancel = true;
            } else if self.gui.video_size_control != VideoSizeControl::FullScreen {
                let [expected_width, _]: [u32; 2] = self.gui.video_size_control.into();
                let (actual_width, _) = self.window.size();
                if actual_width as u32 != expected_width {
                    self.is_video_size_change_pending = true;
                }
            }

            let mut set_video_size_selection =
                |item: MenuBarItem, video_size_ctrl: gui::VideoSizeControl| {
                    if self.is_menu_bar_item_selected(item)
                        && video_size_ctrl != self.gui.video_size_control
                    {
                        if self.gui.video_size_control != gui::VideoSizeControl::FullScreen {
                            self.gui.previous_video_size_control = self.gui.video_size_control;
                        }
                        self.gui.video_size_control = video_size_ctrl;
                        self.is_video_size_change_pending = true;
                    }
                    self.gui.clear_menu_bar_item(item);
                };
            set_video_size_selection(MenuBarItem::VideoSizeDouble, gui::VideoSizeControl::Double);
            set_video_size_selection(MenuBarItem::VideoSizeTriple, gui::VideoSizeControl::Triple);
            set_video_size_selection(
                MenuBarItem::VideoSizeQuadrupal,
                gui::VideoSizeControl::Quadrupal,
            );
            set_video_size_selection(
                MenuBarItem::VideoSizeFullScreen,
                gui::VideoSizeControl::FullScreen,
            );

            if self.cancel && self.gui.video_size_control == VideoSizeControl::FullScreen {
                self.gui.video_size_control = self.gui.previous_video_size_control;
                self.is_video_size_change_pending = true;
            }
        }
        {
            let mut toggle = |item: MenuBarItem| {
                if self.keyboard_shortcuts.is_menu_bar_item_selected(item) {
                    self.gui.toggle_menu_bar_item(item)
                }
            };
            toggle(MenuBarItem::AudioEnabled);
        }
        let toggle = |item: MenuBarItem, value: bool| {
            if self.is_menu_bar_item_selected(item) {
                !value
            } else {
                value
            }
        };

        let toggled_pause = toggle(MenuBarItem::Pause, self.gui.pause);
        self.gui.controllers_setup =
            toggle(MenuBarItem::ControllersSetup, self.gui.controllers_setup);
        self.gui.pause = toggled_pause;
        io_state.pause = self.gui.pause;
    }

    fn check_for_keyboard_shortcuts(
        event: &sdl2::event::Event,
        keyboard_shortcuts: &mut keyboard_shortcuts::KeyboardShortcuts,
    ) {
        if let sdl2::event::Event::KeyDown {
            scancode: Some(scancode),
            keymod,
            ..
        } = *event
        {
            keyboard_shortcuts.update(scancode, keymod)
        }
    }

    fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.keyboard_shortcuts.is_menu_bar_item_selected(item)
            || self.gui.is_menu_bar_item_selected(item)
    }

    fn set_window_tile(&mut self, control: &io::IOControl) {
        if let Some(ref title) = control.title {
            self.window.borrow_mut().set_title(title).unwrap();
        }
    }

    fn set_window_size_and_get_video_size(&mut self) -> Size {
        if self.gui.video_size_control != gui::VideoSizeControl::FullScreen {
            let [video_width, video_height]: [u32; 2] = self.gui.video_size_control.into();
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
        if self.is_video_size_change_pending {
            let video_size = self.set_window_size_and_get_video_size();
            self.gui.video_size = video_size;
            self.is_video_size_change_pending = false;
        }
        self.gui.prepare_for_new_frame(control.clone());
        self.keyboard_shortcuts = Default::default();
        self.cancel = false;

        self.keyboard_state = self
            .events
            .keyboard_state()
            .scancodes()
            .collect::<HashMap<_, _>>();
        for event in self.events.poll_iter() {
            if self.gui.is_key_selection_pending() {
                self.gui.try_get_key_selection(&event);
            } else {
                Self::check_for_keyboard_shortcuts(&event, &mut self.keyboard_shortcuts);
                if let sdl2::event::Event::KeyDown {
                    scancode: Some(sdl2::keyboard::Scancode::Escape),
                    ..
                } = event
                {
                    self.cancel = true;
                }
                self.imgui_sdl2.handle_event(&mut self.imgui, &event);
            }
            if let sdl2::event::Event::Window { win_event, .. } = event {
                io_state.quit = win_event == sdl2::event::WindowEvent::Close
            };
        }

        if let Some(ref audio_queue) = self.maybe_audio_queue {
            if self.gui.pause {
                audio_queue.pause();
            } else {
                audio_queue.resume();
                let audio_saturation_threshold = self.sample_buffer.get_byte_size() as u32 * 10;
                #[cfg(not(target_os = "emscripten"))]
                {
                    while audio_queue.size() > audio_saturation_threshold {}
                    audio_queue.queue(self.sample_buffer.get_samples());
                }
                #[cfg(target_os = "emscripten")]
                if audio_queue.size() < audio_saturation_threshold {
                    audio_queue.queue(&self.sample_buffer.get_samples());
                }

                let volume = if self
                    .gui
                    .is_menu_bar_item_selected(MenuBarItem::AudioEnabled)
                {
                    self.gui.audio_volume as f32 / 100.0
                } else {
                    0.0
                };

                self.sample_buffer.reset(control.target_fps, volume);
            }
        }
        if self.render_frame {
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
            self.render_frame = false;
        }

        self.imgui_sdl2.prepare_frame(
            self.imgui.io_mut(),
            &self.window,
            &self.events.mouse_state(),
        );

        let mut ui = self.imgui.frame();
        self.imgui_sdl2.prepare_render(&ui, &self.window);

        self.gui.build(&mut ui);

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
        self.render_frame = x == FRAME_WIDTH - 1 && y == 239;
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
        button: io::Button,
    ) -> bool {
        let sdl2_scancode =
            self.gui.controller_configs[controller_id as usize].mapping[button as usize].key;
        let key_state = self.keyboard_state.get(&sdl2_scancode);
        *key_state.unwrap_or(&false)
    }
}
