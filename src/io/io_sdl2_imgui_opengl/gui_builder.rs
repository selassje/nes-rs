use std::ops::RangeInclusive;

use imgui::im_str;

use super::{MenuBarItem, MENU_BAR_HEIGHT};
use crate::{
    common,
    io::IOControl,
    io::{IOCommon, VideoSizeControl},
};

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
    io_control: IOControl,
    rom_path: Option<String>,
}

impl GuiBuilder {
    pub fn new(emulation_texture: imgui::TextureId, fonts: GuiFonts) -> Self {
        Self {
            emulation_texture,
            menu_bar_item_selected: Default::default(),
            fonts,
            rom_path: None,
            io_control: Default::default(),
        }
    }

    pub fn get_io_common(&self) -> IOCommon {
        self.io_control.common
    }

    pub fn get_rom_path(&mut self) -> Option<String> {
        self.rom_path.take()
    }

    pub fn prepare_for_new_frame(&mut self, io_control: IOControl) {
        self.menu_bar_item_selected = Default::default();
        self.rom_path = None;
        self.io_control = io_control
    }

    pub fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.menu_bar_item_selected[item as usize]
    }

    fn update_menu_item_status(&mut self, ui: &mut imgui::Ui, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = ui.is_item_clicked(imgui::MouseButton::Left)
            || (ui.is_item_hovered()
                && ui.key_pressed_amount(sdl2::keyboard::Scancode::Return as _, 0.0, 0.0) == 1);
    }

    fn build_menu_bar_and_check_for_mouse_events(&mut self, fps: u16, ui: &mut imgui::Ui) {
        use MenuBarItem::*;
        with_font!(self.fonts[GuiFont::MenuBar as usize], ui, {
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(
                    ui,
                    begin_menu,
                    (im_str!("File"), !self.io_control.common.choose_nes_file),
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
                        .selected(self.io_control.common.pause)
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
                        ui.separator();
                        create_menu_item!("Increase", "Ctrl + =").build(ui);
                        self.update_menu_item_status(ui, SpeedIncrease);
                        create_menu_item!("Decrease", "Ctrl + -").build(ui);
                        self.update_menu_item_status(ui, SpeedDecrease);
                    });
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Video"), true), {
                    with_token!(ui, begin_menu, (im_str!("Size"), true), {
                        create_menu_item!("100%", "F8")
                            .selected(self.io_control.common.video_size == VideoSizeControl::Normal)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeNormal);
                        create_menu_item!("200%", "F9")
                            .selected(self.io_control.common.video_size == VideoSizeControl::Double)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeDouble);
                        create_menu_item!("300%", "F10")
                            .selected(self.io_control.common.video_size == VideoSizeControl::Triple)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeTriple);
                        create_menu_item!("400%", "F11")
                            .selected(
                                self.io_control.common.video_size == VideoSizeControl::Quadrupal,
                            )
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeQuadrupal);
                        create_menu_item!("Full screen", "F12")
                            .selected(
                                self.io_control.common.video_size == VideoSizeControl::FullScreen,
                            )
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeFullScreen);
                    });
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Audio"), true), {
                    create_menu_item!("Enabled", "Ctrl + A")
                        .selected(self.io_control.common.audio_enabled)
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
                                .build(ui, &mut self.io_control.common.volume);
                        });
                    ui.separator();
                    create_menu_item!("Increase", "=").build(ui);
                    self.update_menu_item_status(ui, VolumeIncrease);
                    create_menu_item!("Decrease", "-").build(ui);
                    self.update_menu_item_status(ui, VolumeDecrease);
                });
            });
        });
    }

    fn build_emulation_window(&self, ui: &mut imgui::Ui) {
        create_unmovable_simple_window!(
            "emulation",
            [0.0, MENU_BAR_HEIGHT as _],
            self.get_io_common().video_size.into()
        )
        .bring_to_front_on_focus(false)
        .build(ui, || {
            imgui::Image::new(
                self.emulation_texture,
                self.get_io_common().video_size.into(),
            )
            .build(ui);
        });
    }

    fn build_fps_counter(&self, current_fps: u16, target_fps: u16, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::FpsCounter as usize], ui, {
            let text = format!("FPS {}/{}", current_fps, target_fps);
            let [video_width, _]: [f32; 2] = self.get_io_common().video_size.into();
            let text_size = ui.calc_text_size(
                imgui::ImString::new(text.clone()).as_ref(),
                false,
                video_width,
            );
            create_unmovable_simple_window!(
                "fps",
                [video_width - text_size[0], MENU_BAR_HEIGHT as _],
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
                if self.io_control.common.choose_nes_file {
                    self.build_load_nes_file_explorer();
                    self.io_control.common.pause = false;
                }
            }
        );
    }
}
