use crate::{controllers::Button, io::io_test, keyboard, nes::Nes, read_rom};
use fs::File;
use std::{cell::RefCell, fs, io::Read, path::Path, path::PathBuf, rc::Rc, time::Duration};

type TestFn = dyn Fn(&mut NesTest);

pub struct NesTest {
    nes: Nes,
    io_test: Rc<RefCell<io_test::IOTest>>,
    output_frame_path: String,
    expected_frame_path: String,
    test_fn: Rc<TestFn>,
}

impl NesTest {
    fn create_frame_path(dir: &PathBuf, test_name: &str, suffix: &str) -> String {
        let frame_path = dir.join(Path::new(&(test_name.to_owned() + suffix + ".bmp")));
        let frame_path = frame_path.to_str().unwrap();
        String::from(frame_path)
    }

    pub fn new(
        rom_path: &str,
        suffix: Option<&str>,
        test_fn: impl Fn(&mut NesTest) + 'static,
    ) -> Self {
        let io_test = Rc::new(RefCell::new(io_test::IOTest::new(rom_path)));
        let controller_1 = Rc::new(
            keyboard::KeyboardController::get_default_keyboard_controller_player1(io_test.clone()),
        );
        let controller_2 = Rc::new(
            keyboard::KeyboardController::get_default_keyboard_controller_player1(io_test.clone()),
        );
        io_test.borrow_mut().set_key_mappings(
            controller_1.get_key_mappings(),
            controller_2.get_key_mappings(),
        );
        let mut nes = Nes::new(io_test.clone(), controller_1, controller_2);
        let mut dir = PathBuf::from(rom_path);
        let mut test_name = dir.file_name().unwrap().to_str().unwrap().to_owned();
        if let Some(suffix) = suffix {
            let suffix: String = ".".to_owned() + suffix;
            test_name.extend(suffix.chars())
        }

        dir.pop();
        let output_frame_path = Self::create_frame_path(&dir, &test_name, "");
        let expected_frame_path = Self::create_frame_path(&dir, &test_name, ".expected");
        let nes_file = read_rom(rom_path);
        nes.load(&nes_file);

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
            .expect(&format!("Unable to open {}", self.output_frame_path));
        let mut file_2 = File::open(self.expected_frame_path.clone())
            .expect(&format!("Unable to open {}", self.expected_frame_path));
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
        self.nes.run(Some(duration))
    }

    pub fn press_player_1_start(&mut self) {
        self.io_test
            .borrow_mut()
            .set_button_state(Button::Start, io_test::Player::Player1, true);
    }

    pub fn press_player_1_select(&mut self) {
        self.io_test
            .borrow_mut()
            .set_button_state(Button::Select, io_test::Player::Player1, true);
    }

    pub fn release_player_1_select(&mut self) {
        self.io_test
            .borrow_mut()
            .set_button_state(Button::Select, io_test::Player::Player1, false);
    }

    fn dump_frame(&self) {
        self.io_test.borrow().dump_frame(&self.output_frame_path)
    }
}
