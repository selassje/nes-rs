use crate::memory::{Memory, RAM};
use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use std::sync::mpsc::{Sender};
use std::default::{Default};
use crate::io_sdl::{SCREEN};

type RgbColor = (u8,u8,u8);

pub enum RegisterAddress {
    PpuCtrl = 0x2000
}

struct Register {
    value   : u8,
    address : RegisterAddress
}

impl Register {

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

struct Palette {
    colors : [RgbColor;4]
}

impl Default for Palette {
    fn default() -> Self {
        Palette {colors : [
            (0,0,0),
            (255,0,0),
            (0,255,0),
            (0,0,255)
        ]}
    }
}

pub struct PPU
{
   ram          : RAM,
   screen_tx    : Sender<Screen>,
   screen       : Screen,
}

impl PPU
{
    pub fn new(screen_tx: Sender<Screen>, chr_rom: Vec::<u8>) -> PPU {
        let mut ram = RAM::new();
        ram.store_bytes(0x00, &chr_rom);

        println!("CHROM byte 1 {:#X} CHROM byte 2 {:#X}",ram.get_byte(0x00),ram.get_byte(0x01));
    
        PPU {
            ram : ram,
            screen_tx : screen_tx,
            screen : [[(255,255,255); DISPLAY_HEIGHT]; DISPLAY_WIDTH],
        }
    }

    pub fn render_frame(&self) {
        let pallete : Palette =  Default::default();
        

        let mut pattern_table : PatternTable = Default::default();
        for i in 0.. pattern_table.tiles.len() {
            let mut tile_data = [0 as u8;16];
            for j in 0..16 {
                tile_data[j] = self.ram.get_byte(i as u16 * 16 + j as u16) ;
            }
            pattern_table.tiles[i] = Tile {
                data : tile_data
            }            
        }

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let tile_y = y / 8;
                let tile_x = x / 8;
                //println!("tile_x {} tile_y {}",tile_x,tile_y);
                let tile = pattern_table.tiles[16 * tile_y + tile_x];
                let color_index = tile.get_color_index(x % 8, y % 8);
                let color = pallete.colors[color_index];
                unsafe {
                    SCREEN[x][y] = color;
                }
            }
        }

    }
             
}