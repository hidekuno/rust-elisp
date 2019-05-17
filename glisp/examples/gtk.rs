extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;

use std::f64::consts::PI;
use std::thread;

use cairo::ImageSurface;
use cairo::Matrix;
use gtk::prelude::*;
use std::fs::File;

pub struct Koch {
    sender: glib::Sender<(f64, f64, f64, f64)>,
    sin60: f64,
    cos60: f64,
    scale: i64,
}
impl Koch {
    fn new(txc: glib::Sender<(f64, f64, f64, f64)>, c: i64) -> Koch {
        Koch {
            sender: txc,
            sin60: ((std::f64::consts::PI * 60.0) / 180.0).sin(),
            cos60: ((std::f64::consts::PI * 60.0) / 180.0).cos(),
            scale: c,
        }
    }
    fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i64) {
        if c > 1 {
            let xa = (x0 * 2.0 + x1) / 3.0;
            let ya = (y0 * 2.0 + y1) / 3.0;
            let xb = (x1 * 2.0 + x0) / 3.0;
            let yb = (y1 * 2.0 + y0) / 3.0;

            let yc = ya + (xb - xa) * self.sin60 + (yb - ya) * self.cos60;
            let xc = xa + (xb - xa) * self.cos60 - (yb - ya) * self.sin60;

            self.draw(x0, y0, xa, ya, c - 1);
            self.draw(xa, ya, xc, yc, c - 1);
            self.draw(xc, yc, xb, yb, c - 1);
            self.draw(xb, yb, x1, y1, c - 1);
        } else {
            self.sender
                .send((x0, y0, x1, y1))
                .expect("Couldn't send data to channel");
        }
    }
    fn execute(&self) {
        self.draw(259.0, 0.0, 34.0, 390.0, self.scale);
        self.draw(34.0, 390.0, 483.0, 390.0, self.scale);
        self.draw(483.0, 390.0, 259.0, 0.0, self.scale);
    }
}
pub struct Tree {
    sender: glib::Sender<(f64, f64, f64, f64)>,
    cs: f64,
    sn: f64,
    scale: i64,
}
impl Tree {
    fn new(txc: glib::Sender<(f64, f64, f64, f64)>, c: i64) -> Tree {
        Tree {
            sender: txc,
            cs: ((std::f64::consts::PI * 15.0) / 180.0).cos(),
            sn: ((std::f64::consts::PI * 45.0) / 180.0).sin(),
            scale: c,
        }
    }
    fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i64) {
        let alpha = 0.6;

        self.sender
            .send((x0, y0, x1, y1))
            .expect("Couldn't send data to channel");

        let ya = y1 + self.sn * (x1 - x0) * alpha + self.cs * (y1 - y0) * alpha;
        let xa = x1 + self.cs * (x1 - x0) * alpha - self.sn * (y1 - y0) * alpha;

        let yb = y1 + (-self.sn * (x1 - x0)) * alpha + self.cs * (y1 - y0) * alpha;
        let xb = x1 + self.cs * (x1 - x0) * alpha + self.sn * (y1 - y0) * alpha;

        if 0 >= c {
            self.sender
                .send((x1, y1, xa, ya))
                .expect("Couldn't send data to channel");
            self.sender
                .send((x1, y1, xb, yb))
                .expect("Couldn't send data to channel");
        } else {
            self.draw(x1, y1, xa, ya, c - 1);
            self.draw(x1, y1, xb, yb, c - 1);
        }
    }
    fn execute(&self) {
        self.draw(300.0, 400.0, 300.0, 300.0, self.scale);
    }
}
pub struct Sierpinski {
    sender: glib::Sender<(f64, f64, f64, f64)>,
    scale: i64,
}
impl Sierpinski {
    fn new(txc: glib::Sender<(f64, f64, f64, f64)>, c: i64) -> Sierpinski {
        Sierpinski {
            sender: txc,
            scale: c,
        }
    }
    fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, c: i64) {
        if c > 1 {
            let xx0 = (x0 + x1) / 2.0;
            let yy0 = (y0 + y1) / 2.0;
            let xx1 = (x1 + x2) / 2.0;
            let yy1 = (y1 + y2) / 2.0;
            let xx2 = (x2 + x0) / 2.0;
            let yy2 = (y2 + y0) / 2.0;

            self.draw(x0, y0, xx0, yy0, xx2, yy2, c - 1);
            self.draw(x1, y1, xx0, yy0, xx1, yy1, c - 1);
            self.draw(x2, y2, xx2, yy2, xx1, yy1, c - 1);
        } else {
            self.sender
                .send((x0, y0, x1, y1))
                .expect("Couldn't send data to channel");
            self.sender
                .send((x1, y1, x2, y2))
                .expect("Couldn't send data to channel");
            self.sender
                .send((x2, y2, x0, y0))
                .expect("Couldn't send data to channel");
        }
    }
    fn execute(&self) {
        self.draw(319.0, 40.0, 30.0, 430.0, 609.0, 430.0, self.scale);
    }
}

fn scheme_gtk() {
    gtk::init().expect("Failed to initialize GTK.");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("Rust Lisp");
    window.set_position(gtk::WindowPosition::Center);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });

    //--------------------------------------------------------
    // GtkVBox
    //--------------------------------------------------------
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.set_border_width(5);

    //--------------------------------------------------------
    // DrawingArea
    //--------------------------------------------------------
    let canvas = gtk::DrawingArea::new();
    canvas.set_size_request(720, 560);
    canvas.connect_draw(|_, cr| {
        cr.scale(2.5, 2.5);
        // let mut file = File::open("glenda.png").expect("Couldn't create 'duke.png'");
        // let mut file = File::open("duke.png").expect("Couldn't create 'duke.png'");
        let mut file = File::open("sicp.png").expect("Couldn't create 'sicp.png'");
        let surface = ImageSurface::create_from_png(&mut file).expect("Can't create surface");
        cr.move_to(0.0, 0.0);
        // cr.translate(560.0, 1.0);
        // cr.scale(-1.0, 1.0);
        // cr.translate(1.0, 300.0);
        // cr.scale(1.0, -1.0);
        let matrix = Matrix {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: 1.0,
            x0: 100.0,
            y0: 100.0,
        };
        cr.transform(matrix);
        cr.set_source_surface(&surface, 0.0, 0.0);
        cr.paint();

        cr.scale(720 as f64, 560 as f64);
        cr.set_font_size(0.25);
        cr.move_to(0.27, 0.83);
        cr.show_text("Rust");

        cr.move_to(0.27, 0.75);
        cr.text_path("eLisp");

        cr.set_source_rgb(0.5, 0.5, 1.0);
        cr.fill_preserve();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(0.01);
        cr.stroke();

        cr.set_source_rgba(1.0, 0.2, 0.2, 0.6);
        cr.arc(0.04, 0.53, 0.02, 0.0, PI * 2.);
        cr.arc(0.27, 0.65, 0.02, 0.0, PI * 2.);
        cr.fill();

        Inhibit(false)
    });
    //--------------------------------------------------------
    // TextView
    //--------------------------------------------------------
    let text_view = gtk::TextView::new();
    let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add(&text_view);
    scroll.set_size_request(720, 240);

    //--------------------------------------------------------
    // GtkMenuBar
    //--------------------------------------------------------
    let menu_bar = gtk::MenuBar::new();
    let menu = gtk::Menu::new();
    let file = gtk::MenuItem::new_with_label("File");
    let draw = gtk::MenuItem::new_with_label("Draw");
    let quit = gtk::MenuItem::new_with_label("Quit");

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let clear_canvas = canvas.clone();
    draw.connect_activate(move |_| {
        clear_canvas.connect_draw(move |_, cr| {
            cr.set_source_rgb(0.9, 0.9, 0.9);
            cr.paint();
            Inhibit(false)
        });
        let txc = tx.clone();
        thread::spawn(move || {
            //let fractal = Koch::new(txc.clone(), 8);
            //let fractal = Tree::new(txc.clone(), 16);
            let fractal = Sierpinski::new(txc.clone(), 12);
            fractal.execute();
        });
    });
    menu.append(&draw);

    // https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downgrade
    let window_weak = window.downgrade();
    quit.connect_activate(move |_| {
        // https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade
        if let Some(window) = window_weak.upgrade() {
            window.destroy();
        }
        gtk::main_quit();
    });
    menu.append(&quit);

    file.set_submenu(Some(&menu));
    menu_bar.append(&file);

    //--------------------------------------------------------
    // Application Setup
    //--------------------------------------------------------
    vbox.pack_start(&menu_bar, false, false, 0);
    vbox.pack_start(&canvas, true, true, 0);
    vbox.pack_start(&scroll, true, true, 0);

    //--------------------------------------------------------
    // DrawLine
    //--------------------------------------------------------
    rx.attach(None, move |tuple| {
        let (x0, y0, x1, y1) = tuple;
        canvas.connect_draw(move |_, cr| {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(0.5);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            cr.stroke();
            Inhibit(false)
        });
        // canvas.queue_draw_area(x0 as i32, y0 as i32, x1 as i32, y1 as i32);
        canvas.queue_draw();
        glib::Continue(true)
    });
    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();

    gtk::main();
}
fn main() {
    //--------------------------------------------------------
    // Segment Data
    //--------------------------------------------------------
    scheme_gtk();
}
