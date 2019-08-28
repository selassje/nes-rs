extern crate sdl2;

use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use crate::keyboard::{KeyEvent};


use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::gfx::primitives::DrawRenderer;

use std::iter::FromIterator; 
use std::collections::HashMap;
use std::sync::mpsc::{Sender,Receiver};

pub struct DisplaySdl{
        screen_rx   : Receiver<Screen>,
        keyboard_tx : Sender<KeyEvent>,
}       

impl DisplaySdl
{
    pub fn new(screen_rx : Receiver<Screen>, keyboard_tx : Sender<KeyEvent>) -> DisplaySdl {
        DisplaySdl
        {
            screen_rx,
            keyboard_tx,
        } 
    }
    pub fn run(&self)
    {
        let key_mappings : HashMap<Keycode, u8> = HashMap::from_iter(vec!(
            (Keycode::NumLockClear, 1),
            (Keycode::KpDivide,2),
            (Keycode::KpMultiply,3),
            (Keycode::KpBackspace,0xC),
            (Keycode::Backspace, 0xC),
            (Keycode::Kp7,4),
            (Keycode::Kp8,5),
            (Keycode::Kp9,6),
            (Keycode::KpMinus,0xD),
            (Keycode::Kp4,7),
            (Keycode::Kp5,8),
            (Keycode::Kp6,9),
            (Keycode::KpPlus,0xE),
            (Keycode::Kp1,0xA),
            (Keycode::Kp2,0),
            (Keycode::Kp3,0xB),
            (Keycode::KpEnter,0xF))
        );



        const DISPLAY_SCALING : i16 = 20;

        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window("chip-8", (DISPLAY_WIDTH as u32)*(DISPLAY_SCALING as u32), (DISPLAY_HEIGHT as u32)*(DISPLAY_SCALING as u32))
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();

        let mut events = sdl_context.event_pump().unwrap();
        let mut keys_state : HashMap<Scancode,bool> = HashMap::from_iter(events.keyboard_state().scancodes());
        let mut screen : Screen = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];

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
            if let Ok(ret) = self.screen_rx.try_recv() {
                screen = ret;
            } 
            for (x, col) in screen.iter().enumerate() {
                for (y, b) in col.iter().enumerate() {
                    let x : i16 = (x*(DISPLAY_SCALING as usize)) as i16;
                    let y : i16 = (y*(DISPLAY_SCALING as usize)) as i16;
                    if *b {
                        let _ = canvas.box_(x, y, x + DISPLAY_SCALING, y + DISPLAY_SCALING, pixels::Color::RGB(255, 255, 255));          
                    } else
                    {
                        let _ = canvas.box_(x, y, x + DISPLAY_SCALING, y + DISPLAY_SCALING, pixels::Color::RGB(0, 0, 0));       
                    }
                }
            }
            canvas.present();


        }

    }
}
