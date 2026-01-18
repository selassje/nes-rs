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

#[cfg(target_os = "emscripten")]
unsafe extern "C" {
    fn emscripten_run_script(s: *const std::os::raw::c_char);
}
pub struct Emulation {
    nes: Nes,
    frontend: Sdl2ImGuiOpenGlFrontend,
    frontend_control: FrontendControl,
    frontend_state: FrontendState,
    fps: u16,
    one_second_timer: std::time::Instant,
    error_timer: std::time::Instant,
    frame_start: std::time::Instant,
    is_audio_available: bool,
}
#[allow(clippy::new_without_default)]
impl Emulation {
    pub fn new() -> Result<Self, String> {
        let mut nes: Nes = crate::Nes::new();
        #[cfg(target_os = "emscripten")]
        nes.config().set_audio_target_fps(59.98);

        let mut initial_title: Option<String> = None;
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let path = &args[1];
            load(&mut nes, path).map_err(|e| format!("Error loading ROM {}: {}", path, e))?;
            initial_title = Some(path.clone());
        } else {
            load_demo(&mut nes);
        }
        let frontend = frontend::sdl2_imgui_opengl::Sdl2ImGuiOpenGlFrontend::new();

        let frontend_state: FrontendState = Default::default();
        let frame_start = std::time::Instant::now();
        let fps = 0;
        let one_second_timer = std::time::Instant::now();

        let frontend_control = FrontendControl {
            target_fps: DEFAULT_FPS,
            current_fps: 0,
            title: initial_title,
            controller_type: [crate::ControllerType::NullController; 2],
            error: None,
        };
        let is_audio_available = frontend.is_audio_available();
        Ok(Self {
            nes,
            frontend,
            frontend_control,
            frontend_state,
            fps,
            one_second_timer,
            error_timer: std::time::Instant::now(),
            frame_start,
            is_audio_available,
        })
    }
}

impl emscripten_main_loop::MainLoop for Emulation {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        self.frontend_control.controller_type = [
            self.nes
                .config()
                .get_controller_type(crate::ControllerId::Controller1),
            self.nes
                .config()
                .get_controller_type(crate::ControllerId::Controller2),
        ];
        let mut emulation_frame: Option<&EmulationFrame> = None;
        if !self.frontend_state.pause {
            let emulation_result = self.nes.run_single_frame(&self.frontend);
            match emulation_result {
                Ok(frame) => {
                    emulation_frame = Some(frame);
                }
                Err(e) => {
                    self.frontend_control.error = Some(format!("Emulation error: {}", e));
                    self.frontend_state.pause = true;
                    self.error_timer = std::time::Instant::now();
                }
            }

            if self.one_second_timer.elapsed() < std::time::Duration::from_secs(1) {
                self.fps += 1;
            } else {
                self.one_second_timer = std::time::Instant::now();
                self.frontend_control.current_fps = self.fps;
                self.fps = 1;
            }
        }

        self.frontend_state = self
            .frontend
            .present_frame(self.frontend_control.clone(), emulation_frame);

        handle_io_state(
            &mut self.error_timer,
            &mut self.nes,
            &self.frontend_state,
            &mut self.frontend_control,
        );

        if !self.frontend_state.pause {
            let elapsed_time_since_frame_start = self.frame_start.elapsed();
            let frame_duration: std::time::Duration = std::time::Duration::from_nanos(
                (std::time::Duration::from_secs(1).as_nanos()
                    / (self.frontend_control.target_fps) as u128) as u64,
            );
            if !self.is_audio_available && elapsed_time_since_frame_start < frame_duration {
                #[cfg(not(target_os = "emscripten"))]
                std::thread::sleep(frame_duration - elapsed_time_since_frame_start);
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
    let mut file = File::open(file_name).map_err(|e| e.to_string())?;
    file.read_to_end(&mut rom).map_err(|e| e.to_string())?;
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
    while !emulation.frontend_state.quit {
        emulation.main_loop();
    }
}

fn handle_io_state(
    error_timer: &mut std::time::Instant,
    nes: &mut Nes,
    fontend_state: &FrontendState,
    frontend_control: &mut FrontendControl,
) {
    if fontend_state.power_cycle {
        nes.power_cycle();
    }

    if let Some(ref nes_file_path) = fontend_state.load_nes_file {
        let load_result = load(nes, nes_file_path.as_str());
        if load_result.is_ok() {
            frontend_control.title = Some(nes_file_path.clone());
        } else {
            frontend_control.error = Some(load_result.err().unwrap());
            *error_timer = std::time::Instant::now();
        }
    }
    if frontend_control.error.is_some() && error_timer.elapsed() > std::time::Duration::from_secs(5)
    {
        frontend_control.error = None;
    }

    if let Some(ref save_state_path) = fontend_state.save_state {
        let serialized = nes.save_state();
        if serialized.is_err() {
            frontend_control.error =
                Some(format!("Error saving state: {}", serialized.err().unwrap()));
            *error_timer = std::time::Instant::now();
            return;
        }
        let file_name = save_state_path.as_str();
        let file = File::create(file_name);
        if file.is_err() {
            frontend_control.error = Some(format!(
                "Unable to create save file {}: {}",
                file_name,
                file.err().unwrap()
            ));
            *error_timer = std::time::Instant::now();
            return;
        }
        file.unwrap()
            .write_all(serialized.unwrap().as_slice())
            .unwrap();
        #[cfg(target_os = "emscripten")]
        unsafe {
            let script = std::ffi::CString::new("refreshSaveFilesList();").unwrap();
            emscripten_run_script(script.as_ptr());
        };
    }

    if let Some(ref load_state_path) = fontend_state.load_state {
        let file_name = load_state_path.as_str();
        let save = std::fs::read(file_name).unwrap_or_else(|_| {
            panic!(
                "Unable to open save file {} current dir {}",
                file_name,
                std::env::current_dir().unwrap().display()
            )
        });
        let load_state_result = nes.load_state(save);
        if let Err(e) = load_state_result {
            frontend_control.error = Some(format!("Error loading state: {}", e));
            *error_timer = std::time::Instant::now();
        }
        frontend_control.title = Some(load_state_path.clone());
    }

    if let Some(ref speed) = fontend_state.speed {
        match speed {
            Speed::Half => frontend_control.target_fps = HALF_FPS,
            Speed::Normal => frontend_control.target_fps = DEFAULT_FPS,
            Speed::Double => frontend_control.target_fps = DOUBLE_FPS,
            Speed::Increase => frontend_control.target_fps += 5,
            Speed::Decrease => {
                frontend_control.target_fps =
                    std::cmp::max(0, frontend_control.target_fps as i32 - 5) as u16
            }
        }
        #[cfg(not(target_os = "emscripten"))]
        nes.config()
            .set_audio_target_fps(frontend_control.target_fps as f32);
    }

    for (i, controller_type) in fontend_state.switch_controller_type.iter().enumerate() {
        if let Some(controller_type) = controller_type
            && let Some(id) = ControllerId::from_index(i)
        {
            nes.config().set_controller(id, *controller_type);
        }
    }
    nes.config().set_audio_volume(fontend_state.audio_volume);
}

fn load(nes: &mut Nes, path: &str) -> Result<(), String> {
    let rom = get_bytes_from_file(path)?;
    nes.load_rom(&rom).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_demo(nes: &mut Nes) {
    let demo_rom = read_demo();
    nes.load_rom(&demo_rom).expect("Error loading demo ROM");
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
