extern crate sdl2;

use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use crate::keyboard::{KeyEvent};
use crate::audio::{Audio};

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use lazy_static::lazy_static;

use std::iter::FromIterator; 
use std::collections::HashMap;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Mutex;


pub static mut SCREEN : Screen = [[(255,255,255); DISPLAY_HEIGHT]; DISPLAY_WIDTH];

lazy_static! {
    pub static ref KEYBOARD : Mutex<HashMap<Scancode, bool>> = Mutex::new(HashMap::new());
}

pub fn get_key_status(key : Scancode) -> bool {
    if let Some(status) = KEYBOARD.lock().unwrap().get(&key) {
        *status
    } else {
        false
    }
}

pub fn set_key_status(key : Scancode, value: bool) {
    KEYBOARD.lock().unwrap().insert(key, value);
}

pub struct IOSdl{
        title       : String,
        screen_rx   : Receiver<Screen>,
        keyboard_tx : Sender<KeyEvent>,
        audio_rx    : Receiver<bool>,
}       

impl IOSdl
{
    pub fn new(title       : String,
               screen_rx   : Receiver<Screen>, 
               keyboard_tx : Sender<KeyEvent>, 
               audio_rx    : Receiver<bool>) -> IOSdl {
        IOSdl
        {
            title,
            screen_rx,
            keyboard_tx,
            audio_rx,
        } 
    }
    pub fn run(&self)
    {
        let key_mappings : HashMap<Keycode, u8> = HashMap::from_iter(vec!(
            (Keycode::Num1,1),
            (Keycode::Num2,2),
            (Keycode::Num3,3),
            (Keycode::Num4,0xC),
            (Keycode::Q,4),
            (Keycode::W,5),
            (Keycode::E,6),
            (Keycode::R,0xD),
            (Keycode::A,7),
            (Keycode::S,8),
            (Keycode::D,9),
            (Keycode::F,0xE),
            (Keycode::Z,0xA),
            (Keycode::X,0),
            (Keycode::C,0xB),
            (Keycode::V,0xF))
        );

        const DISPLAY_SCALING : i16 = 2;

        let sdl_context = sdl2::init().unwrap();
        let audio = Audio::new(&sdl_context.audio().unwrap());
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window( &format!("nes-rs: {}", self.title) , (DISPLAY_WIDTH as u32)*(DISPLAY_SCALING as u32), (DISPLAY_HEIGHT as u32)*(DISPLAY_SCALING as u32))
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string()).unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();

        let mut events = sdl_context.event_pump().unwrap();
        let mut keys_state : HashMap<Scancode,bool> = HashMap::from_iter(events.keyboard_state().scancodes());
     
        'main: loop {
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown {keycode: Some(keycode), ..} => {
                        if keycode == Keycode::Escape {
                            break 'main
                        } 
                    }
                    _ => {}
                }   
            }

            *KEYBOARD.lock().unwrap() = HashMap::from_iter(events.keyboard_state().scancodes());
   
            canvas.clear();
        
            unsafe {
            for (x, col) in SCREEN.iter().enumerate() {
                for (y, color) in col.iter().enumerate() {
                    let x  = (x*(DISPLAY_SCALING as usize)) as i32;
                    let y  = (y*(DISPLAY_SCALING as usize)) as i32;
                    let rect = Rect::new(x,y,DISPLAY_SCALING as u32, DISPLAY_SCALING as u32);
                    let _ = canvas.set_draw_color(*color);   
                    let _ = canvas.draw_rect(rect);                      
                }
                }   
            } 
            canvas.present();
        }

    }
}
