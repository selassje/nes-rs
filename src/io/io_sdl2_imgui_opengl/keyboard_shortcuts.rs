use super::MenuBarItem;
use sdl2::keyboard::Mod;
use sdl2::keyboard::Scancode;
use KeyboardShortcut::*;

enum KeyboardShortcut {
    Single(Scancode),
    LeftCtrl(Scancode),
}

fn shortcut_to_menu_bar_item(key: KeyboardShortcut) -> Option<MenuBarItem> {
    match key {
        LeftCtrl(Scancode::R) => Some(MenuBarItem::PowerCycle),
        LeftCtrl(Scancode::O) => Some(MenuBarItem::LoadNesFile),
        LeftCtrl(Scancode::P) => Some(MenuBarItem::Pause),
        LeftCtrl(Scancode::Equals) => Some(MenuBarItem::SpeedIncrease),
        LeftCtrl(Scancode::Minus) => Some(MenuBarItem::SpeedDecrease),
        LeftCtrl(Scancode::A) => Some(MenuBarItem::AudioEnabled),
        LeftCtrl(Scancode::C) => Some(MenuBarItem::ControllersSetup),
        Single(Scancode::Minus) => Some(MenuBarItem::VolumeDecrease),
        Single(Scancode::Equals) => Some(MenuBarItem::VolumeIncrease),
        Single(Scancode::F8) => Some(MenuBarItem::VideoSizeNormal),
        Single(Scancode::F9) => Some(MenuBarItem::VideoSizeDouble),
        Single(Scancode::F10) => Some(MenuBarItem::VideoSizeTriple),
        Single(Scancode::F11) => Some(MenuBarItem::VideoSizeQuadrupal),
        Single(Scancode::F12) => Some(MenuBarItem::VideoSizeFullScreen),
        _ => None,
    }
}

fn is_shortcut(scancode: Scancode, key_mod: Mod) -> Option<MenuBarItem> {
    let is_left_ctrl = Mod::LCTRLMOD & key_mod == Mod::LCTRLMOD;
    if is_left_ctrl {
        return shortcut_to_menu_bar_item(LeftCtrl(scancode));
    }
    return shortcut_to_menu_bar_item(Single(scancode));
}
#[derive(Default)]
pub(super) struct KeyboardShortcuts {
    menu_bar_item_selected: [bool; MenuBarItem::None as usize],
}
impl KeyboardShortcuts {
    pub(super) fn is_menu_bar_item_selected(&self, item: MenuBarItem) -> bool {
        self.menu_bar_item_selected[item as usize]
    }

    pub fn update(&mut self, scancode: Scancode, key_mod: Mod) {
        if let Some(item) = is_shortcut(scancode, key_mod) {
            self.menu_bar_item_selected[item as usize] = true;
        }
    }
}
