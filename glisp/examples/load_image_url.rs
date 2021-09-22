/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ex.)
    (load-image-url "rb" "https://images-fe.ssl-images-amazon.com/images/I/51TCtM6EVWL._AC_UL160_.jpg")
    (draw-clear)
    (draw-image "rb" 0.0 0.0
            (/ (image-width "rb") (screen-width) 1.0) 0.0
            0.0 (/ (image-height "rb")(screen-height) 1.0))

    (define ll (paint-image "ll"))
    (define aframe (make-image-frame "ll" 2.0))
    ((square-limit ll 0) aframe)

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate glisp;
extern crate surf;

use std::cell::RefCell;
use std::rc::Rc;
use gtk::glib::Bytes;
use gtk::gio::MemoryInputStream;
use gtk::gio::Cancellable;
use gtk::gdk_pixbuf::Pixbuf;
use surf::http::StatusCode;
use async_std::task;

use crate::elisp::create_error;
use elisp::lisp;
use glisp::buildin;
use glisp::draw;
use glisp::ui;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Expression;
use lisp::Error;
use lisp::eval;
use buildin::build_demo_function;
use buildin::build_lisp_function;
use draw::create_draw_table;
use draw::DrawTable;
use draw::PixbufWrapper;
use ui::scheme_gtk;

#[cfg(not(feature = "redirect"))]
fn load_url(url: &String) -> Result<(Vec<u8>,StatusCode),
                                    Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(
        async {
            let mut res = surf::get(url).await?;
            let body = res.body_bytes().await?;
            Ok((body, res.status()))
        }
    )
}
#[cfg(feature = "redirect")]
fn load_url(url: &str) -> Result<(Vec<u8>,StatusCode),
                                    Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(
        async {
            let req = surf::get(url);
            let client = surf::client().with(surf::middleware::Redirect::new(5));
            let mut res = client.send(req).await?;
            let body = res.body_bytes().await?;
            Ok((body, res.status()))
        }
    )
}
fn build_example_function(app: &Application) {
    let draw_table = app.draw_table.clone();
    app.env.add_builtin_ext_func("load-image-url", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!(ErrCode::E1007));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(ErrCode::E1015)),
        };

        let url = if let Expression::String(s) = eval(&exp[2],env)? {
            s
        } else {
            return Err(create_error!(ErrCode::E1015));
        };
        if !url.starts_with("http://")  && !url.starts_with("https://") {
            return Err(create_error!(ErrCode::E1021));
        }

        let img = match load_url(&url) {
            Err(e) => {
                println!("{:?}", e);
                return Err(create_error!(ErrCode::E9999));
            },
            Ok(s) => {
                if s.1 != 200 {
                    println!("{}", s.1);
                    return Err(create_error!(ErrCode::E9999));
                }
                s.0
            }
        };
        let b = Bytes::from_owned(img);
        let stream = MemoryInputStream::from_bytes(&b);
        let pix = match Pixbuf::from_stream(&stream, None::<&Cancellable>) {
            Ok(pix) => pix,
            Err(_) => return Err(create_error!(ErrCode::E9999)),
        };
        draw_table.regist(symbol, Rc::new(PixbufWrapper::new(pix)));
        Ok(Expression::Nil())
    });
}
struct Application<'a> {
    env: &'a Environment,
    draw_table: &'a DrawTable,
}
impl<'a> Application<'a> {
    fn new(env: &'a Environment, draw_table: &'a DrawTable) -> Self {
        Application {
            env,
            draw_table,
        }
    }
}
fn create_app<'a>(env: &'a Environment, draw_table: &'a DrawTable) -> Application<'a> {
    Application::new(env, draw_table)
}
fn build_draw_ui(app: &Application) {
    let env = RefCell::new(app.env.clone());
    let mut env = env.borrow_mut();

    // Create Lisp Function
    build_lisp_function(&mut env, app.draw_table);
    build_demo_function(&mut env, app.draw_table);
    build_example_function(app);

    scheme_gtk(&mut env, app.draw_table);
    gtk::main();
}
fn main() {
    let env = Environment::new();
    let draw_table = create_draw_table();

    let app = create_app(&env, &draw_table);
    build_draw_ui(&app);
}
