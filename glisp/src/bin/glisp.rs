extern crate cairo;
extern crate gtk;

extern crate elisp;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use lisp::EvalResult;
use lisp::Expression;
use lisp::RsError;
use lisp::SimpleEnv;

use cairo::ImageSurface;
use cairo::Matrix;
use gtk::prelude::*;

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

const DRAW_WIDTH: i32 = 720;
const DRAW_HEIGHT: i32 = 560;

fn scheme_gtk(rc: &Rc<RefCell<SimpleEnv>>) {
    gtk::init().expect("Failed to initialize GTK.");
    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("Rust eLisp");
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
    canvas.set_size_request(DRAW_WIDTH, DRAW_HEIGHT);
    canvas.connect_draw(|_, cr| {
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
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
    let eval = gtk::MenuItem::new_with_label("Eval");

    let clear_canvas = canvas.clone();
    let text_buffer = text_view.get_buffer().expect("Couldn't get window");
    let env = rc.clone();
    eval.connect_activate(move |_| {
        let s = text_buffer.get_start_iter();
        let e = text_buffer.get_end_iter();
        let exp = text_buffer.get_text(&s, &e, false).expect("die");

        let result = match lisp::do_core_logic(exp.to_string(), &mut (*env).borrow_mut()) {
            Ok(r) => r.value_string(),
            Err(e) => e.get_code(),
        };
        println!("{}", result);
        clear_canvas.queue_draw();
    });
    menu.append(&eval);

    let quit = gtk::MenuItem::new_with_label("Quit");
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
    // Create Lisp Function
    //--------------------------------------------------------
    build_lisp_function(rc, &canvas);

    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();
}
fn build_lisp_function(rc: &Rc<RefCell<SimpleEnv>>, canvas: &gtk::DrawingArea) {
    let mut e = (*rc).borrow_mut();
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    {
        let canvas_ = canvas.clone();
        e.add_builtin_closure("draw-clear", move |exp, _| {
            if exp.len() != 1 {
                return Err(create_error!("E1007"));
            }
            canvas_.connect_draw(move |_, cr| {
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

                Inhibit(false)
            });
            Ok(Expression::Symbol(String::from("draw-clear")))
        });
    }
    //--------------------------------------------------------
    // DrawLine
    //--------------------------------------------------------
    {
        let canvas_ = canvas.clone();
        e.add_builtin_closure("draw-line", move |exp, env| {
            if exp.len() != 5 {
                return Err(create_error!("E1007"));
            }

            let mut vec: Vec<f64> = Vec::new();
            for e in &exp[1 as usize..] {
                if let Expression::Float(f) = lisp::eval(e, env)? {
                    vec.push(f);
                } else {
                    return Err(create_error!("E1003"));
                }
            }
            let (x0, y0, x1, y1) = (vec[0], vec[1], vec[2], vec[3]);
            canvas_.connect_draw(move |_, cr| {
                cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(0.001);
                cr.move_to(x0, y0);
                cr.line_to(x1, y1);
                cr.stroke();

                Inhibit(false)
            });
            canvas_.queue_draw();
            while gtk::events_pending() {
                gtk::main_iteration_do(true);
            }
            Ok(Expression::Symbol(String::from("draw-line")))
        });
    }
    //--------------------------------------------------------
    // Draw Clear
    // (draw-image "/home/kunohi/rust-elisp/glisp/examples/sicp.png" (list -1.0 0.0 0.0 1.0 180.0 0.0))
    // (draw-image "/home/kunohi/rust-elisp/glisp/examples/sicp.png" (list 1.0 0.0 0.0 1.0 0.0 0.0))
    //--------------------------------------------------------
    {
        let canvas_ = canvas.clone();
        e.add_builtin_closure("draw-image", move |exp, env| {
            if exp.len() != 3 {
                return Err(create_error!("E1007"));
            }
            let filename = match lisp::eval(&exp[1], env)? {
                Expression::String(s) => s,
                _ => return Err(create_error!("E1015")),
            };
            let mut file = match File::open(filename) {
                Ok(f) => f,
                Err(e) => return Err(create_error_value!("E9999", e.to_string())),
            };
            let surface = match ImageSurface::create_from_png(&mut file) {
                Ok(s) => s,
                Err(e) => return Err(create_error_value!("E9999", e.to_string())),
            };
            let mut ctm: Vec<f64> = Vec::new();
            if let Expression::List(l) = lisp::eval(&exp[2], env)? {
                if l.len() != 6 {
                    return Err(create_error!("E1007"));
                }
                for e in &l {
                    if let Expression::Float(f) = lisp::eval(e, env)? {
                        ctm.push(f);
                    } else {
                        return Err(create_error!("E1003"));
                    }
                }
            } else {
                return Err(create_error!("E1005"));
            }
            canvas_.connect_draw(move |_, cr| {
                cr.scale(1.0, 1.0);
                cr.move_to(0.0, 0.0);
                let matrix = Matrix {
                    xx: ctm[0],
                    yx: ctm[1],
                    xy: ctm[2],
                    yy: ctm[3],
                    x0: ctm[4],
                    y0: ctm[5],
                };
                cr.transform(matrix);
                cr.set_source_surface(&surface, 0.0, 0.0);
                cr.paint();

                Inhibit(false)
            });
            canvas_.queue_draw();
            while gtk::events_pending() {
                gtk::main_iteration_do(true);
            }
            Ok(Expression::Symbol(String::from("draw-image")))
        });
    }
}
fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let rc = Rc::new(RefCell::new(SimpleEnv::new()));

    scheme_gtk(&rc);

    gtk::main();
}

#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
macro_rules! assert_str {
    ($a: expr,
     $b: expr) => {
        assert!($a == $b.to_string())
    };
}
#[cfg(test)]
fn do_lisp_env(program: &str, rc: &Rc<RefCell<SimpleEnv>>) -> String {
    match lisp::do_core_logic(program.to_string(), &mut (*rc).borrow_mut()) {
        Ok(v) => {
            return v.value_string();
        }
        Err(e) => {
            return String::from(e.get_code());
        }
    }
}

#[test]
fn test_error_check() {
    let png = format!(
        "/tmp/hoge_{}.png",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let rc = Rc::new(RefCell::new(SimpleEnv::new()));
    scheme_gtk(&rc);
    assert_str!(do_lisp_env("(draw-clear 10)", &rc), "E1007");
    assert_str!(do_lisp_env("(draw-line)", &rc), "E1007");
    assert_str!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &rc), "E1003");
    assert_str!(do_lisp_env("(draw-image)", &rc), "E1007");
    assert_str!(
        do_lisp_env(
            format!("(draw-image \"{}\" (list 1 2 3) 10)", png).as_str(),
            &rc
        ),
        "E1007"
    );
    assert_str!(
        do_lisp_env(
            format!("(draw-image \"{}\" (list 1 2 3))", png).as_str(),
            &rc
        ),
        "E9999"
    );

    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xd0,
        0xd2, 0xd2, 0x02, 0x00, 0x01, 0x00, 0x00, 0x7f, 0x09, 0xa9, 0x5a, 0x4d, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let mut file = File::create(&png).unwrap();
    file.write_all(&png_data).unwrap();
    file.flush().unwrap();

    assert_str!(
        do_lisp_env(
            format!(
                "(draw-image \"{}\" (list 0.0 0.1 0.2 0.3 0.4 0.5 0.6))",
                png
            )
            .as_str(),
            &rc
        ),
        "E1007"
    );
    assert_str!(
        do_lisp_env(format!("(draw-image \"{}\" 10)", png).as_str(), &rc),
        "E1005"
    );
    assert_str!(
        do_lisp_env(
            format!("(draw-image \"{}\" (list 0.0 0.1 0.2 0.3 0.4 5))", png).as_str(),
            &rc
        ),
        "E1003"
    );
    std::fs::remove_file(png).unwrap();
}
