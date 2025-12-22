use std::ops::RangeInclusive;

use super::{MenuBarItem, MENU_BAR_HEIGHT};
use crate::{
    common,
    io::{IOControl, FRAME_HEIGHT, FRAME_WIDTH},
};

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
macro_rules! with_font {
    ($font:expr, $ui:ident, $code:block) => {{
        // Inner scope for RAII guard
        let result = {
            let font_token = $ui.push_font($font);
            let inner_result = { $code }; // execute code while font is active
            inner_result
            // font_token dropped here at end of inner scope
        };
        result
    }};
}

macro_rules! with_token {
    ($ui:expr, $token_function:ident, ($($arg:expr),*), $code:expr) => {{
        if let Some(token) = $ui.$token_function($($arg),*) {
            $code
            token.end();
            true
        } else {
            false
        }
    }};
}
macro_rules! with_styles {
    ($ui:expr, ($($style:expr),*), $code:block) => {{
      let _tokens = (
        $(
            $ui.push_style_var($style),
        )*
    );
        $code
}};
}
macro_rules! create_simple_window {
    ($ui:expr, $name:expr, $position:expr, $size:expr, $condition_pos:expr, $condition_size:expr) => {{
        imgui::Window::new($ui, $name)
            .scrollable(false)
            .no_decoration()
            .position($position, $condition_pos)
            .size($size, $condition_size)
    }};
}

macro_rules! create_unmovable_simple_window {
    ($ui:expr, $name:expr, $position:expr, $size:expr) => {{
        create_simple_window!(
            $ui,
            $name,
            $position,
            $size,
            imgui::Condition::Always,
            imgui::Condition::Always
        )
    }};
}

macro_rules! create_menu_item {
    ($ui:expr, $name:tt, $shortcut:tt) => {{
        imgui::MenuItem::new($name, $ui)
            .selected(false)
            .enabled(true)
        //  .shortcut($shortcut)
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
        Self { key }
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
        let nes_file_label = ImString::new("nes_file");
        let open_nes_file_label = ImString::new("Open NES file");
        let open_nes_file_filters_label = ImString::new(".nes,.NES");
        let save_state_label = ImString::new("save_state");
        let save_state_title = ImString::new("Save Emulation state");
        let save_state_filters = ImString::new(".nes,.NES");
        let load_state_label = ImString::new("load_state");
        let load_state_title = ImString::new("Load Emulation state");
        let load_state_filters = ImString::new(".nes,.NES");
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
                &nes_file_label.as_ref(),
                &open_nes_file_label.as_ref(),
                &open_nes_file_filters_label.as_ref(),
            ),
            fd_save_state: create_file_dialog(
                &save_state_label.as_ref(),
                &save_state_title.as_ref(),
                &save_state_filters.as_ref(),
            ),
            fd_load_state: create_file_dialog(
                &load_state_label.as_ref(),
                &load_state_title.as_ref(),
                &load_state_filters.as_ref(),
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

    fn add_menu_item(&mut self, ui: &mut imgui::Ui, name: &str, shortcut: &str, item: MenuBarItem) {
        if imgui::MenuItem::new(name, ui).shortcut(shortcut).build() {
            self.update_menu_item_status(ui, item);
        }
    }

    fn build_menu_bar(&mut self, ui: &mut imgui::Ui) {
        use MenuBarItem::*;

        // -------------------------
        // PHASE 1: Render menus & collect clicks
        // -------------------------
        let mut clicked_file = Vec::new();
        let mut clicked_emulation = Vec::new();
        let mut clicked_video = Vec::new();
        let mut clicked_audio = Vec::new();
        let mut controllers_setup_needed = false;

        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            let _font_token = ui.push_font(self.fonts[GuiFont::MenuBar as usize]);

            // ---- FILE MENU ----
            if let Some(file_menu) = ui.begin_menu("File") {
                if ui
                    .menu_item_config("Load Nes File")
                    .shortcut("Ctrl+O")
                    .build()
                {
                    clicked_file.push(LoadNesFile);
                }
                if ui.menu_item_config("Save State").shortcut("Ctrl+S").build() {
                    clicked_file.push(SaveState);
                }
                if ui.menu_item_config("Load State").shortcut("Ctrl+L").build() {
                    clicked_file.push(LoadState);
                }
                if ui.menu_item_config("Quit").shortcut("Alt+F4").build() {
                    clicked_file.push(Quit);
                }
                drop(file_menu);
            }

            // ---- EMULATION MENU ----
            if let Some(emulation_menu) = ui.begin_menu("Emulation") {
                if ui
                    .menu_item_config("Power Cycle")
                    .shortcut("Ctrl+R")
                    .build()
                {
                    clicked_emulation.push(PowerCycle);
                }
                if ui
                    .menu_item_config("Pause")
                    .shortcut("Ctrl+P")
                    .selected(self.pause)
                    .build()
                {
                    clicked_emulation.push(Pause);
                }

                if let Some(speed_menu) = ui.begin_menu("Speed") {
                    let target_fps = self.io_control.target_fps;
                    let is_speed_selected = |fps: u16| fps == target_fps;

                    if ui
                        .menu_item_config("Normal")
                        .selected(is_speed_selected(common::DEFAULT_FPS))
                        .build()
                    {
                        clicked_emulation.push(SpeedNormal);
                    }
                    if ui
                        .menu_item_config("Double")
                        .selected(is_speed_selected(common::DOUBLE_FPS))
                        .build()
                    {
                        clicked_emulation.push(SpeedDouble);
                    }
                    if ui
                        .menu_item_config("Half")
                        .selected(is_speed_selected(common::HALF_FPS))
                        .build()
                    {
                        clicked_emulation.push(SpeedHalf);
                    }

                    ui.separator();

                    if ui.menu_item_config("Increase").shortcut("Ctrl+=").build() {
                        clicked_emulation.push(SpeedIncrease);
                    }
                    if ui.menu_item_config("Decrease").shortcut("Ctrl+-").build() {
                        clicked_emulation.push(SpeedDecrease);
                    }

                    drop(speed_menu);
                }

                drop(emulation_menu);
            }

            // ---- VIDEO MENU ----
            if let Some(video_menu) = ui.begin_menu("Video") {
                if let Some(size_menu) = ui.begin_menu("Size") {
                    if ui
                        .menu_item_config("200%")
                        .shortcut("F9")
                        .selected(self.video_size_control == VideoSizeControl::Double)
                        .build()
                    {
                        clicked_video.push(VideoSizeDouble);
                    }
                    if ui
                        .menu_item_config("300%")
                        .shortcut("F10")
                        .selected(self.video_size_control == VideoSizeControl::Triple)
                        .build()
                    {
                        clicked_video.push(VideoSizeTriple);
                    }
                    if ui
                        .menu_item_config("400%")
                        .shortcut("F11")
                        .selected(self.video_size_control == VideoSizeControl::Quadrupal)
                        .build()
                    {
                        clicked_video.push(VideoSizeQuadrupal);
                    }
                    if ui
                        .menu_item_config("Full screen")
                        .shortcut("F12")
                        .selected(self.video_size_control == VideoSizeControl::FullScreen)
                        .build()
                    {
                        clicked_video.push(VideoSizeFullScreen);
                    }
                    drop(size_menu);
                }
                drop(video_menu);
            }

            // ---- AUDIO MENU ----
            if let Some(audio_menu) = ui.begin_menu("Audio") {
                if ui
                    .menu_item_config("Enabled")
                    .shortcut("Ctrl+A")
                    .selected(self.is_menu_bar_item_selected(AudioEnabled))
                    .build()
                {
                    clicked_audio.push(AudioEnabled);
                }

                imgui::ChildWindow::new(ui, "child")
                    .size([190.0, ui.current_font_size() + 3.0])
                    .border(false)
                    .scroll_bar(false)
                    .build(|| {
                        ui.slider_config("Volume", 0, 100)
                            .display_format("%d%%")
                            .build(&mut self.audio_volume);
                    });

                ui.separator();

                if ui.menu_item_config("Increase").shortcut("=").build() {
                    clicked_audio.push(VolumeIncrease);
                }
                if ui.menu_item_config("Decrease").shortcut("-").build() {
                    clicked_audio.push(VolumeDecrease);
                }

                drop(audio_menu);
            }

            // ---- CONTROLLERS MENU ----
            if let Some(_controllers_menu) = ui.begin_menu("Controllers") {
                unsafe {
                    imgui_sys::igSetNextWindowSize(
                        imgui_sys::ImVec2 { x: 230.0, y: 230.0 },
                        imgui::Condition::Always as i32,
                    );
                }

                // Mark that controllers setup window should be built after RAII guard
                controllers_setup_needed = true;

                drop(_controllers_menu);
            }

            drop(_font_token);
            drop(menu_bar);
        }

        // -------------------------
        // PHASE 2: Mutably update state
        // -------------------------
        for item in clicked_file {
            self.update_menu_item_status(ui, item);
        }

        for item in clicked_emulation {
            self.update_menu_item_status(ui, item);
        }

        for item in clicked_video {
            self.update_menu_item_status(ui, item);
        }

        for item in clicked_audio {
            match item {
                AudioEnabled => self.toggle_menu_bar_item_if_clicked(ui, AudioEnabled),
                _ => self.update_menu_item_status(ui, item),
            }
        }

        if controllers_setup_needed {
            self.build_controllers_setup_window(ui);

            if !self.controllers_setup {
                self.controller_configs[0].pending_key_select = None;
                self.controller_configs[1].pending_key_select = None;
            }
        }
    }

    fn build_emulation_window(&self, ui: &mut imgui::Ui) {
        with_styles!(ui, (imgui::StyleVar::WindowBorderSize(0.0)), {
            let vertical_offset = if self.build_menu_bar {
                MENU_BAR_HEIGHT as f32
            } else {
                0.0
            };
            create_unmovable_simple_window!(
                ui,
                "emulation",
                [0.0, vertical_offset],
                self.video_size
            )
            .bring_to_front_on_focus(false)
            .build(|| {
                imgui::Image::new(self.emulation_texture, self.video_size).build(ui);
            });
        });
    }
    fn build_fps_counter(&self, ui: &mut imgui::Ui) {
        use imgui::ImString;

        // -------------------------
        // Push font and prepare text
        // -------------------------
        let text = {
            // Push font temporarily
            let _font_token = ui.push_font(self.fonts[GuiFont::FpsCounter as usize]);
            format!(
                "FPS {}/{}",
                self.io_control.current_fps, self.io_control.target_fps
            )
            // _font_token drops here
        };

        let [video_width, _]: [f32; 2] = self.video_size;

        // Create a long-lived ImString for borrowing
        let im_text = ImString::new(&text);
        let text_size = ui.calc_text_size::<&imgui::ImStr>(im_text.as_ref());

        // -------------------------
        // Build unmovable window
        // -------------------------
        create_unmovable_simple_window!(
            ui,
            "fps",
            [video_width - text_size[0], MENU_BAR_HEIGHT as _],
            text_size
        )
        .bg_alpha(0.0)
        .build(|| {
            ui.text(text);
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
            ui.same_line();
            ui.text(text);
        }
    }
    fn build_controllers_setup_window(&mut self, ui: &mut imgui::Ui) {
        use imgui::{StyleVar, TabBar, TabItem};

        // Phase 1: Determine which tabs should be drawn (immutable only)
        let tabs_to_draw: Vec<usize> = {
            let _style_token = ui.push_style_var(StyleVar::WindowBorderSize(2.0));

            let mut tabs = Vec::new();
            if let Some(tab_bar) = TabBar::new("Players").begin(ui) {
                if self.controller_configs[1].pending_key_select.is_none() {
                    tabs.push(0);
                }
                if self.controller_configs[0].pending_key_select.is_none() {
                    tabs.push(1);
                }
                tab_bar.end();
            }

            // _style_token dropped here
            tabs
        };

        // Phase 2: Draw tab titles, push font for visuals only
        let _ = {
            let _font_token = ui.push_font(self.fonts[GuiFont::MenuBar as usize]);
            for player_idx in &tabs_to_draw {
                let _ = TabItem::new(format!("Player {}", player_idx + 1)).begin(ui);
                // Drop TabItemToken immediately
            }
            // _font_token dropped here
        };

        // Phase 3: Mutably borrow UI for controller setup
        for player_idx in tabs_to_draw {
            self.build_controller_setup_for_player(player_idx, ui);
        }
    }

    pub(super) fn build(&mut self, ui: &mut imgui::Ui) {
        self.build_menu_bar = !(self.video_size_control == VideoSizeControl::FullScreen);

        // Phase 1: Push styles for the window
        {
            let _style_rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
            let _style_border = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
            let _style_padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));

            // At this point, only immutable borrows are active
            // We can draw UI that does not require mutable borrows overlapping with styles
        } // style tokens dropped here

        // Phase 2: Build the menu bar separately (mutable borrow inside)
        if self.build_menu_bar {
            self.build_menu_bar(ui);
        }

        // Phase 3: Build other windows / UI components (mutable borrow safely)
        self.build_emulation_window(ui);
        self.build_fps_counter(ui);

        // Phase 4: Build file explorers (these probably do not need &mut Ui simultaneously)
        self.build_load_nes_file_explorer();
        self.build_save_state_file_explorer();
        self.build_load_state_file_explorer();
    }
}
