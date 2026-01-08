use std::{
    env,
    fs::File,
    io::{Read, Write},
};

mod frontend;

use frontend::*;

use emscripten_main_loop::MainLoop;
use frontend::sdl2_imgui_opengl::DOUBLE_FPS;
use frontend::sdl2_imgui_opengl::HALF_FPS;
use frontend::{
    Frontend, FrontendControl, FrontendState, sdl2_imgui_opengl::Sdl2ImGuiOpenGlFrontend,
};
use nes_rs::*;

extern crate enum_tryfrom;

const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(
    (std::time::Duration::from_secs(1).as_nanos() / (crate::DEFAULT_FPS) as u128) as u64,
);

#[cfg(target_os = "emscripten")]
unsafe extern "C" {
    fn emscripten_run_script(s: *const std::os::raw::c_char);
}
pub struct Emulation {
    nes: Nes,
    io: Sdl2ImGuiOpenGlFrontend,
    io_control: FrontendControl,
    io_state: FrontendState,
    fps: u16,
    one_second_timer: std::time::Instant,
    frame_start: std::time::Instant,
    is_audio_available: bool,
}
#[allow(clippy::new_without_default)]
impl Emulation {
    pub fn new() -> Result<Self, String> {
        let io = frontend::sdl2_imgui_opengl::Sdl2ImGuiOpenGlFrontend::new();

        let mut nes: Nes = crate::Nes::new();

        let mut initial_title: Option<String> = None;
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let path = &args[1];
            load(&mut nes, path);
            initial_title = Some(path.clone());
        } else {
            load_demo(&mut nes);
        }

        let io_state: FrontendState = Default::default();
        let frame_start = std::time::Instant::now();
        let fps = 0;
        let one_second_timer = std::time::Instant::now();

        let io_control = FrontendControl {
            target_fps: DEFAULT_FPS,
            current_fps: 0,
            title: initial_title,
            controller_type: [crate::ControllerType::NullController; 2],
            error: None,
        };
        let is_audio_available = io.is_audio_available();
        Ok(Self {
            nes,
            io,
            io_control,
            io_state,
            fps,
            one_second_timer,
            frame_start,
            is_audio_available,
        })
    }
}

impl emscripten_main_loop::MainLoop for Emulation {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        self.io_control.controller_type = [
            self.nes
                .config()
                .get_controller_type(crate::ControllerId::Controller1),
            self.nes
                .config()
                .get_controller_type(crate::ControllerId::Controller2),
        ];
        let mut emulation_frame: Option<&EmulationFrame> = None;
        if !self.io_state.pause {
            emulation_frame = Some(self.nes.run_single_frame(&self.io));
            if self.one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
                self.fps += 1;
            } else {
                self.one_second_timer = std::time::Instant::now();
                self.io_control.current_fps = self.fps;
                self.fps = 1;
            }
        }

        self.io_state = self
            .io
            .present_frame(self.io_control.clone(), emulation_frame);

        handle_io_state(&mut self.nes, &self.io_state, &mut self.io_control);

        if !self.io_state.pause {
            let elapsed_time_since_frame_start = self.frame_start.elapsed();
            if !self.is_audio_available && elapsed_time_since_frame_start < FRAME_DURATION {
                #[cfg(not(target_os = "emscripten"))]
                std::thread::sleep(FRAME_DURATION - elapsed_time_since_frame_start);
            }
            self.frame_start = std::time::Instant::now();
        } else {
            #[cfg(not(target_os = "emscripten"))]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn get_bytes_from_file(file_name: &str) -> Result<Vec<u8>, String> {
    let mut rom = Vec::new();
    let mut file = File::open(file_name).map_err(|e| {
        println!(
            "Unable to open ROM {} current dir {}",
            file_name,
            std::env::current_dir().unwrap().display()
        );
        e.to_string()
    })?;
    file.read_to_end(&mut rom).map_err(|open_err| {
        println!("Unable to read ROM {}", file_name);
        open_err.to_string()
    })?;
    Ok(rom)
}

fn read_demo() -> Vec<u8> {
    let mut demo_rom = include_bytes!("../res/nes-rs-demo.nes").to_vec();
    let pattern = b"xxxxxx";
    const GIT_HASH: &str = git_version::git_version!();
    let replacement = &GIT_HASH.as_bytes()[..6];
    if let Some(pos) = demo_rom
        .windows(pattern.len())
        .position(|window| window == pattern)
    {
        demo_rom[pos..pos + 6].copy_from_slice(replacement);
    } else {
        println!("Pattern not found!");
    }
    demo_rom
}

pub fn run(mut emulation: Emulation) {
    while !emulation.io_state.quit {
        emulation.main_loop();
    }
}

fn handle_io_state(nes: &mut Nes, io_state: &FrontendState, io_control: &mut FrontendControl) {
    if io_state.power_cycle {
        nes.power_cycle();
    }

    if let Some(ref nes_file_path) = io_state.load_nes_file {
        load(nes, nes_file_path.as_str());
        io_control.title = Some(nes_file_path.clone());
    }

    if let Some(ref save_state_path) = io_state.save_state {
        let serialized = nes.save_state();
        let file_name = save_state_path.as_str();
        let mut file = File::create(file_name).unwrap_or_else(|_| {
            panic!(
                "Unable to create save file {} current dir {}",
                file_name,
                std::env::current_dir().unwrap().display()
            )
        });
        file.write_all(serialized.as_slice()).unwrap();
        #[cfg(target_os = "emscripten")]
        unsafe {
            let script = std::ffi::CString::new("refreshDownloadList();").unwrap();
            emscripten_run_script(script.as_ptr());
        };
    }

    if let Some(ref load_state_path) = io_state.load_state {
        let file_name = load_state_path.as_str();
        let save = std::fs::read(file_name).unwrap_or_else(|_| {
            panic!(
                "Unable to open save file {} current dir {}",
                file_name,
                std::env::current_dir().unwrap().display()
            )
        });
        nes.load_state(save);
        io_control.title = Some(load_state_path.clone());
    }

    if let Some(ref speed) = io_state.speed {
        match speed {
            Speed::Half => io_control.target_fps = HALF_FPS,
            Speed::Normal => io_control.target_fps = DEFAULT_FPS,
            Speed::Double => io_control.target_fps = DOUBLE_FPS,
            Speed::Increase => io_control.target_fps += 5,
            Speed::Decrease => {
                io_control.target_fps = std::cmp::max(0, io_control.target_fps as i32 - 5) as u16
            }
        }
        nes.config().set_target_fps(io_control.target_fps);
    }

    for (i, controller_type) in io_state.switch_controller_type.iter().enumerate() {
        if let Some(controller_type) = controller_type
            && let Some(id) = ControllerId::from_index(i)
        {
            nes.config().set_controller(id, *controller_type);
        }
    }
    nes.config().set_audio_volume(io_state.audio_volume);
}

fn load(nes: &mut Nes, path: &str) -> Result<(), String> {
    let rom = get_bytes_from_file(path)?;
    nes.load_rom(&rom).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_demo(nes: &mut Nes) {
    let demo_rom = read_demo();
    nes.load_rom(&demo_rom).unwrap_err();
}

fn main() {
    let emulation = Emulation::new();
    let emulation = match emulation {
        Ok(emulation) => emulation,
        Err(e) => {
            eprintln!("Error initializing emulation: {}", e);
            return;
        }
    };

    #[cfg(target_os = "emscripten")]
    emscripten_main_loop::run(emulation);
    #[cfg(not(target_os = "emscripten"))]
    run(emulation);
}
