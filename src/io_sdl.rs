extern crate sdl2;

use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use crate::keyboard::{KeyEvent};
use crate::audio::{Audio};

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::gfx::primitives::DrawRenderer;

use std::iter::FromIterator; 
use std::collections::HashMap;
use std::sync::mpsc::{Sender,Receiver};

pub static mut SCREEN : Screen = [[(255,255,255); DISPLAY_HEIGHT]; DISPLAY_WIDTH];

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
        let window = video_subsys.window( &format!("chip-8: {}", self.title) , (DISPLAY_WIDTH as u32)*(DISPLAY_SCALING as u32), (DISPLAY_HEIGHT as u32)*(DISPLAY_SCALING as u32))
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

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

            let keys_state_new : HashMap<Scancode,bool> = HashMap::from_iter(events.keyboard_state().scancodes());
            for (k, v) in key_mappings.iter() {
                let old_state = keys_state[&Scancode::from_keycode(*k).unwrap()];
                let new_state = keys_state_new[&Scancode::from_keycode(*k).unwrap()];
                if  old_state != new_state {
                    if new_state {
                        self.keyboard_tx.send(KeyEvent::KeyDown(*v)).unwrap();
                    }
                    else {
                        self.keyboard_tx.send(KeyEvent::KeyUp(*v)).unwrap();
                    }
                }
            }
            keys_state = keys_state_new;
            
            canvas.clear();
            unsafe {
            for (x, col) in SCREEN.iter().enumerate() {
                for (y, color) in col.iter().enumerate() {
                    let x : i16 = (x*(DISPLAY_SCALING as usize)) as i16;
                    let y : i16 = (y*(DISPLAY_SCALING as usize)) as i16;
                    let (r, g, b) = *color;
                    let _ = canvas.box_(x, y, x + DISPLAY_SCALING - 1, y + DISPLAY_SCALING - 1, pixels::Color::RGB(r, g, b));                         
                }
            }
        }

            if let Ok(_) = self.audio_rx.try_recv() {
                audio.beep();
            }
            canvas.present();
        }

    }
}
