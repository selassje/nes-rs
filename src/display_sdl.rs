extern crate sdl2;

use crate::screen::{Screen,DISPLAY_HEIGHT,DISPLAY_WIDTH};

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::gfx::primitives::DrawRenderer;

use std::sync::mpsc::{Sender,Receiver, channel};


pub struct DisplaySdl{
        rx : Receiver<Screen>,
   pub  tx : Sender<Screen>,
}

impl DisplaySdl
{
    pub fn new() -> DisplaySdl {
        let (tx, rx) : (Sender<Screen>,Receiver<Screen>) = channel();
        DisplaySdl
        {
            tx : tx,
            rx : rx,
        } 
    }

    pub fn run(&self)
    {
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
        canvas.clear();
        canvas.present();

        let mut events = sdl_context.event_pump().unwrap();
        let mut screen : Screen = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        loop {
            for _ in events.poll_iter() { }
            canvas.clear();
            if let Ok(ret) = self.rx.try_recv() {
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


fn main() -> Result<(), String> {
    //std::sync::mspc::Sender<S
    let (tx, rx) : (Sender<Screen>,Receiver<Screen>) = channel();
    
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("rust-sdl2_gfx: draw line & FPSManager", 100, 200)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut lastx = 0;
    let mut lasty = 0;

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        for event in events.poll_iter() {

            match event {

                Event::Quit {..} => break 'main,

                Event::KeyDown {keycode: Some(keycode), ..} => {
                    if keycode == Keycode::Escape {
                        break 'main
                    } else if keycode == Keycode::Space {
                        println!("space down");
                        for i in 0..400 {
                            canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
                        }
                        canvas.present();
                    }
                }

                Event::MouseButtonDown {x, y, ..} => {
                    let color = pixels::Color::RGB(x as u8, y as u8, 255);
                    let white= pixels::Color::RGB(255, 255, 255);
                    let _ = canvas.line(lastx, lasty, x as i16, y as i16, color);
                    let _ = canvas.box_(320, 100, 330, 110, white);
                    lastx = x as i16;
                    lasty = y as i16;
                    println!("mouse btn down at ({},{})", x, y);
                    canvas.present();
                }

                _ => {}
            }
        }
    }

    Ok(())
}