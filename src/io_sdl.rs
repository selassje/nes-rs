extern crate sdl2;

use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};
use crate::keyboard::{KeyEvent};
use sdl2::audio::{AudioSpecDesired,AudioQueue};

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
use std::sync::atomic::{AtomicI16, Ordering};
use circular_queue::CircularQueue;
use std::ops::{Deref, DerefMut};



pub type SampleFormat = u8;

pub static mut SCREEN  : Screen = [[(255,255,255); DISPLAY_HEIGHT]; DISPLAY_WIDTH];
pub static PULSE_1_SAMPLE : AtomicI16 = AtomicI16::new(0);

pub const SAMPLE_RATE : usize = 41000;
pub const BUFFER_SIZE : usize = 41000;

pub struct SampleBuffer {
    pub index  : usize,
    pub buffer : [SampleFormat;BUFFER_SIZE]
}



lazy_static! {
    pub static ref KEYBOARD : Mutex<HashMap<Scancode, bool>> = Mutex::new(HashMap::new());
    pub static ref SAMPLES_QUEUE : Mutex<CircularQueue<SampleFormat>> = Mutex::new(CircularQueue::with_capacity(BUFFER_SIZE));
    pub static ref SAMPLE_BUFFER : Mutex<SampleBuffer> = Mutex::new(SampleBuffer{index :0, buffer : [0;BUFFER_SIZE]});

}

pub fn get_key_status(key : Scancode) -> bool {
    if let Some(status) = KEYBOARD.lock().unwrap().get(&key) {
        *status
    } else {
        false
    }
}


pub struct IOSdl{
        title       : String,
}       

impl IOSdl
{
    pub fn new(title       : String) -> IOSdl {
        IOSdl
        {
            title,
        
        } 
    }
    pub fn run(&self)
    {
        const DISPLAY_SCALING : i16 = 2;

        let sdl_context = sdl2::init().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),  
            samples: Some(BUFFER_SIZE as u16)
            };
    
        let apu_audio: AudioQueue<SampleFormat> =  sdl_audio.open_queue(None, &desired_spec).unwrap();
        //let apu_audio =  Audio::new(&sdl_audio).device;
       

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
            //apu_audio.pause();
            /*
            let mut samples_queue = SAMPLES_QUEUE.lock().unwrap();
            if samples_queue.is_full() {
                let vec : Vec<_> = samples_queue.asc_iter().copied().collect();
                apu_audio.queue(vec.as_slice());
                apu_audio.resume();
                samples_queue.clear();
            }
            */
            let mut buffer = SAMPLE_BUFFER.lock().unwrap(); 

            if buffer.index == BUFFER_SIZE {
               // println!("About to play");
                apu_audio.queue(&buffer.buffer);
                apu_audio.resume();
                buffer.index = 0;
            }

          //  audio_queue.queue(&[5,-5,6,7,8,9,10]);
           // audio_queue.resume();
        }

    }
}
