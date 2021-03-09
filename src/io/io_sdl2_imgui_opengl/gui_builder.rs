use std::ops::RangeInclusive;

use imgui::im_str;

use super::{MenuBarItem, DISPLAY_HEIGHT, DISPLAY_WIDTH, MENU_BAR_HEIGHT};
use crate::{common, io::IOCommon};

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
enum GuiFont {
    _Default = 0,
    FpsCounter,
    MenuBar,
    FontsCount,
}

type GuiFonts = [imgui::FontId; GuiFont::FontsCount as usize];

pub(super) fn prepare_fonts(imgui: &mut imgui::Context) -> GuiFonts {
    let default_font = imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut fonts = [default_font; 3];
    fonts[GuiFont::FpsCounter as usize] =
        add_font_from_ttf!("../../../res/OpenSans-Regular.ttf", 30.0, imgui);

    fonts[GuiFont::MenuBar as usize] =
        add_font_from_ttf!("../../../res/Roboto-Regular.ttf", 20.0, imgui);
    fonts
}
pub(super) struct GuiBuilder {
    emulation_texture: imgui::TextureId,
    fonts: GuiFonts,
    menu_bar_item_selected: [bool; MenuBarItem::None as usize],
    io_common: IOCommon,
    rom_path: Option<String>,
}

impl GuiBuilder {
    pub fn new(emulation_texture: imgui::TextureId, fonts: GuiFonts) -> Self {
        Self {
            emulation_texture,
            menu_bar_item_selected: Default::default(),
            fonts,
            rom_path: None,
            io_common: Default::default(),
        }
    }

    pub fn get_io_common(&self) -> IOCommon {
        self.io_common
    }

    pub fn get_rom_path(&mut self) -> Option<String> {
        self.rom_path.take()
    }

    pub fn prepare_for_new_frame(&mut self, io_control: IOCommon) {
        self.menu_bar_item_selected = Default::default();
        self.rom_path = None;
        self.io_common = io_control;
    }

    pub fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.menu_bar_item_selected[item as usize]
    }

    fn update_menu_item_status(&mut self, ui: &mut imgui::Ui, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = ui.is_item_clicked(imgui::MouseButton::Left)
            || (ui.is_item_focused() && ui.is_key_pressed(sdl2::keyboard::Scancode::Return as _));
    }

    fn build_menu_bar_and_check_for_mouse_events(&mut self, fps: u16, ui: &mut imgui::Ui) {
        use MenuBarItem::*;
        with_font!(self.fonts[GuiFont::MenuBar as usize], ui, {
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(
                    ui,
                    begin_menu,
                    (im_str!("File"), !self.io_common.choose_nes_file),
                    {
                        create_menu_item!("Load Nes File", "Ctrl + O").build(ui);
                        self.update_menu_item_status(ui, LoadNesFile);

                        create_menu_item!("Quit", "Esc").build(ui);
                        self.update_menu_item_status(ui, Quit);
                    }
                );
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Emulation"), true), {
                    create_menu_item!("Power Cycle", "Ctrl + R").build(ui);
                    self.update_menu_item_status(ui, PowerCycle);

                    create_menu_item!("Pause", "Ctrl + P")
                        .selected(self.io_common.pause)
                        .build(ui);
                    self.update_menu_item_status(ui, Pause);

                    let is_speed_selected = |target_fps: u16| fps == target_fps;

                    with_token!(ui, begin_menu, (im_str!("Speed"), true), {
                        create_menu_item!("Normal", "")
                            .selected(is_speed_selected(common::DEFAULT_FPS))
                            .build(ui);
                        self.update_menu_item_status(ui, SpeedNormal);
                        create_menu_item!("Double", "")
                            .selected(is_speed_selected(common::DOUBLE_FPS))
                            .build(ui);
                        self.update_menu_item_status(ui, SpeedDouble);
                        create_menu_item!("Half", "")
                            .selected(is_speed_selected(common::HALF_FPS))
                            .build(ui);
                        self.update_menu_item_status(ui, SpeedHalf);
                        create_menu_item!("Increase", "Ctrl + =").build(ui);
                        self.update_menu_item_status(ui, SpeedIncrease);
                        create_menu_item!("Decrease", "Ctrl + -").build(ui);
                        self.update_menu_item_status(ui, SpeedDecrease);
                    });
                });
            });
            ui.set_next_item_width(100.0);
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Audio"), true), {
                    create_menu_item!("Enabled", "Ctrl + A")
                        .selected(self.io_common.audio_enabled)
                        .build(ui);
                    self.update_menu_item_status(ui, AudioEnabled);

                    imgui::ChildWindow::new("child")
                        .size([190.0, ui.current_font_size() + 3.0])
                        .border(false)
                        .scroll_bar(false)
                        .build(ui, || {
                            let range = RangeInclusive::new(0, 100);
                            imgui::Slider::new(im_str!("Volume"))
                                .range(range)
                                .display_format(im_str!("%d%%"))
                                .build(ui, &mut self.io_common.volume);
                        })
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

    fn build_fps_counter(&self, current_fps: u16, target_fps: u16, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::FpsCounter as usize], ui, {
            let text = format!("FPS {}/{}", current_fps, target_fps);
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

    pub(super) fn build(&mut self, current_fps: u16, target_fps: u16, mut ui: &mut imgui::Ui) {
        with_styles!(
            &mut ui,
            (
                imgui::StyleVar::WindowRounding(0.0),
                imgui::StyleVar::WindowBorderSize(0.0),
                imgui::StyleVar::WindowPadding([0.0, 0.0])
            ),
            {
                self.build_menu_bar_and_check_for_mouse_events(target_fps, &mut ui);
                self.build_emulation_window(&mut ui);
                self.build_fps_counter(current_fps, target_fps, &mut ui);
                if self.io_common.choose_nes_file {
                    self.build_load_nes_file_explorer();
                    self.io_common.pause = false;
                }
            }
        );
    }
}
