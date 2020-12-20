use std::collections::HashMap;
use std::default::Default;
use std::iter::FromIterator;

use super::io_internal;
use crate::common;
use crate::io;

use gl::types::*;
use imgui::im_str;

const SAMPLE_RATE: usize = 44100;
const SAMPLE_RATE_ADJ: usize = (SAMPLE_RATE as f32 * 1.0000) as usize;
const INITIAL_SAMPLE_BUCKET_SIZE: f32 =
    (common::FPS * common::CPU_CYCLES_PER_FRAME) as f32 / SAMPLE_RATE_ADJ as f32;
const BUFFER_SIZE: usize = 10000;

const DISPLAY_SCALING: usize = 2;
const DISPLAY_WIDTH: usize = DISPLAY_SCALING * io::FRAME_WIDTH;
const DISPLAY_HEIGHT: usize = DISPLAY_SCALING * io::FRAME_HEIGHT;
const MENU_BAR_HEIGHT: usize = 18;

macro_rules! add_font_from_ttf {
    ($font_path:literal,$size:expr, $imgui:ident) => {{
        let font_source = imgui::FontSource::TtfData {
            data: include_bytes!($font_path),
            size_pixels: $size,
            config: None,
        };
        $imgui.fonts().add_font(&[font_source])
    }};
}
macro_rules! with_font {
    ($font:expr, $ui:ident, $code:expr) => {{
        let font_token = $ui.push_font($font);
        $code
        font_token.pop($ui);
    }};
}
macro_rules! with_token {
    ($ui:expr, $token_function:ident, ($($arg:expr),*), $code:expr) => {{
        if let Some(token) = $ui.$token_function($($arg),*) {
            $code
            token.end($ui);
        }
    }};
}
macro_rules! with_styles {
    ($ui:expr, ($($style:expr),*), $code:expr) => {{
        let styles_token = $ui.push_style_vars(&[$($style),*]);
        $code
        styles_token.pop($ui);
}};
}
macro_rules! create_simple_window {
    ($name:tt, $position:expr, $size:expr, $condition_pos:expr, $condition_size:expr) => {{
        imgui::Window::new(im_str!($name))
            .scrollable(false)
            .no_decoration()
            .position($position, $condition_pos)
            .size($size, $condition_size)
    }};
}

macro_rules! create_unmovable_simple_window {
    ($name:tt, $position:expr, $size:expr) => {{
        create_simple_window!(
            $name,
            $position,
            $size,
            imgui::Condition::Always,
            imgui::Condition::Always
        )
    }};
}

macro_rules! create_menu_item {
    ($name:tt, $shortcut:tt) => {{
        imgui::MenuItem::new(im_str!($name))
            .selected(false)
            .enabled(true)
            .shortcut(im_str!($shortcut))
    }};
}

struct SampleBuffer {
    index: usize,
    sum: f32,
    bucket_size: f32,
    target_bucket_size: f32,
    buffer: [io::SampleFormat; BUFFER_SIZE],
}

enum GuiFont {
    _Default = 0,
    FpsCounter,
    MenuBar,
    FontsCount,
}
#[derive(PartialEq)]
enum MenuBarItem {
    LoadNesFile,
    Quit,
    PowerCycle,
    None,
}

type GuiFonts = [imgui::FontId; GuiFont::FontsCount as usize];

impl SampleBuffer {
    fn add(&mut self, sample: io::SampleFormat) {
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

#[derive(Clone, Copy, Default)]
struct KeyboardShortCuts {
    load_nes_file: bool,
    power_cycle: bool,
    quit: bool,
}

struct GuiBuilder {
    emulation_texture: imgui::TextureId,
    fonts: GuiFonts,
    menu_bar_item_selected: [bool; MenuBarItem::None as usize],
    choose_nes_file: bool,
    rom_path: Option<String>,
}

impl GuiBuilder {
    fn is_menu_item_selected(&self, ui: &mut imgui::Ui) -> bool {
        ui.is_item_clicked(imgui::MouseButton::Left)
            || (ui.is_item_focused() && ui.is_key_pressed(sdl2::keyboard::Scancode::Return as _))
    }

    fn build_menu_bar_and_check_for_mouse_events(&mut self, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::MenuBar as usize], ui, {
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("File"), !self.choose_nes_file), {
                    // create_menu_item!("Load Nes File", "Ctrl + O").build(ui);
                    create_menu_item!("Load Nes File", "Ctrl + O").build(ui);
                    self.menu_bar_item_selected[MenuBarItem::LoadNesFile as usize] =
                        self.is_menu_item_selected(ui);

                    create_menu_item!("Quit", "Esc").build(ui);
                    self.menu_bar_item_selected[MenuBarItem::Quit as usize] =
                        self.is_menu_item_selected(ui);
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Emulation"), true), {
                    create_menu_item!("Power Cycle", "Ctrl + R").build(ui);
                    self.menu_bar_item_selected[MenuBarItem::PowerCycle as usize] =
                        self.is_menu_item_selected(ui);
                });
            });
        });
    }

    fn build_emulation_window(&self, ui: &mut imgui::Ui) {
        create_unmovable_simple_window!(
            "emulation",
            [0.0, MENU_BAR_HEIGHT as _],
            [DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _]
        )
        .bring_to_front_on_focus(false)
        .build(ui, || {
            imgui::Image::new(
                self.emulation_texture,
                [DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _],
            )
            .build(ui);
        });
    }

    fn build_fps_counter(&self, fps: u16, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::FpsCounter as usize], ui, {
            let text = format!("FPS {}", fps);
            let text_size = ui.calc_text_size(
                imgui::ImString::new(text.clone()).as_ref(),
                false,
                DISPLAY_WIDTH as _,
            );
            create_unmovable_simple_window!(
                "fps",
                [DISPLAY_WIDTH as f32 - text_size[0], MENU_BAR_HEIGHT as _],
                text_size
            )
            .bg_alpha(0.0)
            .build(ui, || {
                ui.text(text);
            });
        });
    }

    fn build_load_nes_file_explorer(&mut self) {
        let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
            panic!(e);
        });
        match result {
            nfd::Response::Okay(file_path) => {
                self.rom_path = Some(file_path);
            }
            nfd::Response::Cancel => {}
            _ => panic!("Unsupported file selection"),
        }
    }

    fn build(&mut self, fps: u16, mut ui: &mut imgui::Ui) {
        with_styles!(
            &mut ui,
            (
                imgui::StyleVar::WindowRounding(0.0),
                imgui::StyleVar::WindowBorderSize(0.0),
                imgui::StyleVar::WindowPadding([0.0, 0.0])
            ),
            {
                self.build_menu_bar_and_check_for_mouse_events(&mut ui);
                self.build_emulation_window(&mut ui);
                self.build_fps_counter(fps, &mut ui);
                if self.choose_nes_file {
                    self.build_load_nes_file_explorer();
                }
            }
        );
    }
}

pub struct IOSdl2ImGuiOpenGl {
    io_internal: io_internal::IOInternal,
    sample_buffer: SampleBuffer,
    audio_queue: sdl2::audio::AudioQueue<io::SampleFormat>,
    events: sdl2::EventPump,
    keyboard_state: HashMap<sdl2::keyboard::Scancode, bool>,
    imgui: imgui::Context,
    imgui_sdl2: imgui_sdl2::ImguiSdl2,
    window: sdl2::video::Window,
    renderer: imgui_opengl_renderer::Renderer,
    _gl_context: sdl2::video::GLContext,
    gui_builder: GuiBuilder,
    keyboard_shortcuts: KeyboardShortCuts,
}

fn keycode_to_sdl2_scancode(key: io::KeyCode) -> sdl2::keyboard::Scancode {
    use io::KeyCode;
    use sdl2::keyboard::Scancode;
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

impl IOSdl2ImGuiOpenGl {
    pub fn new(title: &str) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(BUFFER_SIZE as u16),
        };

        let audio_queue: sdl2::audio::AudioQueue<io::SampleFormat> =
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

        let fonts = Self::prepare_fonts(&mut imgui);

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

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

        IOSdl2ImGuiOpenGl {
            io_internal: io_internal::IOInternal::new(),
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
            imgui,
            window,
            imgui_sdl2,
            renderer,
            _gl_context,
            gui_builder: GuiBuilder {
                emulation_texture: imgui::TextureId::from(emulation_texture as usize),
                fonts,
                menu_bar_item_selected: Default::default(),
                choose_nes_file: false,
                rom_path: None,
            },
            keyboard_shortcuts: Default::default(),
        }
    }
    fn prepare_fonts(imgui: &mut imgui::Context) -> GuiFonts {
        let default_font = imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let mut fonts = [default_font; 3];
        fonts[GuiFont::FpsCounter as usize] =
            add_font_from_ttf!("../../res/OpenSans-Regular.ttf", 30.0, imgui);

        fonts[GuiFont::MenuBar as usize] =
            add_font_from_ttf!("../../res/Roboto-Regular.ttf", 20.0, imgui);
        fonts
    }

    fn update_io_state(&mut self, io_state: &mut io::IOState) {
        io_state.quit = self.keyboard_shortcuts.quit
            || self.gui_builder.menu_bar_item_selected[MenuBarItem::Quit as usize];
        io_state.power_cycle = self.keyboard_shortcuts.power_cycle
            || self.gui_builder.menu_bar_item_selected[MenuBarItem::PowerCycle as usize];
        self.gui_builder.choose_nes_file = self.keyboard_shortcuts.load_nes_file
            || self.gui_builder.menu_bar_item_selected[MenuBarItem::LoadNesFile as usize];
    }

    fn check_for_keyboard_shortcuts(
        event: &sdl2::event::Event,
        keyboard_shortcuts: &mut KeyboardShortCuts,
    ) {
        use sdl2::keyboard::Scancode;
        match *event {
            sdl2::event::Event::KeyDown {
                scancode, keymod, ..
            } => {
                if let Some(scancode) = scancode {
                    if scancode == Scancode::Escape {
                        keyboard_shortcuts.quit = true;
                    }
                    if sdl2::keyboard::Mod::LCTRLMOD & keymod == sdl2::keyboard::Mod::LCTRLMOD {
                        keyboard_shortcuts.power_cycle = scancode == Scancode::R;
                        if scancode == Scancode::O {
                            keyboard_shortcuts.load_nes_file = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl io::IO for IOSdl2ImGuiOpenGl {
    fn present_frame(&mut self, control: io::IOControl) -> io::IOState {
        let mut io_state: io::IOState = Default::default();
        self.gui_builder.menu_bar_item_selected = Default::default();
        self.gui_builder.rom_path = None;
        self.keyboard_shortcuts = Default::default();

        self.keyboard_state = HashMap::from_iter(self.events.keyboard_state().scancodes());
        for event in self.events.poll_iter() {
            Self::check_for_keyboard_shortcuts(&event, &mut self.keyboard_shortcuts);
            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
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

        if self.gui_builder.choose_nes_file {
            self.audio_queue.pause();
            self.audio_queue.clear();
        }

        self.gui_builder.build(control.fps, &mut ui);

        io_state.load_nes_file = self.gui_builder.rom_path.take();

        self.renderer.render(ui);
        self.update_io_state(&mut io_state);
        self.window.gl_swap_window();

        self.audio_queue.resume();

        self.audio_queue
            .queue(&self.sample_buffer.buffer[..self.sample_buffer.index]);

        self.sample_buffer.reset(control.fps);

        io_state
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
        let sdl2_scancode = keycode_to_sdl2_scancode(key);
        let key_state = self.keyboard_state.get(&sdl2_scancode);
        *key_state.unwrap_or(&false)
    }
}
