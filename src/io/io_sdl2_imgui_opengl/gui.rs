use super::{MenuBarItem, MENU_BAR_HEIGHT};
use crate::{
    common, controllers,
    io::{IOControl, FRAME_HEIGHT, FRAME_WIDTH},
};

use crate::io::MouseClick;

use imgui::ImString;

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

#[derive(Default, Clone, Copy, PartialEq)]
pub enum VideoSizeControl {
    _Normal = 1,
    #[default]
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
        Self { key }
    }
}
#[derive(Clone, Copy, Default)]
pub struct ControllerConfig {
    pub mapping: [ButtonMapping; crate::io::Button::Right as usize + 1],
    pub pending_key_select: Option<u8>,
    pub controller_type: controllers::ControllerType,
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
            controller_type: controllers::ControllerType::StdNesController,
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
    pub mouse_click: Option<MouseClick>,
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
        let nes_file_label = ImString::new("nes_file");
        let open_nes_file_label = ImString::new("Open NES file");
        let open_nes_file_filters_label = ImString::new(".nes,.NES");
        let save_state_label = ImString::new("save_state");
        let save_state_title = ImString::new("Save Emulation state");
        let save_state_filters = ImString::new(".nesrs");
        let load_state_label = ImString::new("load_state");
        let load_state_title = ImString::new("Load Emulation state");
        let load_state_filters = ImString::new(".nesrs,.NESRS");
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
                nes_file_label.as_ref(),
                open_nes_file_label.as_ref(),
                open_nes_file_filters_label.as_ref(),
            ),
            fd_save_state: create_file_dialog(
                save_state_label.as_ref(),
                save_state_title.as_ref(),
                save_state_filters.as_ref(),
            ),
            fd_load_state: create_file_dialog(
                load_state_label.as_ref(),
                load_state_title.as_ref(),
                load_state_filters.as_ref(),
            ),
            audio_volume: 100,
            controller_configs: [ControllerConfig::new(0), ControllerConfig::new(1)],
            controllers_setup: false,
            pause: false,
            mouse_click: None,
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

    fn update_menu_item_status(&mut self, ui: &imgui::Ui, item: MenuBarItem) {
        self.menu_bar_item_selected[item as usize] = ui.is_item_clicked()
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
        if ui.is_item_clicked()
            || (ui.is_item_hovered() && ui.key_pressed_amount(imgui::Key::Enter, 0.0, 0.0) == 1)
        {
            self.menu_bar_item_selected[item as usize] =
                !self.menu_bar_item_selected[item as usize];
        }
    }

    fn build_menu_bar(&mut self, ui: &imgui::Ui) {
        use MenuBarItem::*;
        let font = ui.push_font(self.fonts[GuiFont::MenuBar as usize]);

        #[allow(clippy::redundant_pattern_matching)]
        if let Some(_) = ui.begin_main_menu_bar() {
            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_menu("File") {
                ui.menu_item_config("Load Nes File")
                    .shortcut("Ctrl+O")
                    .build();

                if !self.is_menu_bar_item_selected(LoadNesFile) {
                    self.update_menu_item_status(ui, LoadNesFile);
                }

                ui.menu_item_config("Save State").shortcut("Ctrl+S").build();
                if !self.is_menu_bar_item_selected(SaveState) {
                    self.update_menu_item_status(ui, SaveState);
                }

                ui.menu_item_config("Load State").shortcut("Ctrl+L").build();
                if !self.is_menu_bar_item_selected(LoadState) {
                    self.update_menu_item_status(ui, LoadState);
                }

                ui.menu_item_config("Quit").shortcut("Alt+F5").build();
                self.update_menu_item_status(ui, Quit);
            }

            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_menu("Emulation") {
                ui.menu_item_config("Power Cycle")
                    .shortcut("Ctrl+R")
                    .build();
                self.update_menu_item_status(ui, PowerCycle);

                ui.menu_item_config("Pause")
                    .shortcut("Ctrl+P")
                    .selected(self.pause)
                    .build();
                self.update_menu_item_status(ui, Pause);

                #[allow(clippy::redundant_pattern_matching)]
                if let Some(_) = ui.begin_menu("Speed") {
                    let target_fps = self.io_control.target_fps;
                    let is_speed_selected = |fps: u16| fps == target_fps;

                    ui.menu_item_config("Normal")
                        .selected(is_speed_selected(common::DEFAULT_FPS))
                        .build();
                    self.update_menu_item_status(ui, SpeedNormal);

                    ui.menu_item_config("Double")
                        .selected(is_speed_selected(common::DOUBLE_FPS))
                        .build();
                    self.update_menu_item_status(ui, SpeedDouble);

                    ui.menu_item_config("Half")
                        .selected(is_speed_selected(common::HALF_FPS))
                        .build();
                    self.update_menu_item_status(ui, SpeedHalf);

                    ui.separator();

                    ui.menu_item_config("Increase").shortcut("Ctrl+=").build();
                    self.update_menu_item_status(ui, SpeedIncrease);

                    ui.menu_item_config("Decrease").shortcut("Ctrl+-").build();
                    self.update_menu_item_status(ui, SpeedDecrease);
                }
            }

            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_menu("Video") {
                #[allow(clippy::redundant_pattern_matching)]
                if let Some(_) = ui.begin_menu("Size") {
                    ui.menu_item_config("200%")
                        .shortcut("F9")
                        .selected(self.video_size_control == VideoSizeControl::Double)
                        .build();
                    self.update_menu_item_status(ui, VideoSizeDouble);

                    ui.menu_item_config("300%")
                        .shortcut("F10")
                        .selected(self.video_size_control == VideoSizeControl::Triple)
                        .build();
                    self.update_menu_item_status(ui, VideoSizeTriple);

                    ui.menu_item_config("400%")
                        .shortcut("F11")
                        .selected(self.video_size_control == VideoSizeControl::Quadrupal)
                        .build();
                    self.update_menu_item_status(ui, VideoSizeQuadrupal);

                    ui.menu_item_config("Full screen")
                        .shortcut("F12")
                        .selected(self.video_size_control == VideoSizeControl::FullScreen)
                        .build();
                    self.update_menu_item_status(ui, VideoSizeFullScreen);
                }
            }

            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_menu("Audio") {
                ui.menu_item_config("Enabled")
                    .shortcut("Ctrl+A")
                    .selected(self.is_menu_bar_item_selected(AudioEnabled))
                    .build();

                self.toggle_menu_bar_item_if_clicked(ui, AudioEnabled);
                ui.child_window("child")
                    .size([190.0, ui.current_font_size() + 3.0])
                    .border(false)
                    .scroll_bar(false)
                    .build(|| {
                        ui.slider_config("Volume", 0, 100)
                            .display_format("%d%%")
                            .build(&mut self.audio_volume);
                    });

                ui.separator();

                ui.menu_item_config("Increase").shortcut("=").build();
                self.update_menu_item_status(ui, VolumeIncrease);

                ui.menu_item_config("Decrease").shortcut("-").build();
                self.update_menu_item_status(ui, VolumeDecrease);
            }

            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_menu("Controllers") {
                unsafe {
                    imgui_sys::igSetNextWindowSize(
                        imgui_sys::ImVec2 { x: 230.0, y: 230.0 },
                        imgui::Condition::Always as i32,
                    );
                }
                #[allow(clippy::redundant_pattern_matching)]
                if let Some(_) = ui.begin_menu("Setup") {
                    self.build_controllers_setup_window(ui);
                    self.controllers_setup = true;
                } else {
                    self.controllers_setup = false;
                }
            }
        }

        font.pop();

        if !self.controllers_setup {
            self.controller_configs[0].pending_key_select = None;
            self.controller_configs[1].pending_key_select = None;
        }
    }

    fn build_emulation_window(&mut self, ui: &imgui::Ui) {
        let style = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
        let vertical_offset = if self.build_menu_bar {
            MENU_BAR_HEIGHT as f32
        } else {
            0.0
        };

        ui.window("emulation")
            .position([0.0, vertical_offset], imgui::Condition::Always)
            .no_decoration()
            .size(self.video_size, imgui::Condition::Always)
            .scroll_bar(false)
            .bring_to_front_on_focus(false)
            .build(|| {
                imgui::Image::new(self.emulation_texture, self.video_size).build(ui);
                self.mouse_click = None;
                if ui.is_window_hovered() {
                    if ui.is_mouse_clicked(imgui::MouseButton::Left) {
                        let io = ui.io();
                        let mouse_pos = io.mouse_pos; // [f32;
                        let window_pos = ui.window_pos();
                        let rel_pos = [mouse_pos[0] - window_pos[0], mouse_pos[1] - window_pos[1]];
                        let tex_x =
                            (rel_pos[0] / self.video_size[0] * FRAME_WIDTH as f32).floor() as usize;
                        let tex_y = (rel_pos[1] / self.video_size[1] * FRAME_HEIGHT as f32).floor()
                            as usize;
                        self.mouse_click = Some(MouseClick { x: tex_x, y: tex_y });
                        println!("Clicked! {} {}", tex_x, tex_y);
                    }
                }
            });
        style.pop();
    }

    fn build_fps_counter(&self, ui: &imgui::Ui) {
        use imgui::ImString;
        let font = ui.push_font(self.fonts[GuiFont::FpsCounter as usize]);
        let text = {
            format!(
                "FPS {}/{}",
                self.io_control.current_fps, self.io_control.target_fps
            )
        };

        let [video_width, _]: [f32; 2] = self.video_size;
        let im_text = ImString::new(&text);
        let text_size = ui.calc_text_size::<&imgui::ImStr>(im_text.as_ref());
        ui.window("fps")
            .position(
                [video_width - text_size[0], MENU_BAR_HEIGHT as _],
                imgui::Condition::Always,
            )
            .size(text_size, imgui::Condition::Always)
            .bg_alpha(0.0)
            .no_decoration()
            .build(|| {
                ui.text(text);
            });
        font.pop();
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

    fn build_controller_setup_for_player(&mut self, player_index: usize, ui: &imgui::Ui) {
        let controller_config = &mut self.controller_configs[player_index];
        let items = ["Standard", "Zapper"];
        if player_index == 1 {
            let mut current =
                controllers::ControllerType::from(controller_config.controller_type) as usize - 1;
            let text = String::from("Controller:");
            ui.text(text);
            ui.same_line();
            ui.combo_simple_string("Controller", &mut current, &items);
            ui.separator();
        }
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
            ui.same_line();
            ui.text(text);
        }
    }
    fn build_controllers_setup_window(&mut self, ui: &imgui::Ui) {
        let font = ui.push_font(self.fonts[GuiFont::MenuBar as usize]);
        let style = ui.push_style_var(imgui::StyleVar::WindowBorderSize(2.0));

        if let Some(tab_bar) = imgui::TabBar::new("Players").begin(ui) {
            if self.controller_configs[1].pending_key_select.is_none() {
                if let Some(player_1_tab) = imgui::TabItem::new("Player 1").begin(ui) {
                    self.build_controller_setup_for_player(0, ui);
                    player_1_tab.end();
                }
            }
            if self.controller_configs[0].pending_key_select.is_none() {
                if let Some(player_2_tab) = imgui::TabItem::new("Player 2").begin(ui) {
                    self.build_controller_setup_for_player(1, ui);
                    player_2_tab.end();
                }
            }
            tab_bar.end();
        }
        style.pop();
        font.pop();
    }

    pub(super) fn build(&mut self, ui: &mut imgui::Ui) {
        self.build_menu_bar = !(self.video_size_control == VideoSizeControl::FullScreen);

        let style_rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
        let style_border = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
        let style_padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));

        if self.build_menu_bar {
            self.build_menu_bar(ui);
        }
        self.build_emulation_window(ui);
        self.build_fps_counter(ui);
        self.build_load_nes_file_explorer();
        self.build_save_state_file_explorer();
        self.build_load_state_file_explorer();

        style_padding.pop();
        style_border.pop();
        style_rounding.pop();
    }
}
