/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate gtk;

use elisp::lisp;
use gtk::gdk;
#[cfg(feature = "animation")]
use gtk::glib;
use lisp::Environment;

use gtk::prelude::*;
use std::env;
use std::rc::Rc;

#[cfg(feature = "animation")]
use std::time::Duration;

use crate::draw::draw_clear;
use crate::draw::save_png_file;
use crate::draw::DrawTable;
use crate::draw::Graffiti;
use crate::helper::load_demo_program;
use crate::helper::search_word_highlight;
use crate::helper::History;
use crate::helper::SourceView;
use crate::helper::HISTORY_SIZE;

pub const DRAW_WIDTH: i32 = 560;
pub const DRAW_HEIGHT: i32 = 560;
const EVAL_RESULT_ID: &str = "result";
const PNG_SAVE_FILE: &str = "glisp.png";

const DRAW_STOP_KEYCODE: u32 = 99;
const EVAL_KEYCODE: u32 = 114;
const TEXT_CLEAR_KEYCODE: u32 = 117;
const DRAW_CLEAR_KEYCODE: u32 = 108;

#[cfg(feature = "animation")]
const MOTION_DELAY: u64 = 70;
const STOP_ERROR_CODE: &str = "E9000";

//--------------------------------------------------------
// Control widget table
//--------------------------------------------------------
#[derive(Clone)]
struct ControlWidget {
    canvas: gtk::DrawingArea,
    text_view: gtk::TextView,
    status_bar: gtk::Statusbar,
    source_view: SourceView,
}
impl ControlWidget {
    fn new() -> Self {
        let text_view = gtk::TextView::new();
        let text_buffer = text_view.buffer().expect("Couldn't get window");
        let source_view = SourceView::new(&text_buffer);

        ControlWidget {
            canvas: gtk::DrawingArea::new(),
            text_view,
            status_bar: gtk::Statusbar::new(),
            source_view,
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
    fn source_view(&self) -> &SourceView {
        &self.source_view
    }
}
macro_rules! set_message {
    ($s: expr, $v: expr) => {
        $s.push($s.context_id(EVAL_RESULT_ID), $v);
    };
}
//--------------------------------------------------------
// Drawing Area clear
//--------------------------------------------------------
fn clear_canvas(draw_table: &DrawTable, canvas: &gtk::DrawingArea) {
    draw_clear(draw_table);
    canvas.queue_draw();
}
//--------------------------------------------------------
// Setting keystroke like Emacs
//--------------------------------------------------------
fn setup_key_emacs_like() {
    // https://gist.github.com/shelling/663759
    let style = "
@binding-set my-text-view-bindings
{
    bind \"<ctrl>b\" { \"move-cursor\" (logical-positions, -1, 0) };
    bind \"<ctrl>f\" { \"move-cursor\" (logical-positions, 1, 0) };
    bind \"<alt>b\"  { \"move-cursor\" (words, -1, 0) };
    bind \"<alt>f\"  { \"move-cursor\" (words, 1, 0) };

    bind \"<ctrl>a\" { \"move-cursor\" (paragraph-ends, -1, 0)};
    bind \"<ctrl>e\" { \"move-cursor\" (paragraph-ends,  1, 0) };
    bind \"<ctrl>p\" { \"move-cursor\" (display-lines, -1, 0) };
    bind \"<ctrl>n\" { \"move-cursor\" (display-lines, 1, 0) };

    bind \"<ctrl>d\" { \"delete-from-cursor\" (chars, 1) };
    bind \"<alt>d\"  { \"delete-from-cursor\" (word-ends, 1) };
    bind \"<ctrl>k\" { \"delete-from-cursor\" (paragraph-ends, 1) };
    bind \"<ctrl>h\" { \"delete-from-cursor\" (chars, -1) };

    bind \"<ctrl>m\" { \"insert-at-cursor\" (\"\\n\")};
    bind \"<ctrl>i\" { \"insert-at-cursor\" (\"  \")};

    bind \"<ctrl>space\" { \"set-anchor\" () };
    bind \"<alt>w\" { \"copy-clipboard\" () };
    bind \"<ctrl>y\" { \"paste-clipboard\" () };
    bind \"<ctrl>w\" { \"cut-clipboard\" () };
    bind \"<ctrl>8\" { \"insert_at_cursor\" (\"()\") \"move-cursor\" (logical-positions, -1, 0)};
}
textview {
  -gtk-key-bindings: my-text-view-bindings;
}";

    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(style.as_bytes())
        .expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
//--------------------------------------------------------
// SICP Program
//--------------------------------------------------------
fn create_demo_program_menu(menu: &str, pdir: &'static str, ui: &ControlWidget) -> gtk::MenuItem {
    let load = gtk::MenuItem::with_mnemonic(menu);
    let ui = ui.clone();
    load.connect_activate(move |_| match load_demo_program(pdir) {
        Ok(v) => {
            let text_buffer = ui.text_view().buffer().expect("Couldn't get window");
            text_buffer.set_text(&v.into_boxed_str());
            ui.source_view().do_highlight(&text_buffer);
        }
        Err(e) => {
            let status_bar = ui.status_bar();
            set_message!(status_bar, &e.to_string().into_boxed_str());
        }
    });
    load
}
//--------------------------------------------------------
// Create about dialog
//--------------------------------------------------------
fn create_environment_menu(
    env: &Environment,
    window: &gtk::Window,
    menu: &'static str,
    f: fn(&Environment) -> String,
) -> gtk::MenuItem {
    let mi = gtk::MenuItem::with_mnemonic(menu);
    let env = env.clone();

    let dialog = gtk::Dialog::new();
    dialog.set_title(menu);
    dialog.set_transient_for(Some(window));
    dialog.add_button("Ok", gtk::ResponseType::Ok);

    let label = gtk::Label::new(None);
    label.set_selectable(true);
    let content_area = dialog.content_area();
    content_area.add(&label);

    mi.connect_activate(move |_| {
        label.set_text(f(&env).as_str());
        dialog.show_all();
        dialog.run();
        dialog.hide();
    });
    mi
}
//--------------------------------------------------------
// Create File dialog
//--------------------------------------------------------
fn create_save_as_menu(
    window: &gtk::Window,
    status_bar: &gtk::Statusbar,
    draw_table: &DrawTable,
) -> gtk::MenuItem {
    let mi = gtk::MenuItem::with_mnemonic("_Save As");

    // Gtk-WARNING **: Failed to measure available space
    // I'm researching the cause
    let dialog =
        gtk::FileChooserDialog::new(Some("PNG Save"), Some(window), gtk::FileChooserAction::Save);
    dialog.add_buttons(&[
        ("Save", gtk::ResponseType::Accept),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    let status_bar = status_bar.clone();
    let draw_table = draw_table.clone();
    mi.connect_activate(move |_| {
        if gtk::ResponseType::Accept == dialog.run() {
            let message = save_png_file(&draw_table, &dialog.filename().unwrap(), true);
            set_message!(status_bar, message.as_str());
        }
        dialog.hide();
    });
    mi
}
//--------------------------------------------------------
// Create about dialog
//--------------------------------------------------------
fn create_search_menu(window: &gtk::Window, text_buffer: gtk::TextBuffer) -> gtk::MenuItem {
    let mi = gtk::MenuItem::with_mnemonic("Search");

    let dialog = gtk::Dialog::new();
    dialog.set_title("Search");
    dialog.set_transient_for(Some(window));
    dialog.add_button("Ok", gtk::ResponseType::Ok);

    let entry = gtk::SearchEntry::new();
    let content_area = dialog.content_area();
    content_area.add(&entry);

    mi.connect_activate(move |_| {
        dialog.show_all();
        if gtk::ResponseType::Ok == dialog.run() {
            let text = entry.text();
            search_word_highlight(&text_buffer, "search", text.as_str());
        }
        dialog.hide();
    });
    mi
}
//--------------------------------------------------------
// Gtk widget initialize
//--------------------------------------------------------
pub fn scheme_gtk(env: &Environment, draw_table: &DrawTable) {
    gtk::init().expect("Failed to initialize GTK.");
    setup_key_emacs_like();

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

    {
        let draw_table = draw_table.clone();
        canvas.connect_draw(move |_, cr| {
            draw_table.set_cairo_surface(cr);
            Inhibit(false)
        });
    }

    let gr = Rc::new(Graffiti::new(draw_table));
    let c = gr.clone();
    canvas.connect_button_press_event(move |_, e| {
        if e.state() == gdk::ModifierType::BUTTON1_MASK {
            let (x, y) = e.position();
            c.start_graffiti(x, y);
        }
        Inhibit(true)
    });
    let c = gr.clone();
    canvas.connect_motion_notify_event(move |w, e| {
        if e.state() == gdk::ModifierType::BUTTON1_MASK {
            let (x, y) = e.position();
            c.draw_graffiti(x, y);
            w.queue_draw_area(x as i32 - 2, y as i32 - 2, 4, 4);
        }
        Inhibit(true)
    });
    let c = gr;
    canvas.connect_button_release_event(move |w, e| {
        if e.state() == gdk::ModifierType::BUTTON1_MASK {
            let (x, y) = e.position();
            c.stop_graffiti(x, y);
            w.queue_draw();
        }
        Inhibit(true)
    });
    canvas.set_events(
        canvas.events()
            | gdk::EventMask::BUTTON_PRESS_MASK
            | gdk::EventMask::BUTTON_RELEASE_MASK
            | gdk::EventMask::POINTER_MOTION_MASK,
    );
    //--------------------------------------------------------
    // TextView
    //--------------------------------------------------------
    let text_view = ui.text_view();
    let source_view = ui.source_view().clone();
    let scroll = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add(text_view);
    scroll.set_size_request(DRAW_WIDTH, 160);
    {
        let env = env.clone();
        let text_buffer = text_view.buffer().expect("Couldn't get window");
        let history = history.clone();
        let ui = ui.clone();
        let draw_table = draw_table.clone();
        text_view.connect_key_press_event(move |_, key| {
            if key.state().intersects(gdk::ModifierType::CONTROL_MASK) {
                if let Some(c) = key.keyval().to_unicode() {
                    match c as u32 {
                        DRAW_STOP_KEYCODE => env.set_force_stop(true),
                        EVAL_KEYCODE => execute_lisp(&env, &ui, &history),
                        DRAW_CLEAR_KEYCODE => clear_canvas(&draw_table, ui.canvas()),
                        TEXT_CLEAR_KEYCODE => text_buffer.set_text(""),
                        _ => {}
                    }
                }
            }
            Inhibit(false)
        });
    }
    {
        let text_buffer = text_view.buffer().expect("Couldn't get window");
        text_buffer.connect_end_user_action(move |w| {
            source_view.do_highlight(w);
        });
    }
    //--------------------------------------------------------
    // GtkMenuBar
    //--------------------------------------------------------
    let menu_bar = gtk::MenuBar::new();
    let menu = gtk::Menu::new();
    let file = gtk::MenuItem::with_mnemonic("_File");
    menu.append(&{
        let status_bar = status_bar.downgrade();
        let save = gtk::MenuItem::with_mnemonic("_Save");

        let draw_table = draw_table.clone();
        save.connect_activate(move |_| {
            let status_bar = status_bar.upgrade().unwrap();
            let mut tmpfile = env::temp_dir();
            tmpfile.push(PNG_SAVE_FILE);
            let message = save_png_file(&draw_table, &tmpfile, false);
            set_message!(status_bar, message.as_str());
        });
        save
    });
    menu.append(&create_save_as_menu(&window, status_bar, draw_table));
    menu.append(&{
        let quit = gtk::MenuItem::with_mnemonic("_Quit");
        let env = env.clone();
        quit.connect_activate(move |_| {
            env.set_force_stop(true);
            gtk::main_quit();
        });
        quit
    });

    file.set_submenu(Some(&menu));
    menu_bar.append(&file);

    let edit = gtk::MenuItem::with_mnemonic("_Edit");
    let menu = gtk::Menu::new();
    menu.append(&{
        let eval = gtk::MenuItem::with_mnemonic("_Eval");
        let env = env.clone();
        let ui = ui.clone();
        let history = history.clone();
        eval.connect_activate(move |_| {
            execute_lisp(&env, &ui, &history);
        });
        eval
    });
    menu.append(&{
        let clear = gtk::MenuItem::with_mnemonic("_Draw Clear");
        let draw_table = draw_table.clone();
        // https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downgrade
        let canvas = canvas.downgrade();
        clear.connect_activate(move |_| {
            // https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade
            let canvas = canvas.upgrade().unwrap();
            clear_canvas(&draw_table, &canvas);
            canvas.queue_draw();
        });
        clear
    });
    menu.append(&{
        let clear = gtk::MenuItem::with_mnemonic("_Code Clear");
        let text_buffer = text_view.buffer().expect("Couldn't get window");

        clear.connect_activate(move |_| {
            text_buffer.set_text("");
        });
        clear
    });
    edit.set_submenu(Some(&menu));
    menu_bar.append(&edit);

    let search = gtk::MenuItem::with_mnemonic("_Search");
    let menu = gtk::Menu::new();
    menu.append(&create_search_menu(
        &window,
        text_view.buffer().expect("Couldn't get window"),
    ));
    search.set_submenu(Some(&menu));
    menu_bar.append(&search);

    let load = gtk::MenuItem::with_mnemonic("_Load");
    let menu = gtk::Menu::new();
    menu.append(&create_demo_program_menu("_SICP", "sicp.scm", &ui));
    menu.append(&create_demo_program_menu("_Fractal", "fractal", &ui));
    load.set_submenu(Some(&menu));
    menu_bar.append(&load);

    //--------------------------------------------------------
    // History Command
    //--------------------------------------------------------
    menu_bar.append(history.menu());

    //--------------------------------------------------------
    // Function List
    //--------------------------------------------------------
    let help = gtk::MenuItem::with_mnemonic("Help");
    let menu = gtk::Menu::new();
    menu.append(&create_environment_menu(
        env,
        &window,
        "Functoins",
        |env| match env.get_function_list() {
            Some(e) => e,
            None => "Not exists".into(),
        },
    ));
    menu.append(&create_environment_menu(
        env,
        &window,
        "Variables",
        |env| match env.get_variable_list() {
            Some(e) => e,
            None => "Not exists".into(),
        },
    ));
    menu.append(&create_environment_menu(env, &window, "Builtins", |env| {
        env.get_builtin_func_list()
    }));
    menu.append(&create_environment_menu(
        env,
        &window,
        "Draw Builtins",
        |env| env.get_builtin_ext_list(),
    ));
    help.set_submenu(Some(&menu));
    menu_bar.append(&help);

    //--------------------------------------------------------
    // Application Setup
    //--------------------------------------------------------
    vbox.pack_start(&menu_bar, false, false, 0);
    vbox.pack_start(canvas, true, true, 0);
    vbox.pack_start(&scroll, true, true, 0);
    vbox.pack_start(status_bar, false, false, 0);

    //--------------------------------------------------------
    // Build Up finish
    //--------------------------------------------------------
    window.add(&vbox);
    window.show_all();
}
//--------------------------------------------------------
// Lisp code tokenize, parse, eval
//--------------------------------------------------------
fn execute_lisp(env: &Environment, ui: &ControlWidget, history: &History) {
    let canvas = ui.canvas();
    let source_view = ui.source_view();
    let text_view = ui.text_view();
    let status_bar = ui.status_bar();
    let text_buffer = text_view.buffer().expect("Couldn't get window");

    #[cfg(feature = "animation")]
    let sid = {
        let canvas = canvas.downgrade();
        glib::timeout_add_local(Duration::from_millis(MOTION_DELAY), move || {
            let canvas = canvas.upgrade().unwrap();
            canvas.queue_draw();
            glib::Continue(true)
        })
    };
    let (s, e) = match text_buffer.selection_bounds() {
        Some(t) => t,
        None => (text_buffer.start_iter(), text_buffer.end_iter()),
    };
    let exp = text_buffer.text(&s, &e, false).expect("die").to_string();

    let result = match lisp::do_core_logic(&exp, env) {
        Ok(r) => {
            if !history.is_once(&exp) {
                history.push(&exp, &text_buffer, source_view);
            }
            r.to_string()
        }
        Err(e) => {
            if STOP_ERROR_CODE == e.get_code() {
                env.set_force_stop(false);
            }
            e.get_msg()
        }
    };
    println!("{}", result);
    set_message!(status_bar, result.as_str());

    #[cfg(feature = "animation")]
    sid.remove();

    canvas.queue_draw();
}
