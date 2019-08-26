/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;

extern crate elisp;

use super::fractal::dragon::Dragon;
use super::fractal::koch::Koch;
use super::fractal::sierpinski::Sierpinski;
use super::fractal::tree::Tree;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use lisp::Environment;
use lisp::Expression;
use lisp::RsError;

use cairo::{Context, Format, ImageSurface, Matrix};
use gtk::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::env;
use std::fs::File;
use std::rc::Rc;

const DRAW_WIDTH: i32 = 720;
const DRAW_HEIGHT: i32 = 560;

const EVAL_RESULT_ID: &str = "result";
const DEFALUT_CANVAS: &str = "canvas";
const PNG_SAVE_FILE: &str = "glisp.png";

const EVAL_KEYCODE: u32 = 101;
const TEXT_CLEAR_KEYCODE: u32 = 107;
const DRAW_CLEAR_KEYCODE: u32 = 108;

const HISTORY_SIZE: usize = 10;
const HISTORY_COL_SIZE: usize = 32;

#[cfg(feature = "animation")]
const MOTION_DELAY: i32 = 70;

pub type ImageTable = Rc<RefCell<HashMap<String, Rc<ImageSurface>>>>;
pub type DrawLine = Box<dyn Fn(f64, f64, f64, f64) + 'static>;

#[derive(Clone)]
struct History {
    menu: gtk::MenuItem,
    children: Rc<RefCell<LinkedList<(gtk::MenuItem, String)>>>,
    max_item: usize,
}
impl History {
    fn new(n: usize) -> Self {
        History {
            menu: gtk::MenuItem::new_with_mnemonic("_History"),
            children: Rc::new(RefCell::new(LinkedList::new())),
            max_item: n,
        }
    }
    fn push(&self, exp: &String, ui: &ControlWidget) {
        let s = String::from(exp).replace("\n", " ");
        let c = if let Some(ref v) = s.get(0..HISTORY_COL_SIZE) {
            gtk::MenuItem::new_with_mnemonic(format!("{} ..", v).as_str())
        } else {
            gtk::MenuItem::new_with_mnemonic(s.as_str())
        };
        let ui = ui.clone();
        let exp_ = exp.clone();
        let exp_ = exp_.into_boxed_str();
        c.connect_activate(move |_| {
            let text_buffer = ui.text_view().get_buffer().expect("Couldn't get window");
            text_buffer.set_text(&exp_);
        });

        if None == self.menu.get_submenu() {
            self.menu.set_submenu(Some(&gtk::Menu::new()));
        }
        if let Some(w) = self.menu.get_submenu() {
            if let Ok(w) = w.downcast::<gtk::Menu>() {
                w.append(&c);

                let mut h = self.children.borrow_mut();
                h.push_front((c, exp.clone()));
                if h.len() > self.max_item {
                    if let Some((c, _)) = h.pop_back() {
                        w.remove(&c);
                    }
                }
                w.show_all();
            }
        }
    }
    fn is_once(&self, exp: &String) -> bool {
        for (_, e) in self.children.borrow().iter() {
            if e == exp {
                return true;
            }
        }
        false
    }
}
#[derive(Clone)]
struct ControlWidget {
    canvas: gtk::DrawingArea,
    text_view: gtk::TextView,
    status_bar: gtk::Statusbar,
}
impl ControlWidget {
    fn new() -> Self {
        ControlWidget {
            canvas: gtk::DrawingArea::new(),
            text_view: gtk::TextView::new(),
            status_bar: gtk::Statusbar::new(),
        }
    }
    fn canvas(&self) -> &gtk::DrawingArea {
        &self.canvas
    }
    fn text_view(&self) -> &gtk::TextView {
        &self.text_view
    }
    fn status_bar(&self) -> &gtk::Statusbar {
        &self.status_bar
    }
}
#[allow(unused_macros)]
macro_rules! get_default_surface {
    ($tbl: expr) => {
        $tbl.borrow()
            .get(&DEFALUT_CANVAS.to_string())
            .unwrap()
            .clone();
    };
}
macro_rules! set_message {
    ($s: expr, $v: expr) => {
        $s.push($s.get_context_id(EVAL_RESULT_ID), $v);
    };
}
pub fn scheme_gtk(env: &Environment, image_table: &ImageTable) {
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
    // History
    //--------------------------------------------------------
    let ui = ControlWidget::new();
    let history = History::new(HISTORY_SIZE);

    //--------------------------------------------------------
    // GtkStatusBar
    //--------------------------------------------------------
    let status_bar = ui.status_bar();
    set_message!(status_bar, "");
    status_bar.set_margin_top(0);
    status_bar.set_margin_bottom(0);

    //--------------------------------------------------------
    // DrawingArea
    //--------------------------------------------------------
    let canvas = ui.canvas();
    canvas.set_size_request(DRAW_WIDTH, DRAW_HEIGHT);

    let surface = get_default_surface!(image_table);
    canvas.connect_draw(move |_, cr| {
        cr.set_source_surface(&*surface, 0.0, 0.0);
        cr.paint();
        Inhibit(false)
    });

    //--------------------------------------------------------
    // TextView
    //--------------------------------------------------------
    let text_view = ui.text_view();
    let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add(text_view);
    scroll.set_size_request(DRAW_WIDTH, 160);
    {
        let env = env.clone();
        let surface = get_default_surface!(image_table);
        let text_buffer = text_view.get_buffer().expect("Couldn't get window");
        let history = history.clone();
        let ui = ui.clone();
        text_view.connect_key_press_event(move |_, key| {
            if key.get_state().intersects(gdk::ModifierType::CONTROL_MASK) {
                match key.get_keyval() {
                    EVAL_KEYCODE => execute_lisp(&env, &ui, &history),
                    DRAW_CLEAR_KEYCODE => clear_canvas(&Context::new(&*surface), ui.canvas()),
                    TEXT_CLEAR_KEYCODE => text_buffer.set_text(""),
                    _ => {}
                }
            }
            Inhibit(false)
        });
    }
    //--------------------------------------------------------
    // GtkMenuBar
    //--------------------------------------------------------
    let menu_bar = gtk::MenuBar::new();
    let menu = gtk::Menu::new();
    let file = gtk::MenuItem::new_with_mnemonic("_File");
    menu.append(&{
        let surface = get_default_surface!(image_table);
        let status_bar = status_bar.downgrade();
        let save = gtk::MenuItem::new_with_mnemonic("_Save");
        save.connect_activate(move |_| {
            let status_bar = status_bar.upgrade().unwrap();
            let mut tmpfile = env::temp_dir();
            tmpfile.push(PNG_SAVE_FILE);
            if tmpfile.exists() {
                set_message!(status_bar, "File is Exists");
                return;
            }
            let mut file = match File::create(tmpfile) {
                Ok(f) => f,
                Err(e) => {
                    set_message!(status_bar, &e.to_string().into_boxed_str());
                    return;
                }
            };
            let msg = match surface.write_to_png(&mut file) {
                Ok(_) => "Saved PNG file".into(),
                Err(e) => e.to_string(),
            };
            set_message!(status_bar, msg.as_str());
        });
        save
    });
    menu.append(&{
        let quit = gtk::MenuItem::new_with_mnemonic("_Quit");
        let env = env.clone();
        quit.connect_activate(move |_| {
            env.set_force_stop(true);
            gtk::main_quit();
        });
        quit
    });

    file.set_submenu(Some(&menu));
    menu_bar.append(&file);

    let edit = gtk::MenuItem::new_with_mnemonic("_Edit");
    let menu = gtk::Menu::new();
    menu.append(&{
        let eval = gtk::MenuItem::new_with_mnemonic("_Eval");
        let env = env.clone();
        let ui = ui.clone();
        let history = history.clone();
        eval.connect_activate(move |_| {
            execute_lisp(&env, &ui, &history);
        });
        eval
    });
    menu.append(&{
        let clear = gtk::MenuItem::new_with_mnemonic("_Clear");
        let surface = get_default_surface!(image_table);

        // https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downgrade
        let canvas = canvas.downgrade();
        clear.connect_activate(move |_| {
            // https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade
            let canvas = canvas.upgrade().unwrap();
            clear_canvas(&Context::new(&*surface), &canvas);
        });
        clear
    });
    edit.set_submenu(Some(&menu));
    menu_bar.append(&edit);

    //--------------------------------------------------------
    // History Command
    //--------------------------------------------------------
    menu_bar.append(&history.menu);

    //--------------------------------------------------------
    // Application Setup
    //--------------------------------------------------------
    vbox.pack_start(&menu_bar, false, false, 0);
    vbox.pack_start(canvas, true, true, 0);
    vbox.pack_start(&scroll, true, true, 0);
    vbox.pack_start(status_bar, true, true, 0);

    //--------------------------------------------------------
    // Create Lisp Function
    //--------------------------------------------------------
    build_lisp_function(env, image_table);
    build_demo_function(env, image_table);
    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();
}
fn execute_lisp(env: &Environment, ui: &ControlWidget, history: &History) {
    let canvas = ui.canvas();
    let text_view = ui.text_view();
    let status_bar = ui.status_bar();
    let text_buffer = text_view.get_buffer().expect("Couldn't get window");

    #[cfg(feature = "animation")]
    let sid = {
        let canvas = canvas.downgrade();
        gtk::timeout_add(MOTION_DELAY as u32, move || {
            let canvas = canvas.upgrade().unwrap();
            canvas.queue_draw();
            gtk::Continue(true)
        })
    };
    let (s, e) = match text_buffer.get_selection_bounds() {
        Some(t) => t,
        None => (text_buffer.get_start_iter(), text_buffer.get_end_iter()),
    };
    let exp = text_buffer
        .get_text(&s, &e, false)
        .expect("die")
        .to_string();

    let result = match lisp::do_core_logic(&exp, env) {
        Ok(r) => {
            if !history.is_once(&exp) {
                history.push(&exp, ui);
            }
            r.to_string()
        }
        Err(e) => {
            if "E9000" == e.get_code() {
                env.set_force_stop(false);
            }
            e.get_msg()
        }
    };
    println!("{}", result);
    set_message!(status_bar, result.as_str());

    #[cfg(feature = "animation")]
    glib::source::source_remove(sid);

    canvas.queue_draw();
}
#[cfg(feature = "animation")]
macro_rules! force_event_loop {
    () => {
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    };
}
fn clear_canvas(cr: &Context, canvas: &gtk::DrawingArea) {
    draw_clear(cr);
    canvas.queue_draw();
}
fn draw_clear(cr: &Context) {
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
fn build_lisp_function(env: &Environment, image_table: &ImageTable) {
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    env.add_builtin_ext_func("draw-clear", move |exp, _| {
        if exp.len() != 1 {
            return Err(create_error!("E1007"));
        }
        draw_clear(&Context::new(&*surface));
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // DrawLine
    // ex. (draw-line 0.0 0.0 1.0 1.0)
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    env.add_builtin_ext_func("draw-line", move |exp, env| {
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

        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(0.001);
        cr.move_to(x0, y0);
        cr.line_to(x1, y1);
        cr.stroke();

        #[cfg(feature = "animation")]
        force_event_loop!();

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Create Image
    // ex. (create-image-from-png "roger" "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png")
    //--------------------------------------------------------
    let image_table_clone = image_table.clone();
    env.add_builtin_ext_func("create-image-from-png", move |exp, env| {
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
        (*image_table_clone)
            .borrow_mut()
            .insert(symbol, Rc::new(surface));
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw Image
    // ex. (draw-image "roger" (list -1.0 0.0 0.0 1.0 180.0 0.0))
    // ex. (draw-image "roger" (list 1.0 0.0 0.0 1.0 0.0 0.0))
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    let image_table_clone = image_table.clone();
    env.add_builtin_ext_func("draw-image", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!("E1007"));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let img = match (*image_table_clone).borrow().get(&symbol) {
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
        let cr = Context::new(&*surface);
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
        cr.set_source_surface(&*img, 0.0, 0.0);
        cr.paint();
        #[cfg(feature = "animation")]
        force_event_loop!();

        Ok(Expression::Nil())
    });
}
fn build_demo_function(env: &Environment, image_table: &ImageTable) {
    macro_rules! make_demo_closure {
        ($drawable: ty, $func: expr, $env: expr, $image_table: expr) => {
            let surface = get_default_surface!($image_table);

            $env.add_builtin_ext_func($func, move |exp, env| {
                if exp.len() != 2 {
                    return Err(create_error!("E1007"));
                }
                let c = match lisp::eval(&exp[1], env)? {
                    Expression::Integer(c) => c,
                    _ => return Err(create_error!("E1002")),
                };
                let cr = Context::new(&*surface);
                cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(0.001);

                let fractal = <$drawable>::new(
                    c,
                    Box::new(move |x0, y0, x1, y1| {
                        cr.move_to(x0, y0);
                        cr.line_to(x1, y1);
                        cr.stroke();
                        #[cfg(feature = "animation")]
                        force_event_loop!();
                    }),
                );
                fractal.do_demo();
                Ok(Expression::Nil())
            });
        };
    }
    make_demo_closure!(Koch, "draw-koch", env, image_table);
    make_demo_closure!(Tree, "draw-tree", env, image_table);
    make_demo_closure!(Sierpinski, "draw-sierpinski", env, image_table);
    make_demo_closure!(Dragon, "draw-dragon", env, image_table);
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
