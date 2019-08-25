use cursive::theme::{Color, ColorStyle, BorderStyle};
use cursive::traits::*;
use cursive::view::Boxable;
use cursive::Cursive;
use cursive::Printer;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

type Screen = [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH];

struct ScreenView {
    screen: Screen,
}

impl View for ScreenView {
    fn draw(&self, printer: &Printer) {
        let style = ColorStyle::new(Color::from_256colors(255), Color::from_256colors(0));

        for (x, row) in self.screen.iter().enumerate() {
            for (y, _) in row.iter().enumerate() {           
                printer.with_color(style, |printer| {
                    if self.screen[x][y] {
                        printer.print((x, y), "*");
                    } else {printer.print((x, y), " ")};
                });
            }
        }
    }
}

pub fn update_screen_view(siv: &mut Cursive, x: usize, y: usize, val: bool) {
     siv.call_on_id("display", |view : &mut ScreenView |{view.screen[x][y] = val;});
}

pub fn create_display() -> Cursive {

    let mut siv = Cursive::default();
    let screen_view = ScreenView {
        screen: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    };

    siv.add_global_callback('q', Cursive::quit);

    let mut theme = siv.current_theme().clone();
    theme.shadow = false;
    theme.borders = BorderStyle::None;
    theme.palette.set_color("background",Color::from_256colors(0));
    siv.add_layer(
        screen_view.with_id("display")
            .fixed_width(DISPLAY_WIDTH)
            .fixed_height(DISPLAY_HEIGHT)
            
    );
    siv.set_theme(theme);
    siv
}
