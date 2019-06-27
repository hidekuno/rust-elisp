extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;

extern crate elisp;

use cairo::ImageSurface;
use cairo::Matrix;
use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use gtk::prelude::*;
use lisp::Environment;
use lisp::Expression;
use lisp::RsError;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

const DRAW_WIDTH: i32 = 720;
const DRAW_HEIGHT: i32 = 560;
const EVAL_KEYCODE: u32 = 101;
const EVAL_RESULT_ID: &str = "result";

#[cfg(feature = "animation")]
const MOTION_DELAY: i32 = 700;

type ImageTable = Rc<RefCell<HashMap<String, ImageSurface>>>;

fn scheme_gtk(env: &mut Environment, image_table: &ImageTable) {
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
    // GtkStatusBar
    //--------------------------------------------------------
    let status_bar = gtk::Statusbar::new();
    status_bar.push(status_bar.get_context_id(EVAL_RESULT_ID), "");
    status_bar.set_margin_top(0);
    status_bar.set_margin_bottom(0);

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
    scroll.set_size_request(DRAW_WIDTH, 160);

    let env_ = RefCell::new(env.clone());
    let canvas_ = canvas.clone();
    let clone_bar = status_bar.clone();
    text_view.connect_key_press_event(move |w, key| {
        if key.get_state().intersects(gdk::ModifierType::CONTROL_MASK)
            && key.get_keyval() == EVAL_KEYCODE
        {
            execute_lisp(&mut env_.borrow_mut(), &canvas_, w, &clone_bar);
        }
        Inhibit(false)
    });
    //--------------------------------------------------------
    // GtkMenuBar
    //--------------------------------------------------------
    let menu_bar = gtk::MenuBar::new();
    let menu = gtk::Menu::new();
    let file = gtk::MenuItem::new_with_mnemonic("_File");
    let eval = gtk::MenuItem::new_with_mnemonic("_Eval");

    let env_ = RefCell::new(env.clone());
    let canvas_weak = canvas.downgrade();
    let status_bar_weak = status_bar.downgrade();
    eval.connect_activate(move |_| {
        execute_lisp(
            &mut env_.borrow_mut(),
            &canvas_weak.upgrade().unwrap(),
            &text_view,
            &status_bar_weak.upgrade().unwrap(),
        );
    });
    menu.append(&eval);

    let quit = gtk::MenuItem::new_with_mnemonic("_Quit");
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
    vbox.pack_start(&status_bar, true, true, 0);

    //--------------------------------------------------------
    // Create Lisp Function
    //--------------------------------------------------------
    build_lisp_function(env, &canvas, image_table);

    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();
}
fn execute_lisp(
    env: &mut Environment,
    canvas: &gtk::DrawingArea,
    text_view: &gtk::TextView,
    status_bar: &gtk::Statusbar,
) {
    let text_buffer = text_view.get_buffer().expect("Couldn't get window");

    #[cfg(feature = "animation")]
    let canvas_weak = canvas.downgrade();
    #[cfg(feature = "animation")]
    let sid = gtk::timeout_add(MOTION_DELAY as u32, move || {
        let canvas = canvas_weak.upgrade().unwrap();
        canvas.queue_draw();
        gtk::Continue(true)
    });

    let s = text_buffer.get_start_iter();
    let e = text_buffer.get_end_iter();
    let exp = text_buffer.get_text(&s, &e, false).expect("die");

    let result = match lisp::do_core_logic(&exp.to_string(), env) {
        Ok(r) => r.value_string(),
        Err(e) => e.get_code(),
    };

    println!("{}", result);
    status_bar.push(status_bar.get_context_id(EVAL_RESULT_ID), result.as_str());

    #[cfg(feature = "animation")]
    glib::source::source_remove(sid);

    canvas.queue_draw();
}

#[allow(unused_macros)]
macro_rules! force_event_loop {
    ($e: expr) => {
        while gtk::events_pending() {
            gtk::main_iteration_do($e);
        }
    };
}
fn build_lisp_function(env: &mut Environment, canvas: &gtk::DrawingArea, image_table: &ImageTable) {
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    let canvas_weak = canvas.downgrade();
    env.add_builtin_closure("draw-clear", move |exp, _| {
        if exp.len() != 1 {
            return Err(create_error!("E1007"));
        }
        let canvas = canvas_weak.upgrade().unwrap();
        canvas.connect_draw(move |_, cr| {
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
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // DrawLine
    // ex. (draw-line 0.0 0.0 1.0 1.0)
    //--------------------------------------------------------
    let canvas_weak = canvas.downgrade();
    env.add_builtin_closure("draw-line", move |exp, env| {
        const N: usize = 4;
        if exp.len() != (N + 1) {
            return Err(create_error!("E1007"));
        }
        let mut loc: [f64; N] = [0.0; N];
        let mut iter = exp[1 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = lisp::eval(e, env)? {
                    loc[i] = f;
                } else {
                    return Err(create_error!("E1003"));
                }
            }
        }
        let (x0, y0, x1, y1) = (loc[0], loc[1], loc[2], loc[3]);
        let canvas = canvas_weak.upgrade().unwrap();
        canvas.connect_draw(move |_, cr| {
            cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(0.001);
            cr.move_to(x0, y0);
            cr.line_to(x1, y1);
            cr.stroke();

            Inhibit(false)
        });
        #[cfg(feature = "animation")]
        force_event_loop!(true);

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Create Image
    // ex. (create-image-from-png "roger" "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png")
    //--------------------------------------------------------
    let image_table_clone = image_table.clone();
    env.add_builtin_closure("create-image-from-png", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!("E1007"));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let filename = match lisp::eval(&exp[2], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(create_error_value!("E9999", e)),
        };
        let surface = match ImageSurface::create_from_png(&mut file) {
            Ok(s) => s,
            Err(e) => return Err(create_error_value!("E9999", e)),
        };
        (*image_table_clone).borrow_mut().insert(symbol, surface);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw Image
    // ex. (draw-image "roger" (list -1.0 0.0 0.0 1.0 180.0 0.0))
    // ex. (draw-image "roger" (list 1.0 0.0 0.0 1.0 0.0 0.0))
    //--------------------------------------------------------
    let canvas_weak = canvas.downgrade();
    let image_table_clone = image_table.clone();
    env.add_builtin_closure("draw-image", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!("E1007"));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let surface = match (*image_table_clone).borrow().get(&symbol) {
            Some(v) => v.clone(),
            None => return Err(create_error!("E1008")),
        };

        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        if let Expression::List(l) = lisp::eval(&exp[2], env)? {
            if l.len() != 6 {
                return Err(create_error!("E1007"));
            }
            let mut iter = l.iter();
            for i in 0..N {
                if let Some(e) = iter.next() {
                    if let Expression::Float(f) = lisp::eval(e, env)? {
                        ctm[i] = f;
                    } else {
                        return Err(create_error!("E1003"));
                    }
                }
            }
        } else {
            return Err(create_error!("E1005"));
        }
        let canvas = canvas_weak.upgrade().unwrap();
        canvas.connect_draw(move |_, cr| {
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

        #[cfg(feature = "animation")]
        force_event_loop!(true);

        Ok(Expression::Nil())
    });
}
fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let mut env = Environment::new();
    let image_table = Rc::new(RefCell::new(HashMap::new()));
    scheme_gtk(&mut env, &image_table);

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
fn do_lisp_env(program: &str, env: &mut Environment) -> String {
    match lisp::do_core_logic(&String::from(program), env) {
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
    let mut env = Environment::new();
    let image_table = Rc::new(RefCell::new(HashMap::new()));
    scheme_gtk(&mut env, &image_table);

    // draw-clear check
    assert_str!(do_lisp_env("(draw-clear 10)", &mut env), "E1007");

    // draw-line check
    assert_str!(do_lisp_env("(draw-line)", &mut env), "E1007");
    assert_str!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &mut env), "E1003");
    assert_str!(do_lisp_env("(draw-line a b 2.0 3)", &mut env), "E1008");

    // create-image-from-png check
    assert_str!(do_lisp_env("(create-image-from-png)", &mut env), "E1007");
    assert_str!(
        do_lisp_env("(create-image-from-png \"sample\")", &mut env),
        "E1007"
    );
    assert_str!(
        do_lisp_env("(create-image-from-png 10 \"/tmp/hoge.png\")", &mut env),
        "E1015"
    );
    assert_str!(
        do_lisp_env("(create-image-from-png \"sample\" 20)", &mut env),
        "E1015"
    );
    assert_str!(
        do_lisp_env(
            format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
            &mut env
        ),
        "E9999"
    );
    let mut file = File::create(&png).unwrap();
    assert_str!(
        do_lisp_env(
            format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
            &mut env
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
    file.write_all(&png_data).unwrap();
    file.flush().unwrap();
    do_lisp_env(
        format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
        &mut env,
    );

    assert_str!(do_lisp_env("(draw-image)", &mut env), "E1007");
    assert_str!(do_lisp_env("(draw-image 10)", &mut env), "E1007");
    assert_str!(
        do_lisp_env("(draw-image \"sample\" (list 1 2 3) 10)", &mut env),
        "E1007"
    );
    assert_str!(
        do_lisp_env("(draw-image 10 (list 0.0 0.0 1.0 1.0))", &mut env),
        "E1015"
    );
    assert_str!(
        do_lisp_env("(draw-image \"sample1\" (list 0.0 0.0 1.0 1.0))", &mut env),
        "E1008"
    );
    assert_str!(
        do_lisp_env("(draw-image \"sample\" (list 0.0 0.0 1.0 1.0))", &mut env),
        "E1007"
    );
    assert_str!(
        do_lisp_env(
            "(draw-image \"sample\" (list 0.0 0.0 1.0 1.0 1.0 10))",
            &mut env
        ),
        "E1003"
    );
    assert_str!(do_lisp_env("(draw-image \"sample\" 10)", &mut env), "E1005");

    std::fs::remove_file(png).unwrap();
}
