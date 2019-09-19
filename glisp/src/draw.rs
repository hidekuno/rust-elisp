/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate cairo;
extern crate elisp;
extern crate gtk;

use crate::ui::DRAW_HEIGHT;
use crate::ui::DRAW_WIDTH;
use cairo::{Context, Format, ImageSurface, Matrix};
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

const DEFALUT_CANVAS: &str = "canvas";
pub type DrawImage = Box<dyn Fn(f64, f64, f64, f64, f64, f64, &ImageSurface) + 'static>;
pub type DrawLine = Box<dyn Fn(f64, f64, f64, f64) + 'static>;
pub type DrawString = Box<dyn Fn(f64, f64, f64, String) + 'static>;
pub type DrawArc = Box<dyn Fn(f64, f64, f64, f64) + 'static>;

struct Color {
    red: f64,
    green: f64,
    blue: f64,
}
impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Color {
            red: red,
            green: green,
            blue: blue,
        }
    }
}
struct Graphics {
    image_table: HashMap<String, Rc<ImageSurface>>,
    fg: Color,
    bg: Color,
}
impl Graphics {
    pub fn set_background(&mut self, red: f64, green: f64, blue: f64) {
        self.bg.red = red;
        self.bg.green = green;
        self.bg.blue = blue;
    }
    pub fn set_foreground(&mut self, red: f64, green: f64, blue: f64) {
        self.fg.red = red;
        self.fg.green = green;
        self.fg.blue = blue;
    }
}
#[derive(Clone)]
pub struct DrawTable {
    core: Rc<RefCell<Graphics>>,
}
impl DrawTable {
    pub fn regist(&self, key: String, surface: Rc<ImageSurface>) {
        self.core.borrow_mut().image_table.insert(key, surface);
    }
    pub fn find(&self, key: &String) -> Option<Rc<ImageSurface>> {
        match self.core.borrow().image_table.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    pub fn set_background(&mut self, red: f64, green: f64, blue: f64) {
        self.core.borrow_mut().set_background(red, green, blue);
    }
    pub fn set_foreground(&mut self, red: f64, green: f64, blue: f64) {
        self.core.borrow_mut().set_foreground(red, green, blue);
    }
}
#[cfg(feature = "animation")]
macro_rules! force_event_loop {
    () => {
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    };
}
pub fn get_default_surface(draw_table: &DrawTable) -> Rc<ImageSurface> {
    draw_table
        .find(&DEFALUT_CANVAS.to_string())
        .unwrap()
        .clone()
}
// ----------------------------------------------------------------
// rakugaki
// ----------------------------------------------------------------
pub fn draw_graffiti(draw_table: &DrawTable, x: f64, y: f64) {
    let surface = get_default_surface(draw_table);
    let cr = Context::new(&*surface);
    cr.scale(1.0, 1.0);
    let fg = &draw_table.core.borrow().fg;
    cr.set_source_rgb(fg.red, fg.green, fg.blue);

    cr.rectangle(x - 3.0, y - 3.0, 4.0, 4.0);
    cr.fill();
}
// ----------------------------------------------------------------
// screen clear
// ----------------------------------------------------------------
pub fn draw_clear(draw_table: &DrawTable) {
    let surface = get_default_surface(draw_table);
    let cr = &Context::new(&*surface);
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: 1.0,
        x0: 0.0,
        y0: 0.0,
    });
    let bg = &draw_table.core.borrow().bg;
    cr.set_source_rgb(bg.red, bg.green, bg.blue);
    cr.paint();
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and create draw_line
// ----------------------------------------------------------------
pub fn create_draw_line(draw_table: &DrawTable) -> DrawLine {
    let surface = get_default_surface(draw_table);
    let cr = Context::new(&*surface);
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
    cr.set_line_width(0.001);

    let draw_table = draw_table.clone();
    let draw_line = move |x0, y0, x1, y1| {
        let fg = &draw_table.core.borrow().fg;
        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.move_to(x0, y0);
        cr.line_to(x1, y1);
        cr.stroke();
        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_line)
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and create draw_image
// ----------------------------------------------------------------
pub fn create_draw_image(draw_table: &DrawTable) -> DrawImage {
    let surface = get_default_surface(draw_table);
    let draw_image = move |x0, y0, x1, y1, xorg, yorg, img: &ImageSurface| {
        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.move_to(0.0, 0.0);
        let matrix = Matrix {
            xx: x0 / img.get_width() as f64,
            yx: y0 / img.get_height() as f64,
            xy: x1 / img.get_width() as f64,
            yy: y1 / img.get_height() as f64,
            x0: xorg,
            y0: yorg,
        };
        cr.transform(matrix);
        cr.set_source_surface(&*img, 0.0, 0.0);
        cr.paint();
        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_image)
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and create draw_image
// ----------------------------------------------------------------
pub fn create_draw_string(draw_table: &DrawTable) -> DrawString {
    let surface = get_default_surface(draw_table);

    let draw_table = draw_table.clone();
    let draw_string = move |x, y, f, s: String| {
        let fg = &draw_table.core.borrow().fg;
        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.move_to(x, y);
        cr.set_font_size(f);
        cr.show_text(s.as_str());

        cr.stroke();
        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_string)
}
pub fn create_draw_arc(draw_table: &DrawTable) -> DrawArc {
    let surface = get_default_surface(draw_table);
    let draw_table = draw_table.clone();

    let draw_arc = move |x, y, r, a| {
        let fg = &draw_table.core.borrow().fg;
        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.arc(x, y, r, a, PI * 2.);
        cr.fill();
        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_arc)
}
pub fn create_draw_table() -> DrawTable {
    let mut image_table = HashMap::new();

    let surface = ImageSurface::create(Format::ARgb32, DRAW_WIDTH, DRAW_HEIGHT)
        .expect("Can't create surface");
    let fg = Color::new(0.0, 0.0, 0.0);
    let bg = Color::new(0.9, 0.9, 0.9);

    let cr = Context::new(&surface);
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
    cr.set_source_rgb(bg.red, bg.green, bg.blue);
    cr.paint();

    cr.set_source_rgb(fg.red, fg.green, fg.blue);
    cr.move_to(0.04, 0.50);
    cr.set_font_size(0.25);
    cr.show_text("Rust");

    cr.move_to(0.27, 0.69);
    cr.text_path("eLisp");
    cr.set_source_rgb(0.5, 0.5, 1.0);
    cr.fill_preserve();
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(0.01);
    cr.stroke();

    image_table.insert(DEFALUT_CANVAS.to_string(), Rc::new(surface));

    DrawTable {
        core: Rc::new(RefCell::new(Graphics {
            image_table: image_table,
            fg: fg,
            bg: bg,
        })),
    }
}
