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
        Single(Scancode::Escape) => Some(MenuBarItem::Quit),
        LeftCtrl(Scancode::R) => Some(MenuBarItem::PowerCycle),
        LeftCtrl(Scancode::O) => Some(MenuBarItem::LoadNesFile),
        LeftCtrl(Scancode::P) => Some(MenuBarItem::Pause),
        LeftCtrl(Scancode::Equals) => Some(MenuBarItem::SpeedIncrease),
        LeftCtrl(Scancode::Minus) => Some(MenuBarItem::SpeedDecrease),
        _ => None,
    }
}

fn is_shortcut(scancode: Scancode, key_mod: Mod) -> Option<MenuBarItem> {
    let single = shortcut_to_menu_bar_item(Single(scancode));
    if single.is_some() {
        return single;
    } else if Mod::LCTRLMOD & key_mod == Mod::LCTRLMOD {
        return shortcut_to_menu_bar_item(LeftCtrl(scancode));
    } else {
        return None;
    }
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
