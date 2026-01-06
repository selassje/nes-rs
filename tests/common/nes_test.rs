use nes_rs::{ControllerId, Nes, StdNesControllerButton, FRAME_HEIGHT, FRAME_WIDTH, PIXEL_SIZE};

use fs::File;
use sdl2::{
    pixels::{self, PixelFormatEnum},
    rect::Rect,
};
use std::{cell::RefCell, fs, io::Read, path::Path, path::PathBuf, rc::Rc, time::Duration};
type TestFn = dyn Fn(&mut NesTest);

fn get_bytes_from_file(file_name: &str) -> Vec<u8> {
    let mut rom = Vec::new();
    let mut file = File::open(file_name).unwrap_or_else(|_| {
        panic!(
            "Unable to open ROM {} current dir {}",
            file_name,
            std::env::current_dir().unwrap().display()
        )
    });
    file.read_to_end(&mut rom).expect("Unable to read ROM");
    rom
  }

pub struct NesTest {
    nes: Nes,
    io_test: Rc<RefCell<super::io_test::IOTest>>,
    output_frame_path: String,
    expected_frame_path: String,
    test_fn: Rc<TestFn>,
}

impl NesTest {
    fn create_frame_path(dir: &Path, test_name: &str, suffix: &str) -> String {
        let frame_path = dir.join(Path::new(&(test_name.to_owned() + suffix + ".bmp")));
        let frame_path = frame_path.to_str().unwrap();
        String::from(frame_path)
    }

    pub fn new(
        rom_path: &str,
        suffix: Option<&str>,
        test_fn: impl Fn(&mut NesTest) + 'static,
    ) -> Self {
        let io_test = Rc::new(RefCell::new(super::io_test::IOTest::new(rom_path)));
        let rom = get_bytes_from_file(rom_path);
        let mut nes = Nes::new();
        nes.config().set_controller_access(io_test.clone());
        let mut dir = PathBuf::from(rom_path);
        let mut test_name = dir.file_name().unwrap().to_str().unwrap().to_owned();
        if let Some(suffix) = suffix {
            let suffix: String = ".".to_owned() + suffix;
            test_name.push_str(&suffix)
        }

        dir.pop();
        let output_frame_path = Self::create_frame_path(&dir, &test_name, "");
        let expected_frame_path = Self::create_frame_path(&dir, &test_name, ".expected");
        nes.load_rom(&rom);

        NesTest {
            io_test,
            nes,
            output_frame_path,
            expected_frame_path,
            test_fn: Rc::new(test_fn),
        }
    }

    fn delete_output_frame(&self) {
        fs::remove_file(self.output_frame_path.clone()).unwrap_or_default();
    }

    fn frames_are_the_same(&self) -> bool {
        let mut file_1 = File::open(self.output_frame_path.clone())
            .unwrap_or_else(|_| panic!("Unable to open {}", self.output_frame_path));
        let mut file_2 = File::open(self.expected_frame_path.clone())
            .unwrap_or_else(|_| panic!("Unable to open {}", self.expected_frame_path));
        let mut buffer_1 = Vec::new();
        let mut buffer_2 = Vec::new();
        let _ = file_1.read_to_end(&mut buffer_1);
        let _ = file_2.read_to_end(&mut buffer_2);
        buffer_1 == buffer_2
    }

    pub fn run(&mut self) -> bool {
        self.delete_output_frame();
        self.test_fn.clone()(self);
        self.dump_frame();
        self.frames_are_the_same()
    }

    pub fn run_for(&mut self, duration: Duration) {
        self.nes.run_for(duration)
    }

    #[allow(dead_code)]
    pub fn serialize_and_reset(&mut self) -> Vec<u8> {
        let serialized = self.nes.save_state();
        self.nes.power_cycle();
        serialized
    }

    #[allow(dead_code)]
    pub fn deserialize(&mut self, state: Vec<u8>) {
        self.nes.load_state(state);
    }

    #[allow(dead_code)]
    pub fn press_player_1_start(&mut self) {
        self.io_test.borrow_mut().set_button_state(
            StdNesControllerButton::Start,
            ControllerId::Controller1,
            true,
        );
    }
    #[allow(dead_code)]
    pub fn release_player_1_start(&mut self) {
        self.io_test.borrow_mut().set_button_state(
            StdNesControllerButton::Start,
            ControllerId::Controller1,
            false,
        );
    }
    #[allow(dead_code)]
    pub fn press_player_1_select(&mut self) {
        self.io_test.borrow_mut().set_button_state(
            StdNesControllerButton::Select,
            ControllerId::Controller1,
            true,
        );
    }
    #[allow(dead_code)]
    pub fn release_player_1_select(&mut self) {
        self.io_test.borrow_mut().set_button_state(
            StdNesControllerButton::Select,
            ControllerId::Controller1,
            false,
        );
    }

    fn dump_frame(&self) {
        let frame = &self.nes.get_emulation_frame().video;
        let mut bitmap = sdl2::surface::Surface::new(
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        for x in 0..FRAME_WIDTH {
            for y in 0..FRAME_HEIGHT {
                let index = y * PIXEL_SIZE * FRAME_WIDTH + x * PIXEL_SIZE;
                let pixel_color =
                    pixels::Color::RGB(frame[index], frame[index + 1], frame[index + 2]);
                let _ = bitmap.fill_rect(Rect::new(x as i32, y as i32, 1, 1), pixel_color);
            }
        }
        let _ = bitmap.save_bmp(&self.output_frame_path);
    }
}
