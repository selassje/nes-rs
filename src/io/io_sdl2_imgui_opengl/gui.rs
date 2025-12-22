use std::ops::RangeInclusive;

use imgui::im_str;

use super::{MenuBarItem, MENU_BAR_HEIGHT};
use crate::{
    common,
    io::{IOControl, FRAME_HEIGHT, FRAME_WIDTH},
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
            true
        } else {
            false
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
        add_font_from_ttf!("../../../res/fonts/OpenSans-Regular.ttf", 30.0, imgui);

    fonts[GuiFont::MenuBar as usize] =
        add_font_from_ttf!("../../../res/fonts/Roboto-Regular.ttf", 20.0, imgui);
    fonts
}

#[derive(Clone, Copy, PartialEq)]
pub enum VideoSizeControl {
    _Normal = 1,
    Double = 2,
    Triple = 3,
    Quadrupal = 4,
    FullScreen = 5,
}

impl From<VideoSizeControl> for [u32; 2] {
    fn from(val: VideoSizeControl) -> Self {
        if val == VideoSizeControl::FullScreen {
            panic!("VideoSizeControl::FullScreen size can't be converted to [u32;2]")
        }

        let scaling = val as u32;
        [scaling * FRAME_WIDTH as u32, scaling * FRAME_HEIGHT as u32]
    }
}

impl From<VideoSizeControl> for [f32; 2] {
    fn from(val: VideoSizeControl) -> Self {
        let [width, height]: [u32; 2] = val.into();
        [width as _, height as _]
    }
}

impl Default for VideoSizeControl {
    fn default() -> Self {
        Self::Double
    }
}
#[derive(Clone, Copy)]
pub struct ButtonMapping {
    pub key: sdl2::keyboard::Scancode,
}

impl Default for ButtonMapping {
    fn default() -> Self {
        Self {
            key: sdl2::keyboard::Scancode::A,
        }
    }
}

impl ButtonMapping {
    pub fn new(key: sdl2::keyboard::Scancode) -> Self {
        Self {
            key,
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct ControllerConfig {
    pub mapping: [ButtonMapping; crate::io::Button::Right as usize + 1],
    pub pending_key_select: Option<u8>,
}

impl ControllerConfig {
    pub fn new(player: u8) -> Self {
        use sdl2::keyboard::Scancode::*;
        Self {
            pending_key_select: None,
            mapping: match player {
                0 => [
                    ButtonMapping::new(Q),
                    ButtonMapping::new(E),
                    ButtonMapping::new(C),
                    ButtonMapping::new(Space),
                    ButtonMapping::new(W),
                    ButtonMapping::new(S),
                    ButtonMapping::new(A),
                    ButtonMapping::new(D),
                ],
                1 => [
                    ButtonMapping::new(Kp4),
                    ButtonMapping::new(Kp5),
                    ButtonMapping::new(Kp6),
                    ButtonMapping::new(KpPlus),
                    ButtonMapping::new(Up),
                    ButtonMapping::new(Down),
                    ButtonMapping::new(Left),
                    ButtonMapping::new(Right),
                ],
                _ => panic!("Wrong player!"),
            },
        }
    }
}
pub(super) struct Gui {
    emulation_texture: imgui::TextureId,
    fonts: GuiFonts,
    menu_bar_item_selected: [bool; MenuBarItem::Count as usize],
    io_control: IOControl,
    nes_file_path: Option<String>,
    save_state_path: Option<String>,
    load_state_path: Option<String>,
    build_menu_bar: bool,
    fd_load_nes_file: imgui_filedialog::FileDialog,
    fd_save_state: imgui_filedialog::FileDialog,
    fd_load_state: imgui_filedialog::FileDialog,
    pub video_size: [f32; 2],
    pub video_size_control: VideoSizeControl,
    pub previous_video_size_control: VideoSizeControl,
    pub audio_volume: u8,
    pub controllers_setup: bool,
    pub controller_configs: [ControllerConfig; 2],
    pub pause: bool,
}

fn create_file_dialog(
    name: &imgui::ImStr,
    title: &imgui::ImStr,
    filters: &imgui::ImStr,
) -> imgui_filedialog::FileDialog {
    imgui_filedialog::FileDialog::new(name)
        .title(title)
        .filters(filters)
        .min_size([2.0 * FRAME_WIDTH as f32 - 30.0, 200.0])
        .max_size([
            2.0 * FRAME_WIDTH as f32,
            (2 * FRAME_HEIGHT - MENU_BAR_HEIGHT as usize) as _,
        ])
}

impl Gui {
    pub fn new(emulation_texture: imgui::TextureId, fonts: GuiFonts) -> Self {
        Self {
            emulation_texture,
            menu_bar_item_selected: Default::default(),
            fonts,
            nes_file_path: None,
            save_state_path: None,
            load_state_path: None,
            io_control: IOControl {
                ..Default::default()
            },
            video_size_control: VideoSizeControl::Double,
            previous_video_size_control: VideoSizeControl::Double,
            video_size: [FRAME_WIDTH as f32 * 2.0, FRAME_HEIGHT as f32 * 2.0],
            build_menu_bar: Default::default(),
            fd_load_nes_file: create_file_dialog(
                im_str!("nes_file"),
                im_str!("Open NES file"),
                im_str!(".nes,.NES"),
            ),
            fd_save_state: create_file_dialog(
                im_str!("save_state"),
                im_str!("Save Emulation state"),
                im_str!(".nesrs,.NESRS"),
            ),
            fd_load_state: create_file_dialog(
                im_str!("load_state"),
                im_str!("Load Emulation state"),
                im_str!(".nesrs,.NESRS"),
            ),
            audio_volume: 100,
            controller_configs: [ControllerConfig::new(0), ControllerConfig::new(1)],
            controllers_setup: false,
            pause: false,
        }
    }

    pub fn get_rom_path(&mut self) -> Option<String> {
        self.nes_file_path.take()
    }
    pub fn get_save_state_path(&mut self) -> Option<String> {
        self.save_state_path.take()
    }
    pub fn get_load_state_path(&mut self) -> Option<String> {
        self.load_state_path.take()
    }

    pub fn prepare_for_new_frame(&mut self, io_control: IOControl) {
        self.nes_file_path = None;
        self.io_control = io_control;
    }

    pub fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.menu_bar_item_selected[item as usize]
    }

    fn update_menu_item_status(&mut self, ui: &mut imgui::Ui, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = ui.is_item_clicked(imgui::MouseButton::Left)
            || (ui.is_item_hovered() && ui.key_pressed_amount(imgui::Key::Enter, 0.0, 0.0) == 1);
    }

    pub fn toggle_menu_bar_item(&mut self, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = !self.menu_bar_item_selected[item as usize];
    }

    pub fn clear_menu_bar_item(&mut self, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = false;
    }

    pub fn set_menu_bar_item(&mut self, item: MenuBarItem, state: bool) {
        self.menu_bar_item_selected[item as usize] = state;
    }

    fn toggle_menu_bar_item_if_clicked(&mut self, ui: &imgui::Ui, item: MenuBarItem) {
        if ui.is_item_clicked(imgui::MouseButton::Left)
            || (ui.is_item_hovered() && ui.key_pressed_amount(imgui::Key::Enter, 0.0, 0.0) == 1)
        {
            self.menu_bar_item_selected[item as usize] =
                !self.menu_bar_item_selected[item as usize];
        }
    }
    fn build_menu_bar(&mut self, ui: &mut imgui::Ui) {
        use MenuBarItem::*;
        with_font!(self.fonts[GuiFont::MenuBar as usize], ui, {
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("File"), true), {
                    create_menu_item!("Load Nes File", "Ctrl + O").build(ui);
                    if !self.is_menu_bar_item_selected(LoadNesFile) {
                        self.update_menu_item_status(ui, LoadNesFile);
                    }
                    create_menu_item!("Save State", "Ctrl + S").build(ui);
                    if !self.is_menu_bar_item_selected(SaveState) {
                        self.update_menu_item_status(ui, SaveState);
                    }
                    create_menu_item!("Load State", "Ctrl + L").build(ui);
                    if !self.is_menu_bar_item_selected(LoadState) {
                        self.update_menu_item_status(ui, LoadState);
                    }
                    create_menu_item!("Quit", "Alt + F4").build(ui);
                    self.update_menu_item_status(ui, Quit);
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Emulation"), true), {
                    create_menu_item!("Power Cycle", "Ctrl + R").build(ui);
                    self.update_menu_item_status(ui, PowerCycle);

                    create_menu_item!("Pause", "Ctrl + P")
                        .selected(self.pause)
                        .build(ui);
                    self.update_menu_item_status(ui, Pause);

                    let target_fps = self.io_control.target_fps;
                    let is_speed_selected = |fps: u16| fps == target_fps;

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
                        create_menu_item!("200%", "F9")
                            .selected(self.video_size_control == VideoSizeControl::Double)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeDouble);
                        create_menu_item!("300%", "F10")
                            .selected(self.video_size_control == VideoSizeControl::Triple)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeTriple);
                        create_menu_item!("400%", "F11")
                            .selected(self.video_size_control == VideoSizeControl::Quadrupal)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeQuadrupal);
                        create_menu_item!("Full screen", "F12")
                            .selected(self.video_size_control == VideoSizeControl::FullScreen)
                            .build(ui);
                        self.update_menu_item_status(ui, VideoSizeFullScreen);
                    });
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Audio"), true), {
                    create_menu_item!("Enabled", "Ctrl + A")
                        .selected(self.is_menu_bar_item_selected(AudioEnabled))
                        .build(ui);

                    self.toggle_menu_bar_item_if_clicked(ui, AudioEnabled);

                    imgui::ChildWindow::new("child")
                        .size([190.0, ui.current_font_size() + 3.0])
                        .border(false)
                        .scroll_bar(false)
                        .build(ui, || {
                            let range = RangeInclusive::new(0, 100);
                            imgui::Slider::new(im_str!("Volume"))
                                .range(range)
                                .display_format(im_str!("%d%%"))
                                .build(ui, &mut self.audio_volume);
                        });
                    ui.separator();
                    create_menu_item!("Increase", "=").build(ui);
                    self.update_menu_item_status(ui, VolumeIncrease);
                    create_menu_item!("Decrease", "-").build(ui);
                    self.update_menu_item_status(ui, VolumeDecrease);
                });
            });
            with_token!(ui, begin_main_menu_bar, (), {
                with_token!(ui, begin_menu, (im_str!("Controllers"), true), {
                    unsafe {
                        imgui_sys::igSetNextWindowSize(
                            imgui_sys::ImVec2 { x: 230.0, y: 230.0 },
                            imgui::Condition::Always as i32,
                        );
                    }
                    self.controllers_setup =
                        with_token!(ui, begin_menu, (im_str!("Setup"), true), {
                            self.build_controllers_setup_window(ui);
                        });

                    if !self.controllers_setup {
                        self.controller_configs[0].pending_key_select = Option::None;
                        self.controller_configs[1].pending_key_select = Option::None;
                    }
                });
            });
        });
    }

    fn build_emulation_window(&self, ui: &mut imgui::Ui) {
        with_styles!(ui, (imgui::StyleVar::WindowBorderSize(0.0)), {
            let vertical_offset = if self.build_menu_bar {
                MENU_BAR_HEIGHT as f32
            } else {
                0.0
            };
            create_unmovable_simple_window!("emulation", [0.0, vertical_offset], self.video_size)
                .bring_to_front_on_focus(false)
                .build(ui, || {
                    imgui::Image::new(self.emulation_texture, self.video_size).build(ui);
                });
        });
    }

    fn build_fps_counter(&self, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::FpsCounter as usize], ui, {
            let text = format!(
                "FPS {}/{}",
                self.io_control.current_fps, self.io_control.target_fps
            );
            let [video_width, _]: [f32; 2] = self.video_size;
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
        if self.is_menu_bar_item_selected(MenuBarItem::LoadNesFile) {
            self.pause = false;
            self.toggle_menu_bar_item(MenuBarItem::LoadNesFile);
            self.fd_load_nes_file.open_modal();
        }
        if self.fd_load_nes_file.display() {
            if self.fd_load_nes_file.is_ok() {
                let file = &self.fd_load_nes_file.selection().unwrap().files()[0];
                self.nes_file_path = Some(file.to_str().unwrap().to_owned());
            }

            self.fd_load_nes_file.close();
        }
    }

    fn build_save_state_file_explorer(&mut self) {
        if self.is_menu_bar_item_selected(MenuBarItem::SaveState) {
            self.pause = false;
            self.toggle_menu_bar_item(MenuBarItem::SaveState);
            self.fd_save_state.open_modal();
        }
        if self.fd_save_state.display() {
            if self.fd_save_state.is_ok() {
                let file = self.fd_save_state.current_file_path().unwrap();
                self.save_state_path = Some(file);
            }

            self.fd_save_state.close();
        }
    }

    fn build_load_state_file_explorer(&mut self) {
        if self.is_menu_bar_item_selected(MenuBarItem::LoadState) {
            self.pause = false;
            self.toggle_menu_bar_item(MenuBarItem::LoadState);
            self.fd_load_state.open_modal();
        }
        if self.fd_load_state.display() {
            if self.fd_load_state.is_ok() {
                let file = &self.fd_load_state.selection().unwrap().files()[0];
                self.load_state_path = Some(file.to_str().unwrap().to_owned());
            }
            self.fd_load_state.close();
        }
    }

    pub fn try_get_key_selection(&mut self, event: &sdl2::event::Event) {
        if let sdl2::event::Event::KeyDown {
            scancode, keymod, ..
        } = *event
        {
            if keymod & sdl2::keyboard::Mod::NOMOD == sdl2::keyboard::Mod::NOMOD {
                if let Some(scancode) = scancode {
                    if let Some(button) = self.controller_configs[0].pending_key_select.take() {
                        self.controller_configs[0].mapping[button as usize].key = scancode;

                        self.controller_configs[0].pending_key_select = None;
                    } else if let Some(button) =
                        self.controller_configs[1].pending_key_select.take()
                    {
                        self.controller_configs[1].mapping[button as usize].key = scancode;
                    }
                }
            }
        };
    }
    pub fn is_key_selection_pending(&self) -> bool {
        self.controllers_setup
            && (self.controller_configs[0].pending_key_select.is_some()
                || self.controller_configs[1].pending_key_select.is_some())
    }

    fn build_controller_setup_for_player(&mut self, player_index: usize, ui: &mut imgui::Ui) {
        let controller_config = &mut self.controller_configs[player_index];

        for i in 0..8u8 {
            let button = crate::io::Button::from(i);
            let caption = imgui::ImString::from(button.to_string());
            let key = controller_config.mapping[i as usize].key;
            let mut text = key.to_string();
            if ui.small_button(&caption) && controller_config.pending_key_select.is_none() {
                controller_config.pending_key_select = Some(i);
            }
            if Some(i) == controller_config.pending_key_select {
                text = String::from("Press Key");
            }
            ui.same_line(150.0);
            ui.text(text);
        }
    }

    fn build_controllers_setup_window(&mut self, ui: &mut imgui::Ui) {
        with_font!(self.fonts[GuiFont::MenuBar as usize], ui, {
            with_styles!(ui, (imgui::StyleVar::WindowBorderSize(2.0)), {
                {
                    if let Some(tab_bar) = imgui::TabBar::new(im_str!("Players")).begin(ui) {
                        if self.controller_configs[1].pending_key_select.is_none() {
                            if let Some(player_1_tab) =
                                imgui::TabItem::new(im_str!("Player 1")).begin(ui)
                            {
                                self.build_controller_setup_for_player(0, ui);
                                player_1_tab.end(ui);
                            }
                        }
                        if self.controller_configs[0].pending_key_select.is_none() {
                            if let Some(player_2_tab) =
                                imgui::TabItem::new(im_str!("Player 2")).begin(ui)
                            {
                                self.build_controller_setup_for_player(1, ui);
                                player_2_tab.end(ui);
                            }
                        }
                        tab_bar.end(ui);
                    }
                }
            });
        });
    }

    pub(super) fn build(&mut self, ui: &mut imgui::Ui) {
        self.build_menu_bar = !(self.video_size_control == VideoSizeControl::FullScreen);
        with_styles!(
            ui,
            (
                imgui::StyleVar::WindowRounding(0.0),
                imgui::StyleVar::WindowBorderSize(0.0),
                imgui::StyleVar::WindowPadding([0.0, 0.0])
            ),
            {
                if self.build_menu_bar {
                    self.build_menu_bar(ui);
                }
                self.build_emulation_window(ui);
                self.build_fps_counter(ui);
                self.build_load_nes_file_explorer();
                self.build_save_state_file_explorer();
                self.build_load_state_file_explorer();
            }
        );
    }
}
