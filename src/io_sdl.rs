extern crate sdl2;

use crate::screen::{Screen, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl2::audio::{AudioQueue, AudioSpecDesired};

use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::surface::Surface;
use std::iter::{FromIterator};

use crate::{nes_format_reader, nes::Nes, keyboard, controllers, ppu::PPU, apu, cpu};

use circular_queue::CircularQueue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI16};
use std::{cell::RefCell, sync::Mutex, rc::Rc};

pub type SampleFormat = u8;

pub static mut SCREEN: Screen = [[(255, 255, 255); DISPLAY_HEIGHT]; DISPLAY_WIDTH];
pub static PULSE_1_SAMPLE: AtomicI16 = AtomicI16::new(0);

pub const SCREEN2_SIZE : usize = DISPLAY_HEIGHT * DISPLAY_WIDTH * 3;
pub static mut SCREEN2: [u8;SCREEN2_SIZE] = [0;SCREEN2_SIZE];


pub const SAMPLE_RATE: usize = 41000;
pub const BUFFER_SIZE: usize = 41000;

pub struct SampleBuffer {
    pub index: usize,
    pub buffer: [SampleFormat; BUFFER_SIZE],
}

lazy_static! {
    pub static ref KEYBOARD: Mutex<HashMap<Scancode, bool>> = Mutex::new(HashMap::new());
    pub static ref SAMPLES_QUEUE: Mutex<CircularQueue<SampleFormat>> =
        Mutex::new(CircularQueue::with_capacity(BUFFER_SIZE));
    pub static ref SAMPLE_BUFFER: Mutex<SampleBuffer> = Mutex::new(SampleBuffer {
        index: 0,
        buffer: [0; BUFFER_SIZE]
    });
}

pub fn get_key_status(key: Scancode) -> bool {
    if let Some(status) = KEYBOARD.lock().unwrap().get(&key) {
        *status
    } else {
        false
    }
}

pub struct IOSdl<'a> {
    title: String,
    nes: Option<Nes<'a>>,
}

impl<'a> IOSdl<'a> {
    pub fn new(title: String, nes: Option<Nes>) -> IOSdl {
        IOSdl { title, nes }
    }
    pub unsafe fn run(&mut self, nes_file : &nes_format_reader::NesFile ) {
        const DISPLAY_SCALING: i16 = 1;

        let sdl_context = sdl2::init().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(BUFFER_SIZE as u16),
        };

        let apu_audio: AudioQueue<SampleFormat> =
            sdl_audio.open_queue(None, &desired_spec).unwrap();


       // let mut nes = Nes::new(nes_file);
       let mapper = nes_file.create_mapper();
       let controller_1 = keyboard::KeyboardController::get_default_keyboard_controller_player1();
       let controller_2 = keyboard::KeyboardController::get_default_keyboard_controller_player2();
    
       let controllers = Rc::new(RefCell::new(controllers::Controllers::new(Box::new(controller_1), Box::new(controller_2))));
                                                    
       let ppu = RefCell::new(PPU::new(mapper.get_chr_rom().to_vec(),nes_file.get_mirroring()));
       let apu = Rc::new(RefCell::new(apu::APU::new()));
       let mut cpu = cpu::CPU::new(mapper, &ppu, apu, controllers);

      
       //let apu_audio =  Audio::new(&sdl_audio).device;

        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                &format!("nes-rs: {}", self.title),
                (DISPLAY_WIDTH as u32) * (DISPLAY_SCALING as u32),
                (DISPLAY_HEIGHT as u32) * (DISPLAY_SCALING as u32),
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();


        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

            let texture_creator = canvas.texture_creator();
    
           
        

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.present();
     
     

        let mut events = sdl_context.event_pump().unwrap();
        
        'main: loop {
            
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if keycode == Keycode::Escape {
                            break 'main;
                        }
                    }
                    _ => {}
                }
            }
            

            *KEYBOARD.lock().unwrap() = HashMap::from_iter(events.keyboard_state().scancodes());
            if self.nes.is_some() {
               //self.nes.as_mut().unwrap().run_cpu_instruction();
            }
  

               
         //  canvas.with_texture_canvas(&mut texture, || {});
          if  true {
            println!("here");
            let s = Surface::from_data(&mut SCREEN2[..], DISPLAY_WIDTH as u32 , DISPLAY_HEIGHT as u32, (DISPLAY_WIDTH * 3 ) as u32, pixels::PixelFormatEnum::RGB24).unwrap();
            let mut texture = texture_creator.create_texture_from_surface(s).unwrap();
            Rect::new(0,0,DISPLAY_WIDTH as u32,DISPLAY_HEIGHT as u32);
            canvas.copy(&texture, None, None );
            canvas.present();
        }
        
           
            //println!("asdad");
            //apu_audio.pause();
            /*
                        let mut samples_queue = SAMPLES_QUEUE.lock().unwrap();
                        if samples_queue.is_full() {
                            let vec : Vec<_> = samples_queue.asc_iter().copied().collect();
                            apu_audio.queue(vec.as_slice());
                            apu_audio.resume();
                            samples_queue.clear();
                        }
    
                        let mut buffer = SAMPLE_BUFFER.lock().unwrap();

                        if buffer.index == BUFFER_SIZE {
                            println!("About to play");
                            apu_audio.queue(&buffer.buffer);
                            apu_audio.resume();
                            buffer.index = 0;
                        }
            */
            //  audio_queue.queue(&[5,-5,6,7,8,9,10]);
            // audio_queue.resume();
        }
    }
}
