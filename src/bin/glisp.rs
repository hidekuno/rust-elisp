#[macro_use]
extern crate elisp;
extern crate gtk;

use elisp::create_error;
use elisp::lisp;
use lisp::EvalResult;
use lisp::Expression;
use lisp::RsError;
use lisp::SimpleEnv;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

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
    canvas.set_size_request(720, 560);
    canvas.connect_draw(|_, cr| {
        cr.scale(720 as f64, 560 as f64);
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
    // DrawLine
    //--------------------------------------------------------
    {
        let r = rc.clone();
        let mut e = (*r).borrow_mut();
        let clear_canvas = canvas.clone();
        e.add_builtin_closure("draw-clear", move |exp, env| {
            if exp.len() != 1 {
                return Err(create_error!("E1007"));
            }
            clear_canvas.connect_draw(move |_, cr| {
                cr.set_source_rgb(0.9, 0.9, 0.9);
                cr.paint();

                Inhibit(false)
            });
            Ok(Expression::Symbol(String::from("draw-clear")))
        });
    }
    {
        let r = rc.clone();
        let mut e = (*r).borrow_mut();
        let clear_canvas = canvas.clone();

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
            clear_canvas.connect_draw(move |_, cr| {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(0.5);
                cr.move_to(x0, y0);
                cr.line_to(x1, y1);
                cr.stroke();

                Inhibit(false)
            });
            Ok(Expression::Symbol(String::from("draw-line")))
        });
    }
    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();

    gtk::main();
}
fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let rc = Rc::new(RefCell::new(SimpleEnv::new()));

    scheme_gtk(&rc);
}
