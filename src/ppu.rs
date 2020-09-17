use crate::io_sdl::SCREEN;
use crate::memory::VideoMemory;
use crate::ram_ppu::*;
use crate::{
    colors::{ColorMapper, DefaultColorMapper, RgbColor},
    screen::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
};

use std::{cell::RefCell, default::Default, fmt::Display, rc::Rc};

enum ControlRegisterFlag {
    BaseNametableAddress = 0b00000011,
    VramIncrement = 0b00000100,
    SpritePattern8x8 = 0b00001000,
    BackgroundPatternTableAddress = 0b00010000,
    SpriteSize = 0b00100000,
    PpuMasterSlaveSelect = 0b01000000,
    GenerateNMI = 0b10000000,
}
const TILE_SIDE_LENGTH: usize = 8;

const PPU_CYCLES_PER_SCANLINE: u16 = 341;
const PRE_RENDER_SCANLINE: i16 = -1;
const FIRST_VISIBLE_SCANLINE: i16 = 0;
const LAST_VISIBLE_SCANLINE: i16 = 239;
const POST_RENDER_SCANLINE: i16 = 240;
const VBLANK_START_SCANLINE: i16 = 241;

const ACTIVE_PIXELS_START: u16 = 4;
const ACTIVE_PIXELS_END: u16 = ACTIVE_PIXELS_START + DISPLAY_WIDTH as u16 - 1;

struct ControlRegister {
    value: u8,
}

impl ControlRegister {
    fn get_base_nametable_index(&self) -> u8 {
        self.value & ControlRegisterFlag::BaseNametableAddress as u8
    }

    fn get_vram_increment(&self) -> u16 {
        if self.value & ControlRegisterFlag::VramIncrement as u8 == 0 {
            1
        } else {
            32
        }
    }

    fn get_sprite_pattern_table_index_for_8x8_mode(&self) -> u8 {
        (self.value & ControlRegisterFlag::SpritePattern8x8 as u8) >> 4
    }

    fn get_background_pattern_table_index(&self) -> u8 {
        (self.value & ControlRegisterFlag::BackgroundPatternTableAddress as u8) >> 4
    }

    fn get_sprite_size_height(&self) -> u8 {
        let bit = (self.value & ControlRegisterFlag::SpriteSize as u8) >> 5;
        (1 + bit) * 8
    }

    fn is_read_backdrop_color_from_ext_enabled(&self) -> bool {
        (self.value & ControlRegisterFlag::PpuMasterSlaveSelect as u8) != 0
    }

    fn is_generate_nmi_enabled(&self) -> bool {
        (self.value & ControlRegisterFlag::GenerateNMI as u8) != 0
    }
}

enum MaskRegisterFlag {
    GrayScale = 0b00000001,
    ShowBackgroundInLeftMost8Pixels = 0b00000010,
    ShowSpritesdInLeftMost8Pixels = 0b00000100,
    ShowBackground = 0b00001000,
    ShowSprites = 0b00010000,
    EmphasizeRed = 0b00100000,
    EmphasizeGreen = 0b01000000,
    EmphasizeBlue = 0b10000000,
}

struct MaskRegister {
    value: u8,
}

impl MaskRegister {
    fn is_flag_enabled(&self, flag: MaskRegisterFlag) -> bool {
        (self.value & flag as u8) != 0
    }
}

enum StatusRegisterFlag {
    SpriteOverflow = 0b00100000,
    Sprite0Hit = 0b01000000,
    VerticalBlankStarted = 0b10000000,
}

#[derive(Copy, Clone, Default)]
struct StatusRegister {
    value: u8,
}

impl StatusRegister {
    fn set_flag(&mut self, flag: StatusRegisterFlag, enable: bool) {
        if enable {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }

    fn get_flag(&self, flag: StatusRegisterFlag) -> bool {
        (self.value & flag as u8) != 0
    }
}

#[derive(Copy, Clone, Default)]
struct Tile {
    data: [u8; 16],
}

impl Tile {
    fn get_color_index(&self, x: usize, y: usize) -> usize {
        assert!(y < 8);
        let shift = 7 - x;
        let mask = 1 << shift;
        let lo_bit = (mask & self.data[y]) >> shift;
        let hi_bit = (mask & self.data[y + 8]) >> shift;
        (2 * hi_bit + lo_bit) as usize
    }
}

struct PatternTable {
    tiles: [Tile; 256],
}

impl Default for PatternTable {
    fn default() -> Self {
        PatternTable {
            tiles: [Default::default(); 256],
        }
    }
}

impl PatternTable {
    fn new(vram: &dyn VideoMemory, table_index: u8) -> Self {
        let mut pattern_table: PatternTable = Default::default();
        for i in 0..pattern_table.tiles.len() {
            pattern_table.tiles[i] = Tile {
                data: vram.get_pattern_table_tile_data(table_index, i as u8),
            }
        }
        pattern_table
    }
}
type Palette = [RgbColor; 4];
type Palettes = [Palette; 4];

#[derive(Copy, Clone, Default)]
struct Sprite {
    oam_index: u8,
    data: [u8; 4],
}

type Sprites = Vec<Sprite>;

impl Sprite {
    fn get_y(&self) -> u8 {
        ((self.data[0] as u16) % 256) as u8
    }

    fn get_x(&self) -> u8 {
        self.data[3]
    }

    fn get_tile_index(&self, is_8x16_mode: bool) -> u8 {
        if is_8x16_mode {
            (self.data[1] >> 1) << 1
        } else {
            self.data[1]
        }
    }

    fn get_pattern_table_index_for_8x16_mode(&self) -> u8 {
        self.data[1] & 1
    }

    fn get_palette_index(&self) -> u8 {
        self.data[2] & 0b00000011
    }

    fn if_draw_in_front(&self) -> bool {
        self.data[2] & 0b00100000 == 0
    }

    fn if_flip_horizontally(&self) -> bool {
        self.data[2] & self.data[2] & 0b01000000 != 0
    }

    fn if_flip_vertically(&self) -> bool {
        self.data[2] & self.data[2] & 0b10000000 != 0
    }
}

type OAM = [u8; 256];

type VRAMAddressFlag = (u16, u16);
const COARSE_X: VRAMAddressFlag = (0b00000000_00011111, 0);
const COARSE_Y: VRAMAddressFlag = (0b00000011_11100000, 5);
const NM_TABLE: VRAMAddressFlag = (0b00001100_00000000, 10);
const NM_TABLE_X: VRAMAddressFlag = (0b00000100_00000000, 10);
const NM_TABLE_Y: VRAMAddressFlag = (0b00001000_00000000, 11);
const FINE_Y: VRAMAddressFlag = (0b01110000_00000000, 12);
const BIT_14: VRAMAddressFlag = (0b01000000_00000000, 14);
const BITS_8_13: VRAMAddressFlag = (0b00111111_00000000, 8);
const LOW_BYTE: VRAMAddressFlag = (0b00000000_11111111, 0);

#[derive(Copy, Clone, Default)]
struct VRAMAddress {
    address: u16,
}

impl VRAMAddress {
    fn get(&self, flag: VRAMAddressFlag) -> u16 {
        let (mask, shift) = flag;
        (self.address & mask) >> shift
    }
    fn set(&mut self, flag: VRAMAddressFlag, value: u16) {
        let (mask, shift) = flag;
        self.address &= !mask;
        self.address |= value << shift;
    }

    fn toggle_x_name_table_index(&mut self) {
        let (nm_table_mask, _) = NM_TABLE_X;
        self.address ^= nm_table_mask;
    }

    fn toggle_y_name_table_index(&mut self) {
        let (nm_table_mask, _) = NM_TABLE_Y;
        self.address ^= nm_table_mask;
    }

    fn inc_coarse_x(&mut self) {
        let mut coarse_x = self.get(COARSE_X);
        if coarse_x < 31 {
            coarse_x += 1;
        } else {
            coarse_x = 0;
            self.toggle_x_name_table_index();
        }
        self.set(COARSE_X, coarse_x);
    }

    fn inc_y(&mut self) {
        let mut fine_y = self.get(FINE_Y);
        let mut coarse_y = self.get(COARSE_Y);
        if fine_y < 7 {
            fine_y += 1;
        } else {
            fine_y = 0;
            if coarse_y == 29 {
                coarse_y = 0;
                self.toggle_y_name_table_index();
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }
        }
        self.set(FINE_Y, fine_y);
        self.set(COARSE_Y, coarse_y);
    }
}

impl Display for VRAMAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "X {} Y({} {})",
            self.get(COARSE_X),
            self.get(COARSE_Y),
            self.get(FINE_Y)
        )
    }
}

pub struct PPU {
    vram: Rc<RefCell<dyn VideoMemory>>,
    control_reg: ControlRegister,
    mask_reg: MaskRegister,
    status_reg: StatusRegister,
    oam_address: u8,
    oam: OAM,
    pattern_tables: [PatternTable; 2],
    ppu_cycles: u16,
    scanline: i16,
    scanline_sprites: Vec<Sprite>,
    frame: u128,
    color_mapper: Box<dyn ColorMapper>,
    write_toggle: bool,
    nmi_supressed: bool,
    nmi_triggered: bool,
    vbl_flag_supressed: bool,
    vram_address: VRAMAddress,
    t_vram_address: VRAMAddress,
    fine_x_scroll: u8,
}

impl PPU {
    pub fn new(vram: Rc<RefCell<dyn VideoMemory>>) -> PPU {
        PPU {
            vram: vram,
            control_reg: ControlRegister { value: 0 },
            mask_reg: MaskRegister { value: 0 },
            status_reg: StatusRegister { value: 0 },
            oam_address: 0,
            oam: [0; 256],
            pattern_tables: Default::default(),
            ppu_cycles: 27,
            scanline: 0,
            scanline_sprites: Vec::new(),
            frame: 1,
            color_mapper: Box::new(DefaultColorMapper {}),
            vram_address: Default::default(),
            t_vram_address: Default::default(),
            fine_x_scroll: 0,
            write_toggle: false,
            nmi_supressed: false,
            nmi_triggered: false,
            vbl_flag_supressed: false,
        }
    }

    pub fn reset(&mut self) {
        self.control_reg.value = 0;
        self.mask_reg.value = 0;
        self.status_reg.value = 0;
        self.oam_address = 0;
        self.oam = [0; 256];
        self.pattern_tables = [
            PatternTable::new(&*self.vram.borrow(), 0),
            PatternTable::new(&*self.vram.borrow(), 1),
        ];
        self.ppu_cycles = 0;
        self.scanline = 0;
        self.scanline_sprites.clear();
        self.frame = 1;
        self.vram_address = Default::default();
        self.t_vram_address = Default::default();
        self.fine_x_scroll = 0;
        self.write_toggle = false;
        self.nmi_supressed = false;
        self.nmi_triggered = false;
        self.vbl_flag_supressed = false;
    }

    fn render_pixel(&mut self) {
        let background_palettes = self.get_palettes(true);
        let sprite_palettes = self.get_palettes(false);

        let x = self.ppu_cycles - ACTIVE_PIXELS_START;
        let (bg_color_index, bg_palette_index) = self.get_background_color_index(x as usize);
        let (sprite_color_index, sprite) = self.get_sprite_color_index(x as u8);

        let (color, is_sprite0_hit_detected) = self
            .determine_pixel_color_and_check_for_sprite0_hit(
                bg_color_index as u8,
                sprite_color_index,
                &sprite,
                &background_palettes[bg_palette_index as usize],
                &sprite_palettes,
            );

        unsafe {
            SCREEN[x as usize][self.scanline as usize] = color;
        }

        if !self.status_reg.get_flag(StatusRegisterFlag::Sprite0Hit)
            && x < 255
            && is_sprite0_hit_detected
        {
            self.status_reg
                .set_flag(StatusRegisterFlag::Sprite0Hit, true);
        }
    }

    pub fn run_single_ppu_cycle(&mut self) -> bool {
        if self.ppu_cycles > ACTIVE_PIXELS_START && self.ppu_cycles <= ACTIVE_PIXELS_END {
            let x = self.ppu_cycles - ACTIVE_PIXELS_START;
            if self.is_rendering_in_progress() && (x as usize % TILE_SIDE_LENGTH == 0) {
                self.vram_address.inc_coarse_x();
            }
        }

        if self.ppu_cycles == ACTIVE_PIXELS_END + 1 {
            if self.is_rendering_in_progress() {
                self.vram_address.inc_y();
                self.vram_address
                    .set(COARSE_X, self.t_vram_address.get(COARSE_X));
                self.vram_address
                    .set(NM_TABLE_X, self.t_vram_address.get(NM_TABLE_X));
            }
        }

        match self.scanline {
            PRE_RENDER_SCANLINE => match self.ppu_cycles {
                1 => {
                    self.nmi_supressed = false;
                    self.vbl_flag_supressed = false;

                    self.status_reg
                        .set_flag(StatusRegisterFlag::VerticalBlankStarted, false);
                    self.status_reg
                        .set_flag(StatusRegisterFlag::SpriteOverflow, false);
                    self.status_reg
                        .set_flag(StatusRegisterFlag::Sprite0Hit, false);
                }
                280 => {
                    if self.is_rendering_in_progress() {
                        self.vram_address
                            .set(FINE_Y, self.t_vram_address.get(FINE_Y));
                        self.vram_address
                            .set(COARSE_Y, self.t_vram_address.get(COARSE_Y));
                        self.vram_address
                            .set(NM_TABLE_Y, self.t_vram_address.get(NM_TABLE_Y));
                    }
                }

                339 => {
                    if self.is_rendering_enabled() && self.frame % 2 == 1 {
                        self.ppu_cycles += 1;
                    }
                }

                _ => (),
            },
            FIRST_VISIBLE_SCANLINE..=LAST_VISIBLE_SCANLINE => match self.ppu_cycles {
                0 => {
                    self.scanline_sprites.clear();
                    let (sprites, is_overflow_detected) =
                        self.get_sprites_for_scanline_and_check_for_overflow();
                    self.scanline_sprites = sprites;

                    if !self.status_reg.get_flag(StatusRegisterFlag::SpriteOverflow)
                        && is_overflow_detected
                    {
                        self.status_reg
                            .set_flag(StatusRegisterFlag::SpriteOverflow, true);
                    }
                }

                ACTIVE_PIXELS_START..=ACTIVE_PIXELS_END => {
                    if self.is_rendering_in_progress() {
                        self.render_pixel();
                    }
                }
                _ => (),
            },
            VBLANK_START_SCANLINE => {
                if self.ppu_cycles == 1 {
                    if !self.vbl_flag_supressed {
                        self.status_reg
                            .set_flag(StatusRegisterFlag::VerticalBlankStarted, true);
                        //println!("Setting VBLANK");
                        self.nmi_triggered = self.control_reg.is_generate_nmi_enabled();
                    }
                }
            }

            _ => {}
        };

        self.ppu_cycles += 1;
        if self.ppu_cycles == PPU_CYCLES_PER_SCANLINE {
            self.ppu_cycles = 0;
            self.scanline += 1;
            if self.scanline == 261 {
                self.scanline = PRE_RENDER_SCANLINE;
                if self.frame == std::u128::MAX {
                    self.frame = 0;
                } else {
                    self.frame += 1;
                }
            }
        };
        let nmi_triggered = self.nmi_triggered;
        if nmi_triggered {
            self.nmi_triggered = false;
        }
        nmi_triggered && !self.nmi_supressed
    }

    fn get_scrolled_x(&self, x: usize) -> usize {
        ((self.fine_x_scroll as usize + x)
            + (self.vram_address.get(COARSE_X) * TILE_SIDE_LENGTH as u16) as usize
            + (self.vram_address.get(NM_TABLE_X) as usize) * DISPLAY_WIDTH as usize)
            % (2 * DISPLAY_WIDTH)
    }

    fn get_scrolled_y(&self) -> usize {
        ((self.vram_address.get(FINE_Y) as usize)
            + (self.vram_address.get(COARSE_Y) * (TILE_SIDE_LENGTH as u16)) as usize
            + ((self.vram_address.get(NM_TABLE_Y) as usize) * DISPLAY_HEIGHT))
            % (2 * DISPLAY_HEIGHT)
    }

    fn determine_pixel_color_and_check_for_sprite0_hit(
        &self,
        bg_color_index: u8,
        sprite_color_index: u8,
        sprite: &Sprite,
        bg_pallete: &Palette,
        sprite_palletes: &Palettes,
    ) -> (RgbColor, bool) {
        let bg_color = bg_pallete[bg_color_index as usize];
        let sprite_color =
            sprite_palletes[sprite.get_palette_index() as usize][sprite_color_index as usize];
        let mut final_color = bg_pallete[0];
        let mut sprite0_hit = false;
        if bg_color_index != 0 && sprite_color_index == 0 {
            final_color = bg_color;
        } else if bg_color_index == 0 && sprite_color_index != 0 {
            final_color = sprite_color;
        } else if bg_color_index != 0 && sprite_color_index != 0 {
            if sprite.if_draw_in_front() {
                final_color = sprite_color;
            } else {
                final_color = bg_color;
            }
            sprite0_hit = sprite.oam_index == 0;
        }
        (final_color, sprite0_hit)
    }

    fn get_background_color_index(&mut self, x: usize) -> (u8, u8) {
        let bg_pattern_table_index = self.control_reg.get_background_pattern_table_index();
        let bg_pattern_table = &self.pattern_tables[bg_pattern_table_index as usize];
        let scrolled_x = self.get_scrolled_x(x % 8);
        let scrolled_y = self.get_scrolled_y();
        let name_table_index =
            (2 * (scrolled_y / DISPLAY_HEIGHT) + (scrolled_x / DISPLAY_WIDTH)) as u8;
        let scrolled_x = scrolled_x % DISPLAY_WIDTH;
        let scrolled_y = scrolled_y % DISPLAY_HEIGHT;
        let bg_color_tile_y = (scrolled_y / 16) as u8;
        let bg_color_tile_x = (scrolled_x / 16) as u8;
        let bg_palette_index = self.vram.borrow().get_background_pallete_index(
            name_table_index,
            bg_color_tile_x,
            bg_color_tile_y,
        );
        let mut bg_color_index = 0;
        if self
            .mask_reg
            .is_flag_enabled(MaskRegisterFlag::ShowBackgroundInLeftMost8Pixels)
            || x >= 8
        {
            let bg_tile_y = (scrolled_y / 8) as u8;
            let bg_tile_x = (scrolled_x / 8) as u8;
            let bg_tile_index =
                self.vram
                    .borrow()
                    .get_nametable_tile_index(name_table_index, bg_tile_x, bg_tile_y);
            let bg_tile = bg_pattern_table.tiles[bg_tile_index as usize];
            bg_color_index = bg_tile.get_color_index(scrolled_x % 8, scrolled_y % 8);
        }
        (bg_color_index as u8, bg_palette_index)
    }

    fn get_sprite_color_index(&self, x: u8) -> (u8, Sprite) {
        if self
            .mask_reg
            .is_flag_enabled(MaskRegisterFlag::ShowSpritesdInLeftMost8Pixels)
            || x >= 8
        {
            let is_sprite_mode_8x16 = self.control_reg.get_sprite_size_height() == 16;
            for sprite in self.scanline_sprites.iter().filter(|s| s.get_y() > 0) {
                if x >= sprite.get_x() && (x as u16) < sprite.get_x() as u16 + 8 {
                    let pattern_table_index = if is_sprite_mode_8x16 {
                        sprite.get_pattern_table_index_for_8x16_mode()
                    } else {
                        self.control_reg
                            .get_sprite_pattern_table_index_for_8x8_mode()
                    };
                    let pattern_table = &self.pattern_tables[pattern_table_index as usize];
                    let mut tile_index = sprite.get_tile_index(is_sprite_mode_8x16);
                    if self.scanline as u8 > sprite.get_y() + 7 {
                        tile_index += 1;
                    }

                    if is_sprite_mode_8x16 {
                        if sprite.if_flip_vertically() {
                            if self.scanline as u8 <= sprite.get_y() + 7 {
                                tile_index += 1;
                            } else {
                                tile_index -= 1;
                            }
                        }
                    }
                    let mut x = x - sprite.get_x();
                    let mut y = (self.scanline as u8 - sprite.get_y()) % 8;
                    if sprite.if_flip_horizontally() {
                        x = 7 - x;
                    }
                    if sprite.if_flip_vertically() {
                        y = 7 - y;
                    }

                    let tile = pattern_table.tiles[tile_index as usize];
                    let color_index = tile.get_color_index(x as usize, y as usize);
                    if color_index != 0 {
                        return (color_index as u8, *sprite);
                    }
                }
            }
        }
        (0, Default::default())
    }

    fn get_sprites_for_scanline_and_check_for_overflow(&self) -> (Sprites, bool) {
        let sprites = self.oam.chunks(4).enumerate().map(|(i, s)| Sprite {
            oam_index: i as u8,
            data: [s[0], s[1], s[2], s[3]],
        });
        let sprites = sprites.filter(|sprite| {
            (self.scanline as u8) >= sprite.get_y()
                && (self.scanline as u8)
                    < sprite.get_y() + self.control_reg.get_sprite_size_height()
        });
        let if_overflow = sprites.clone().count() > 8;
        (sprites.take(8).collect(), if_overflow)
    }

    fn get_palettes(&self, for_background: bool) -> Palettes {
        let mut palletes: Palettes = Default::default();
        let raw_universal_bckg_color = self.vram.borrow().get_universal_background_color();
        for (i, p) in palletes.iter_mut().enumerate() {
            let raw_colors = if for_background {
                self.vram.borrow().get_background_palette(i as u8)
            } else {
                self.vram.borrow().get_sprite_palette(i as u8)
            };
            *p = [
                self.color_mapper
                    .map_nes_color(raw_universal_bckg_color & 0x3F),
                self.color_mapper.map_nes_color(raw_colors[0] & 0x3F),
                self.color_mapper.map_nes_color(raw_colors[1] & 0x3F),
                self.color_mapper.map_nes_color(raw_colors[2] & 0x3F),
            ];
        }
        palletes
    }

    fn is_rendering_in_progress(&self) -> bool {
        self.is_rendering_enabled() && self.is_scanline_visible_or_pre_render()
    }

    fn is_rendering_enabled(&self) -> bool {
        self.mask_reg
            .is_flag_enabled(MaskRegisterFlag::ShowBackground)
            || self.mask_reg.is_flag_enabled(MaskRegisterFlag::ShowSprites)
    }

    fn is_scanline_visible_or_pre_render(&self) -> bool {
        self.scanline >= PRE_RENDER_SCANLINE && self.scanline <= LAST_VISIBLE_SCANLINE
    }
}

impl WritePpuRegisters for PPU {
    fn write(&mut self, register: WriteAccessRegister, value: u8) -> () {
        match register {
            WriteAccessRegister::PpuCtrl => {
                let new_control_register = ControlRegister { value };
                self.nmi_triggered = self
                    .status_reg
                    .get_flag(StatusRegisterFlag::VerticalBlankStarted)
                    && (new_control_register.is_generate_nmi_enabled()
                        && !self.control_reg.is_generate_nmi_enabled());

                self.control_reg.value = value;
                self.t_vram_address
                    .set(NM_TABLE, self.control_reg.get_base_nametable_index() as u16);
            }
            WriteAccessRegister::PpuMask => {
                self.mask_reg.value = value;
            }
            WriteAccessRegister::PpuScroll => {
                if self.write_toggle {
                    self.t_vram_address.set(FINE_Y, (value & 7) as u16);
                    self.t_vram_address.set(COARSE_Y, (value >> 3) as u16);
                } else {
                    self.fine_x_scroll = value & 7;
                    self.t_vram_address.set(COARSE_X, (value >> 3) as u16);
                }
                self.write_toggle = !self.write_toggle;
            }

            WriteAccessRegister::PpuAddr => {
                if self.write_toggle {
                    self.t_vram_address.set(LOW_BYTE, value as u16);
                    self.vram_address.address = self.t_vram_address.address;
                } else {
                    self.t_vram_address
                        .set(BITS_8_13, (value & 0b00111111) as u16);
                    self.t_vram_address.set(BIT_14, 0);
                }
                self.write_toggle = !self.write_toggle;
            }
            WriteAccessRegister::PpuData => {
                self.vram
                    .borrow_mut()
                    .store_byte(self.vram_address.address, value);
                assert!(
                    !self.is_rendering_in_progress(),
                    "Scanline {}",
                    self.scanline
                );
                assert!(!self.is_rendering_in_progress());
                self.vram_address.address += self.control_reg.get_vram_increment();
            }

            WriteAccessRegister::OamAddr => self.oam_address = value,
            WriteAccessRegister::OamData => {
                self.oam[self.oam_address as usize % 256] = value;
                self.oam_address += 1
            }
        }
        ()
    }
}

impl WriteOamDma for PPU {
    fn write_oam_dma(&mut self, data: [u8; 256]) -> () {
        let write_len = 256 - self.oam_address as usize;
        self.oam[self.oam_address as usize..].copy_from_slice(&data[..write_len as usize])
    }
}

impl ReadPpuRegisters for PPU {
    fn read(&mut self, register: ReadAccessRegister) -> u8 {
        match register {
            ReadAccessRegister::PpuStatus => {
                let mut vbl_flag_read_value = self
                    .status_reg
                    .get_flag(StatusRegisterFlag::VerticalBlankStarted);
                if self.scanline == VBLANK_START_SCANLINE && self.ppu_cycles == 0 {
                    self.nmi_supressed = true;
                    self.vbl_flag_supressed = true;
                    vbl_flag_read_value = false;
                }

                if self.scanline == VBLANK_START_SCANLINE
                    && (self.ppu_cycles == 1 || self.ppu_cycles == 2)
                {
                    self.nmi_supressed = true;
                    self.vbl_flag_supressed = true;
                    vbl_flag_read_value = true;
                }

                let mut current_status = self.status_reg;
                current_status.set_flag(
                    StatusRegisterFlag::VerticalBlankStarted,
                    vbl_flag_read_value,
                );
                self.write_toggle = false;
                self.status_reg
                    .set_flag(StatusRegisterFlag::VerticalBlankStarted, false);
                current_status.value
            }
            ReadAccessRegister::PpuData => {
                let val = self.vram.borrow_mut().get_byte(self.vram_address.address);
                assert!(!self.is_rendering_in_progress());
                self.vram_address.address += self.control_reg.get_vram_increment();
                val
            }
            ReadAccessRegister::OamData => {
                let val = self.oam[self.oam_address as usize];
                val
            }
        }
    }
}
impl PpuRegisterAccess for PPU {}