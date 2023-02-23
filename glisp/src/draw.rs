/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate gtk;

use crate::ui::DRAW_HEIGHT;
use crate::ui::DRAW_WIDTH;
use elisp::create_error;
use elisp::draw::DrawArc;
use elisp::draw::DrawImage;
use elisp::draw::DrawLine;
use elisp::lisp::ErrCode;
use elisp::lisp::Error;
use gtk::cairo;
use gtk::gdk;
use gtk::gdk_pixbuf;

use cairo::{Context, Format, ImageSurface, Matrix};
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

const DEFALUT_LINE_WIDTH: f64 = 0.001;
const DEFALUT_BG_COLOR: (f64, f64, f64) = (0.9, 0.9, 0.9);
const DEFALUT_FG_COLOR: (f64, f64, f64) = (0.0, 0.0, 0.0);

const CAIRO_ERR_MSG: &str = "Invalid cairo state";
// ----------------------------------------------------------------
// Color table
// ----------------------------------------------------------------
struct Color {
    red: f64,
    green: f64,
    blue: f64,
}
impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Color { red, green, blue }
    }
}
// ----------------------------------------------------------------
// Graphics table
// ----------------------------------------------------------------
struct Graphics {
    image_table: HashMap<String, Rc<dyn ImageData>>,
    line_width: f64,
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
// ----------------------------------------------------------------
// Draw table manage
// ----------------------------------------------------------------
#[derive(Clone)]
pub struct DrawTable {
    core: Rc<RefCell<Graphics>>,
    surface: Rc<ImageSurface>,
}
impl DrawTable {
    pub fn regist(&self, key: String, surface: Rc<dyn ImageData>) {
        self.core.borrow_mut().image_table.insert(key, surface);
    }
    pub fn find(&self, key: &str) -> Option<Rc<dyn ImageData>> {
        self.core.borrow().image_table.get(key).cloned()
    }
    pub fn set_background(&mut self, red: f64, green: f64, blue: f64) {
        self.core.borrow_mut().set_background(red, green, blue);
    }
    pub fn set_foreground(&mut self, red: f64, green: f64, blue: f64) {
        self.core.borrow_mut().set_foreground(red, green, blue);
    }
    pub fn set_line_width(&mut self, w: f64) {
        self.core.borrow_mut().line_width = w;
    }
    pub fn get_default_surface(&self) -> Rc<ImageSurface> {
        self.surface.clone()
    }
    pub fn set_cairo_surface(&self, cr: &Context) {
        cr.set_source_surface(&*self.surface, 0.0, 0.0)
            .expect(CAIRO_ERR_MSG);

        cr.paint().expect(CAIRO_ERR_MSG);
    }
}
// ----------------------------------------------------------------
// Image data
// ----------------------------------------------------------------
pub trait ImageData {
    fn get_width(&self) -> f64;
    fn get_height(&self) -> f64;
    fn set_context_image(&self, cr: &Context);
}
pub struct ImageSurfaceWrapper {
    surface: ImageSurface,
}
impl ImageSurfaceWrapper {
    pub fn new(surface: ImageSurface) -> ImageSurfaceWrapper {
        ImageSurfaceWrapper { surface }
    }
}
impl ImageData for ImageSurfaceWrapper {
    fn get_width(&self) -> f64 {
        self.surface.width() as f64
    }
    fn get_height(&self) -> f64 {
        self.surface.height() as f64
    }
    fn set_context_image(&self, cr: &Context) {
        cr.set_source_surface(&self.surface, 0.0, 0.0)
            .expect(CAIRO_ERR_MSG);
    }
}
pub struct PixbufWrapper {
    pixbuf: Pixbuf,
}
impl PixbufWrapper {
    pub fn new(pixbuf: Pixbuf) -> PixbufWrapper {
        PixbufWrapper { pixbuf }
    }
}
impl ImageData for PixbufWrapper {
    fn get_width(&self) -> f64 {
        self.pixbuf.width() as f64
    }
    fn get_height(&self) -> f64 {
        self.pixbuf.height() as f64
    }
    fn set_context_image(&self, cr: &Context) {
        cr.set_source_pixbuf(&self.pixbuf, 0.0, 0.0);
    }
}
// ----------------------------------------------------------------
// rakugaki
// ----------------------------------------------------------------
pub struct Graffiti {
    cr: Context,
}
impl Graffiti {
    pub fn new(draw_table: &DrawTable) -> Self {
        let surface = draw_table.get_default_surface();
        Graffiti {
            cr: Context::new(&*surface).unwrap(),
        }
    }
    pub fn start_graffiti(&self, x: f64, y: f64) {
        self.cr.scale(1.0, 1.0);
        self.cr.set_line_width(1.5);
        self.cr.move_to(x, y);
    }
    pub fn draw_graffiti(&self, x: f64, y: f64) {
        self.cr.line_to(x, y);
        self.cr.stroke().expect(CAIRO_ERR_MSG);
        self.cr.move_to(x, y);
    }
    pub fn stop_graffiti(&self, x: f64, y: f64) {
        self.cr.line_to(x, y);
        self.cr.stroke().expect(CAIRO_ERR_MSG);
    }
}
// ----------------------------------------------------------------
// animation macro
// ----------------------------------------------------------------
#[cfg(feature = "animation")]
macro_rules! force_event_loop {
    () => {
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    };
}
// ----------------------------------------------------------------
// screen clear
// ----------------------------------------------------------------
pub fn draw_clear(draw_table: &DrawTable) {
    let surface = draw_table.get_default_surface();
    let cr = &Context::new(&*surface).unwrap();
    cr.transform(Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));
    let bg = &draw_table.core.borrow().bg;
    cr.set_source_rgb(bg.red, bg.green, bg.blue);
    cr.paint().expect(CAIRO_ERR_MSG);
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and draw line
// ----------------------------------------------------------------
#[allow(unused_variables)]
pub fn create_draw_line(draw_table: &DrawTable, redraw_times: usize) -> DrawLine {
    let surface = draw_table.get_default_surface();
    let cr = Context::new(&*surface).unwrap();
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);

    let draw_table = draw_table.clone();

    #[cfg(feature = "animation")]
    let count = RefCell::new(0);

    let draw_line = move |x0, y0, x1, y1| {
        let fg = &draw_table.core.borrow().fg;
        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.set_line_width(draw_table.core.borrow().line_width);
        cr.move_to(x0, y0);
        cr.line_to(x1, y1);
        cr.stroke().expect(CAIRO_ERR_MSG);

        #[cfg(feature = "animation")]
        {
            let c = count.try_borrow_mut();
            if c.is_err() {
                return Err(create_error!(ErrCode::E9002));
            }
            let mut c = c.unwrap();
            *c += 1;

            if 0 == (*c % redraw_times) {
                force_event_loop!();
            }
        }
        Ok(())
    };
    Box::new(draw_line)
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and draw image
// ----------------------------------------------------------------
pub fn create_draw_image(draw_table: &DrawTable) -> DrawImage {
    let surface = draw_table.get_default_surface();
    let draw_table = draw_table.clone();

    let draw_image = move |x0, y0, x1, y1, xorg, yorg, symbol: &String| {
        let img = match draw_table.find(symbol) {
            Some(v) => v.clone(),
            None => return Err(create_error!(ErrCode::E1008)),
        };

        let cr = Context::new(&*surface).unwrap();
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.move_to(0.0, 0.0);

        let matrix = Matrix::new(
            x0 / img.get_width(),
            y0 / img.get_height(),
            x1 / img.get_width(),
            y1 / img.get_height(),
            xorg,
            yorg,
        );
        cr.transform(matrix);

        img.set_context_image(&cr);
        cr.paint().expect(CAIRO_ERR_MSG);

        #[cfg(feature = "animation")]
        force_event_loop!();

        Ok(())
    };
    Box::new(draw_image)
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and draw string
// ----------------------------------------------------------------
pub fn create_draw_string(draw_table: &DrawTable) -> Box<dyn Fn(f64, f64, f64, String) + 'static> {
    let surface = draw_table.get_default_surface();

    let draw_table = draw_table.clone();
    let draw_string = move |x, y, f, s: String| {
        let fg = &draw_table.core.borrow().fg;
        let cr = Context::new(&*surface).unwrap();
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.move_to(x, y);
        cr.set_font_size(f);
        cr.show_text(s.as_str()).expect(CAIRO_ERR_MSG);

        cr.stroke().expect(CAIRO_ERR_MSG);
        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_string)
}
// ----------------------------------------------------------------
// create new cairo from imagetable, and draw arc
// ----------------------------------------------------------------
pub fn create_draw_arc(draw_table: &DrawTable) -> DrawArc {
    let surface = draw_table.get_default_surface();
    let draw_table = draw_table.clone();

    let draw_arc = move |x, y, r, a| {
        let fg = &draw_table.core.borrow().fg;
        let cr = Context::new(&*surface).unwrap();

        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);

        cr.set_source_rgb(fg.red, fg.green, fg.blue);
        cr.set_line_width(draw_table.core.borrow().line_width);
        cr.arc(x, y, r, a, PI * 2.);
        cr.stroke().expect(CAIRO_ERR_MSG);

        #[cfg(feature = "animation")]
        force_event_loop!();
    };
    Box::new(draw_arc)
}
// ----------------------------------------------------------------
// save surface(as PNG file)
// ----------------------------------------------------------------
pub fn save_png_file(draw_table: &DrawTable, filename: &Path, overwrite: bool) -> String {
    if filename.exists() && !overwrite {
        return format!("\"{}\" is exists", filename.to_str().unwrap());
    }
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            return e.to_string();
        }
    };
    let surface = draw_table.get_default_surface();
    match surface.write_to_png(&mut file) {
        Ok(_) => format!("Saved \"{}\"", filename.to_str().unwrap()),
        Err(e) => e.to_string(),
    }
}
// ----------------------------------------------------------------
// create draw table
// ----------------------------------------------------------------
pub fn create_draw_table() -> DrawTable {
    let surface = ImageSurface::create(Format::ARgb32, DRAW_WIDTH, DRAW_HEIGHT)
        .expect("Can't create surface");
    let fg = Color::new(DEFALUT_FG_COLOR.0, DEFALUT_FG_COLOR.1, DEFALUT_FG_COLOR.2);
    let bg = Color::new(DEFALUT_BG_COLOR.0, DEFALUT_BG_COLOR.1, DEFALUT_BG_COLOR.2);

    let cr = Context::new(&surface).unwrap();
    cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
    cr.set_source_rgb(bg.red, bg.green, bg.blue);
    cr.paint().expect(CAIRO_ERR_MSG);

    cr.set_source_rgb(fg.red, fg.green, fg.blue);
    cr.move_to(0.04, 0.50);
    cr.set_font_size(0.25);
    cr.show_text("Rust").expect(CAIRO_ERR_MSG);

    cr.move_to(0.27, 0.69);
    cr.text_path("eLisp");
    cr.set_source_rgb(0.5, 0.5, 1.0);
    cr.fill_preserve().expect(CAIRO_ERR_MSG);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(0.01);
    cr.stroke().expect(CAIRO_ERR_MSG);

    DrawTable {
        core: Rc::new(RefCell::new(Graphics {
            image_table: HashMap::new(),
            line_width: DEFALUT_LINE_WIDTH,
            fg,
            bg,
        })),
        surface: Rc::new(surface),
    }
}
