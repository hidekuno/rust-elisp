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
use std::rc::Rc;

const DEFALUT_CANVAS: &str = "canvas";
type DrawImage = Box<dyn Fn(f64, f64, f64, f64, f64, f64, &ImageSurface) + 'static>;

pub type ImageTable = Rc<RefCell<HashMap<String, Rc<ImageSurface>>>>;
pub type DrawLine = Box<dyn Fn(f64, f64, f64, f64) + 'static>;

#[cfg(feature = "animation")]
macro_rules! force_event_loop {
    () => {
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    };
}
pub fn get_default_surface(image_table: &ImageTable) -> Rc<ImageSurface> {
    image_table
        .borrow()
        .get(&DEFALUT_CANVAS.to_string())
        .unwrap()
        .clone()
}
pub fn draw_clear(image_table: &ImageTable) {
    let surface = get_default_surface(image_table);
    let cr = &Context::new(&*surface);
    cr.transform(Matrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: 1.0,
        x0: 0.0,
        y0: 0.0,
    });
    cr.set_source_rgb(0.9, 0.9, 0.9);
    cr.paint();
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and create draw_line
// ----------------------------------------------------------------
pub fn create_draw_line(image_table: &ImageTable) -> DrawLine {
    let surface = get_default_surface(image_table);
    let cr = Context::new(&*surface);
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(0.001);

    let draw_line = move |x0, y0, x1, y1| {
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
pub fn create_draw_image(image_table: &ImageTable) -> DrawImage {
    let surface = get_default_surface(image_table);
    let draw_image = move |x0, y0, x1, y1, xorg, yorg, img: &ImageSurface| {
        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.move_to(0.0, 0.0);
        let matrix = Matrix {
            xx: x0,
            yx: y0,
            xy: x1,
            yy: y1,
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
pub fn create_image_table() -> ImageTable {
    let mut image_table = HashMap::new();

    let surface = ImageSurface::create(Format::ARgb32, DRAW_WIDTH, DRAW_HEIGHT)
        .expect("Can't create surface");

    let cr = Context::new(&surface);
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
    cr.set_source_rgb(0.9, 0.9, 0.9);
    cr.paint();
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_font_size(0.25);

    cr.move_to(0.04, 0.50);
    cr.show_text("Rust");

    cr.move_to(0.27, 0.69);
    cr.text_path("eLisp");
    cr.set_source_rgb(0.5, 0.5, 1.0);
    cr.fill_preserve();
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(0.01);
    cr.stroke();

    image_table.insert(DEFALUT_CANVAS.to_string(), Rc::new(surface));

    Rc::new(RefCell::new(image_table))
}
