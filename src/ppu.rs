use crate::io_sdl::{SCREEN};
use crate::common::{Mirroring};
use crate::cpu_ppu::*;
use crate::memory::{Memory};
use crate::vram::{VRAM};
use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use crate::colors::{RgbColor, ColorMapper, DefaultColorMapper};
use std::thread::park;

use std::sync::mpsc::{Sender};
use std::default::{Default};

enum ControlRegisterFlag {
    BaseNametableAddress          = 0b00000011,
    VramIncrement                 = 0b00000100,
    SpritePattern8x8              = 0b00001000,
    BackgroundPatternTableAddress = 0b00010000,
    SpriteSize                    = 0b00100000,
    PpuMasterSlaveSelect          = 0b01000000,
    GenerateNMI                   = 0b10000000,
}

static  PPU_CYCLES_PER_SCANLINE : u16 = 341;
static  PRE_RENDER_SCANLINE     : i16 = -1;
static  POST_RENDER_SCANLINE    : i16 = 240;
static  VBLANK_START_SCANLINE   : i16 = 241;


struct ControlRegister {
    value : u8
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
        (self.value & ControlRegisterFlag::GenerateNMI as u8) !=0
    }

} 

enum MaskRegisterFlag {
    GrayScale                       = 0b00000001,
    ShowBackgroundInLeftMost8Pixels = 0b00000010,
    ShowSpritesdInLeftMost8Pixels   = 0b00000100,
    ShowBackground                  = 0b00001000,
    ShowSprites                     = 0b00010000,
    EmphasizeRed                    = 0b00100000,
    EmphasizeGreen                  = 0b01000000,
    EmphasizeBlue                   = 0b10000000,
}

struct MaskRegister {
    value : u8
}

impl MaskRegister {
    fn is_flag_enabled(&self, flag: MaskRegisterFlag) -> bool {
        (self.value & flag as u8) != 0
    }
}

enum StatusRegisterFlag {
    SpriteOverflow       = 0b00100000,
    Sprite0Hit           = 0b01000000,
    VerticalBlankStarted = 0b10000000,
}

struct StatusRegister {
    value : u8
}

impl StatusRegister {
    fn set_flag(&mut self, flag: StatusRegisterFlag, enable : bool) {
        if enable {
            self.value |= flag as u8;
        } else {
            self.value &= !(flag as u8);
        }
    }
}

struct ScrollRegister {
    x : u8,
    y : u8,
    next_read_is_x : bool
}

#[derive(Copy,Clone,Default)]
struct Tile {
   data: [u8;16]
}

impl Tile {
    fn get_color_index(&self, x: usize, y: usize) -> usize {
        let shift = 7 - x;
        let mask = 1<<shift; 
        let lo_bit =  (mask & self.data[y]) >> shift;
        let hi_bit =  (mask & self.data[y + 8]) >> shift;
        (2*hi_bit + lo_bit) as usize
    } 
}

struct PatternTable {
    tiles: [Tile;256]
}

impl Default for PatternTable {
    fn default() -> Self {
        PatternTable {tiles : [Default::default();256]}
    }
}

impl PatternTable {
    fn new(vram : &VRAM, table_index : u8) -> Self {
        let mut pattern_table : PatternTable = Default::default();
        for i in 0.. pattern_table.tiles.len() {
            pattern_table.tiles[i] = Tile {
                data : vram.get_pattern_table_tile_data(table_index, i as u8)
            }            
        }
        pattern_table   
    }

}
type Palette  = [RgbColor;4];
type Palettes = [Palette; 4];

#[derive(Copy,Clone,Default)]
struct Sprite {
    oam_index : u8,
    data      : [u8;4]
}

type Sprites = Vec<Sprite>;

impl Sprite {
    fn get_y(&self) -> u8 {
        self.data[0] 
    }

    fn get_x(&self) -> u8 {
        self.data[3]
    }

    fn get_tile_index(&self, is_8x16_mode: bool) -> u8 {
        if is_8x16_mode {
            self.data[1] >> 1
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

type OAM =  [u8;256];

struct VRAMAddress {
    address : u16,
    next_ppuaddr_write_is_hi : bool
}

pub struct PPU
{
   ram            : VRAM,
   screen_tx      : Sender<Screen>,
   screen         : Screen,
   control_reg    : ControlRegister,
   mask_reg       : MaskRegister,
   status_reg     : StatusRegister,
   scroll_reg     : ScrollRegister,
   vram_address   : VRAMAddress,
   oam_address    : u8,
   oam            : OAM,
   pattern_tables : [PatternTable;2],
   ppu_cycles     : u16,
   scanline       : i16,
   color_mapper   : Box::<dyn ColorMapper>,
}

impl PPU
{
    pub fn new(screen_tx: Sender<Screen>, chr_rom: Vec::<u8>, mirroring: Mirroring) -> PPU {
        let ram = VRAM::new(&chr_rom, mirroring);
        let pattern_tables = [PatternTable::new(&ram, 0), PatternTable::new(&ram, 1)];
        PPU {
            ram            : ram,
            screen_tx      : screen_tx,
            screen         : [[(255,255,255); DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            control_reg    : ControlRegister{value: 0},
            mask_reg       : MaskRegister{value: 0},
            status_reg     : StatusRegister{value: 0},
            scroll_reg     : ScrollRegister{x: 0, y: 0, next_read_is_x : true },
            vram_address   : VRAMAddress{address: 0, next_ppuaddr_write_is_hi : true},
            oam_address    : 0,
            oam            : [0;256],
            pattern_tables : pattern_tables,
            ppu_cycles     : 0,
            scanline       : -1,
            color_mapper   : Box::new(DefaultColorMapper{})
        }
    }

    //fn get_pattern_table()

    fn displayTileInfo(&self, tile_x : u8, tile_y :u8) {
        let name_table_index    = self.control_reg.get_base_nametable_index();
        let tile_index = self.ram.get_nametable_tile_index(name_table_index, tile_x, tile_y);
        let pattern_table_index = self.control_reg.get_background_pattern_table_index();
        println!("Tile Info");
        println!("X={} Y={} TileIndex {:#2X} PatternTable {}", tile_x, tile_y, tile_index,pattern_table_index);
        println!("Nametable {} Mirroring {:?}",name_table_index, self.ram.get_mirroring());

    }

    fn default_palette()-> Palette {
        [ (0,0,0),
          (255,0,0),
          (0,255,0),
          (0,0,255)    
        ]
    }

    fn render_background_frame(&mut self) {
        let background_palettes = self.get_palettes(true);
        let sprite_palettes     = self.get_palettes(false);

        
        let name_table_index    = self.control_reg.get_base_nametable_index();
        let pattern_table_index = self.control_reg.get_background_pattern_table_index();
        let pattern_table       = &self.pattern_tables[pattern_table_index as usize];

       // self.displayTileInfo(13,22);
        //self.render_chr_data();
        //park();
        //panic!("");

        for y in 0..DISPLAY_HEIGHT {
            let (sprites, is_overflow_detected) = self.get_sprites_for_scanline_and_check_for_overflow(y as u8);
            self.status_reg.set_flag(StatusRegisterFlag::SpriteOverflow, is_overflow_detected);
            for x in 0..DISPLAY_WIDTH {
                let tile_y = (y / 8) as u8;
                let tile_x = (x / 8) as u8;
                let color_tile_y = (y / 16) as u8;
                let color_tile_x = (x / 16) as u8;
                let tile_index = self.ram.get_nametable_tile_index(name_table_index, tile_x, tile_y);
                let tile = pattern_table.tiles[tile_index as usize];
                let bg_color_index = tile.get_color_index(x % 8, y % 8);
                let (sprite_color_index, sprite) = self.get_sprite_color_index(&sprites, x as u8, y as u8);
                let bg_palette_index  = self.ram.get_background_pallete_index(name_table_index, color_tile_x, color_tile_y);
                let color = self.determine_pixel_color(bg_color_index as u8, 
                                                      sprite_color_index, 
                                                      &sprite, 
                                                      &background_palettes[bg_palette_index as usize], 
                                                      &sprite_palettes);
                unsafe {
                    SCREEN[x][y] = color;
                }
            }
        }
       // self.oam = [0;256];
        //park();
    }
    
    fn determine_pixel_color(&self, bg_color_index: u8, 
                                    sprite_color_index : u8, 
                                    sprite : &Sprite,
                                    bg_pallete : &Palette,
                                    sprite_palletes : &Palettes) -> RgbColor
    {
            let bg_color        =  bg_pallete[bg_color_index as usize];
            let sprite_color    =  sprite_palletes[sprite.get_palette_index() as usize][sprite_color_index as usize];
            let universal_color =  bg_pallete[0];
            if bg_color_index == 0 && sprite_color_index == 0 {
                universal_color
            } else if bg_color_index != 0 && sprite_color_index == 0 {
                bg_color
            } else if bg_color_index == 0 && sprite_color_index != 0 {
                sprite_color
            } else {
                if sprite.if_draw_in_front() {
                    sprite_color
                }else {
                    bg_color
                }
            }
    }  

    fn get_sprite_color_index(&self, sprites : &Sprites, x :u8, y:u8) -> (u8,Sprite) {
        let is_sprite_mode_8x16 = self.control_reg.get_sprite_size_height() == 16;
        for sprite in sprites {
            if x >= sprite.get_x() && (x as u16) < sprite.get_x() as u16 + 8 {
                let pattern_table_index =   if is_sprite_mode_8x16 {sprite.get_pattern_table_index_for_8x16_mode()} 
                                            else {self.control_reg.get_sprite_pattern_table_index_for_8x8_mode()};
                let pattern_table = &self.pattern_tables[pattern_table_index as usize];
                let mut tile_index = sprite.get_tile_index(is_sprite_mode_8x16);
                if is_sprite_mode_8x16 {
                    if sprite.if_flip_vertically() {
                        if y <= sprite.get_y() + 7 {
                            tile_index += 1;
                        }
                        else {
                            tile_index -= 1;
                        }
                    }
                }
                let mut x = x - sprite.get_x();
                let mut y = y - sprite.get_y();     
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
        (0, Default::default())
    }

    fn get_sprites_for_scanline_and_check_for_overflow(&self, y: u8) -> (Sprites, bool) {
        let sprites = self.oam.chunks(4).enumerate().map(|(i,s)|
            Sprite {oam_index: i as u8, data: [s[0],s[1],s[2],s[3]]}    
        );
        let sprites = sprites.filter(|sprite| y >= sprite.get_y() && y < sprite.get_y() + self.control_reg.get_sprite_size_height());
        let if_overflow = sprites.clone().count() > 8;
        (sprites.take(8).collect(), if_overflow)   
    }

    fn render_chr_data(&mut self) {
        let palette = Self::default_palette();
        let pattern_table_index = self.control_reg.get_background_pattern_table_index();
        let pattern_table       = &self.pattern_tables[pattern_table_index as usize];

        for y in 0..DISPLAY_HEIGHT {
            let (sprites, is_overflow_detected) = self.get_sprites_for_scanline_and_check_for_overflow(y as u8);
            self.status_reg.set_flag(StatusRegisterFlag::SpriteOverflow, is_overflow_detected);
            for x in 0..DISPLAY_WIDTH {
                let tile_y = y / 8;
                let tile_x = x / 8;
                //println!("tile_x {} tile_y {}",tile_x,tile_y);
                let tile = pattern_table.tiles[16 * tile_y + tile_x];
                let bg_color_index = tile.get_color_index(x % 8, y % 8);
                let sprite_color_index = 0;
                let color = palette[bg_color_index];
                unsafe {
                    SCREEN[x][y] = color;
                }
            }
        }
    }

    fn render_sprites_frame(&self) {

    }

    pub fn process_cpu_cycles(&mut self, cpu_cycles: u8) -> bool {
        let mut nmi_triggered = false;
        
              
        self.ppu_cycles += 3 * cpu_cycles as u16;
        //println!("PPU Cycles {} Scanline {}",self.ppu_cycles,self.scanline);
        if self.ppu_cycles > PPU_CYCLES_PER_SCANLINE {
            self.scanline +=1;
            self.ppu_cycles %= PPU_CYCLES_PER_SCANLINE;
           // println!("process_cpu_cycles scanline {}",self.scanline);
            if self.scanline == 262 {
                self.scanline = PRE_RENDER_SCANLINE;
                self.status_reg.set_flag(StatusRegisterFlag::VerticalBlankStarted, false);
                self.status_reg.set_flag(StatusRegisterFlag::SpriteOverflow, false);
            } else if self.scanline == POST_RENDER_SCANLINE {
                //println!("About to render frame");
              
                if self.mask_reg.is_flag_enabled(MaskRegisterFlag::ShowBackground) {
                    //println!("About to render background");
                    self.render_background_frame();
                }
                if self.mask_reg.is_flag_enabled(MaskRegisterFlag::ShowSprites) {
                    self.render_sprites_frame();
                }
               
            } else if self.scanline == VBLANK_START_SCANLINE {
               // println!("Sending NMI {} {}",self.control_reg.value & ControlRegisterFlag::GenerateNMI as u8, self.control_reg.value);
                nmi_triggered = self.control_reg.is_generate_nmi_enabled();
                self.status_reg.set_flag(StatusRegisterFlag::VerticalBlankStarted, true);
            }
        }
        return nmi_triggered;
    }

    fn get_palettes(&self, for_background : bool) -> Palettes {
            let mut palletes : Palettes =  Default::default();
            let raw_universal_bckg_color = self.ram.get_universal_background_color();
            for (i, p) in palletes.iter_mut().enumerate() {
                let raw_colors = if for_background {self.ram.get_background_palette(i as u8)} else {self.ram.get_sprite_palette(i as u8)} ;
                *p = [self.color_mapper.map_nes_color(raw_universal_bckg_color), 
                      self.color_mapper.map_nes_color(raw_colors[0]), 
                      self.color_mapper.map_nes_color(raw_colors[1]), 
                      self.color_mapper.map_nes_color(raw_colors[2])];
            }
            palletes
    }
    
}

impl WritePpuRegisters for PPU {
    fn write(&mut self, register : WriteAccessRegister, value: u8) -> () {
        
        match register {
            WriteAccessRegister::PpuCtrl =>  { 
                //println!("Control register updated {:X}",value);  
                self.control_reg.value = value;
            }
            WriteAccessRegister::PpuMask => { 
                //println!("Mask register updated {:X}",value);  
                self.mask_reg.value = value;
            }
            WriteAccessRegister::PpuScroll => {
                if self.scroll_reg.next_read_is_x {
                    self.scroll_reg.x = value;
                } else {
                    self.scroll_reg.y = value;
                }
            } 
            WriteAccessRegister::OamAddr => self.oam_address = value,
            WriteAccessRegister::OamData => {
                self.oam[self.oam_address as usize % 256] = value;
                self.oam_address += 1
            }
            WriteAccessRegister::PpuAddr => {
                //println!("Setting VRAM address {:X}", value);
                if self.vram_address.next_ppuaddr_write_is_hi {
                    self.vram_address.address = (value as u16)<<8 | (self.vram_address.address & 0x00FF);
                } else {
                    self.vram_address.address = (value as u16) | (self.vram_address.address & 0xFF00);  
                }
                self.vram_address.next_ppuaddr_write_is_hi = !self.vram_address.next_ppuaddr_write_is_hi;
            } 
            WriteAccessRegister::PpuData => {
                self.ram.store_byte(self.vram_address.address, value);
                //println!("VRAM address {:X} inc {}", self.vram_address.address, self.control_reg.get_vram_increment());
                self.vram_address.address += self.control_reg.get_vram_increment();
            }
            _         => panic!("Unrecognised register {:?} in WritePpuRegisters", register)
        }
        ()
    }
}

impl WriteOamDma for PPU {
    fn writeOamDma(&mut self , data: [u8;256]) -> () {
        let write_len  = 256 - self.oam_address as usize;
        self.oam[self.oam_address as usize ..].copy_from_slice(&data[..write_len as usize])
    }
}

impl ReadPpuRegisters for PPU {
    fn read(&mut self, register : ReadAccessRegister) -> u8 {
        match register {
            ReadAccessRegister::PpuStatus => self.status_reg.value,
            ReadAccessRegister::PpuData   => {
                let val = self.ram.get_byte(self.vram_address.address);
                self.vram_address.address += self.control_reg.get_vram_increment();
                val
            }
            ReadAccessRegister::OamData  => {
                let val = self.oam[self.oam_address as usize];
                self.oam_address += 1;
                val
            }
            _         => panic!("Unrecognised register in ReadPpuRegisters")
        }
    }
}

impl PpuRegisterAccess for PPU {}