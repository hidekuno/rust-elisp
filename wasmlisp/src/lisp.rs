/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use crate::alert;
use crate::console_log;
use crate::log;

use crate::draw::create_draw_arc;
use crate::draw::create_draw_image;
use crate::draw::create_draw_line;
use crate::draw::create_draw_string;
use crate::draw::Graphics;
use crate::fractal::build_demo_function;
use elisp::create_error;
use elisp::create_error_value;
use elisp::draw::util::regist_draw_arc;
use elisp::draw::util::regist_draw_image;
use elisp::draw::util::regist_draw_line;
use elisp::lisp;
use lisp::do_core_logic;
use lisp::eval;
use lisp::repl;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;
use lisp::Expression;

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;

use js_sys::Promise;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlImageElement,
    HtmlTextAreaElement, Request, RequestInit, RequestMode, Response,
};

#[cfg(not(feature = "develop"))]
const SCHEME_URL: &str = "https://raw.githubusercontent.com/hidekuno/picture-language/master";

// wasm-pack build -- --features develop
#[cfg(feature = "develop")]
const SCHEME_URL: &str = "https://raw.githubusercontent.com/hidekuno/picture-language/develop";

const LINE_WIDTH: f64 = 0.8;
//--------------------------------------------------------
// entry point
//--------------------------------------------------------
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let button = document
        .get_element_by_id("eval")
        .unwrap()
        .dyn_into::<Element>()
        .unwrap();

    let text = document
        .get_element_by_id("codearea")
        .unwrap()
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap();

    let env = Environment::new();
    build_lisp_function(&env, &document);
    build_demo_function(&env, &document);

    let closure = Closure::wrap(Box::new(move |_event: Event| {
        let c = Closure::wrap(Box::new(move |v: JsValue| {
            alert(&v.as_string().unwrap());
        }) as Box<dyn FnMut(_)>);

        // It's experimental code for study.
        let _ = future_to_promise(execute_lisp(text.value(), env.clone())).then(&c);
        c.forget();

        if let Some(element) = document.get_element_by_id("loading") {
            let loading = element.dyn_into::<Element>().unwrap();
            loading.remove();
        }
        console_log!("eval done.");
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

    // The instance of `Closure` that we created will invalidate its
    // corresponding JS callback whenever it is dropped, so if we were to
    // normally return from `setup_clock` then our registered closure will
    // raise an exception when invoked.
    //
    // Normally we'd store the handle to later get dropped at an appropriate
    // time but for now we want it to be a global handler so we use the
    // `forget` method to drop it without invalidating the closure. Note that
    // this is leaking memory in Rust, so this should be done judiciously!
    closure.forget();

    console_log!("Hello World from Rust");
    Ok(())
}
//--------------------------------------------------------
// It's experimental code for study.
//--------------------------------------------------------
async fn execute_lisp(code: String, env: Environment) -> Result<JsValue, JsValue> {
    fn call_elisp(code: String, env: Environment) -> JsValue {
        let v = match do_core_logic(&code, &env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_msg(),
        };
        JsValue::from(v)
    }
    let text = JsFuture::from(Promise::resolve(&call_elisp(code, env))).await?;
    Ok(text)
}
//--------------------------------------------------------
// lisp functions
//--------------------------------------------------------
pub fn build_lisp_function(env: &Environment, document: &Document) {
    let canvas = document
        .get_element_by_id("drawingarea")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let graphics = Rc::new(RefCell::new(Graphics::new()));

    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    let c = canvas.clone();
    let ctx = context.clone();
    let g = graphics.clone();
    env.add_builtin_ext_func("draw-clear", move |exp, _| {
        if exp.len() != 1 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        if let Some(color) = &g.borrow().bg {
            ctx.set_fill_style(color);
            ctx.fill_rect(0.0, 0.0, c.width() as f64, c.height() as f64);
        } else {
            ctx.clear_rect(0.0, 0.0, c.width() as f64, c.height() as f64);
        }
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw Line
    // ex. (draw-line 10.0 10.0 100.0 100.0)
    // ex. (draw-line (cons 0.0 0.0) (cons 1.0 1.0))
    //--------------------------------------------------------
    context.set_line_width(LINE_WIDTH);
    regist_draw_line("draw-line", env, create_draw_line(&context));

    //--------------------------------------------------------
    // ex. (gtk-major-version)
    //--------------------------------------------------------
    env.add_builtin_ext_func("gtk-major-version", move |exp, _env| {
        if exp.len() != 1 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        // It's dummy code
        Ok(Expression::Integer(-1))
    });
    //--------------------------------------------------------
    // ex. (screen-width)
    //--------------------------------------------------------
    let c = canvas.clone();
    env.add_builtin_ext_func("screen-width", move |exp, _env| {
        if exp.len() != 1 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        Ok(Expression::Float(c.width() as f64))
    });
    //--------------------------------------------------------
    // ex. (screen-height)
    //--------------------------------------------------------
    let c = canvas.clone();
    env.add_builtin_ext_func("screen-height", move |exp, _env| {
        if exp.len() != 1 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        Ok(Expression::Float(c.height() as f64))
    });
    //--------------------------------------------------------
    // ex. (draw-image "roger" 0.0 0.0 180.0 0.0 0.0 180.0)
    //--------------------------------------------------------
    regist_draw_image("draw-image", env, create_draw_image(&context, document));

    //--------------------------------------------------------
    // ex. (load-image "roger"
    //        "https://github.com/hidekuno/picture-language/blob/master/sicp/sicp.png?raw=true")
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("load-image", move |exp, env| {
        if exp.len() != 3 && exp.len() != 4 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        let url = match eval(&exp[2], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        // update if it exists
        let (img, exists) = match doc.get_element_by_id(&symbol) {
            Some(e) => (e.dyn_into::<HtmlImageElement>().unwrap(), true),
            None => (
                doc.create_element("img")
                    .unwrap()
                    .dyn_into::<HtmlImageElement>()
                    .unwrap(),
                false,
            ),
        };
        if !exists {
            img.set_id(&symbol);
            doc.body().unwrap().append_child(&img).unwrap();
        }
        img.style().set_property("display", "none").unwrap();
        img.set_src(&url);

        if exp.len() == 4 {
            let e = exp[3].clone();
            let env = env.clone();
            let closure = Closure::wrap(Box::new(move |_: JsValue| match eval(&e, &env) {
                Ok(v) => console_log!("load-image: {}", v.to_string()),
                Err(e) => console_log!("load-image: {}", e.get_code()),
            }) as Box<dyn FnMut(_)>);

            let _promise = img.decode().then(&closure);
            closure.forget();
        }
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // ex. (image-width "am")
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("image-width", move |exp, env| {
        let (w, _) = image_size(exp, env, &doc)?;
        Ok(Expression::Float(w))
    });
    //--------------------------------------------------------
    // ex. (image-height "am")
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("image-height", move |exp, env| {
        let (_, h) = image_size(exp, env, &doc)?;
        Ok(Expression::Float(h))
    });
    //--------------------------------------------------------
    // ex. (load-url "sicp/segments-fish.scm")
    //--------------------------------------------------------
    env.add_builtin_ext_func("load-url", move |exp, env| {
        if exp.len() != 2 && exp.len() != 3 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let scm = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        let env_ = env.clone();
        let program = scm.to_string();
        let closure = Closure::wrap(Box::new(move |v: JsValue| {
            if let Some(s) = v.as_string() {
                let mut cur = Cursor::new(s.into_bytes());
                if let Err(e) = repl(&mut cur, &env_, None) {
                    console_log!("load-url {} {:?}", program, e);
                } else {
                    console_log!("load-url {}", program);
                }
            }
        }) as Box<dyn FnMut(_)>);
        let promise = future_to_promise(get_program_file(scm.to_string()));

        if exp.len() == 2 {
            let _promise = promise.then(&closure);
        } else {
            let env_ = env.clone();
            let e = exp[2].clone();
            let c = Closure::wrap(Box::new(move |_: JsValue| match eval(&e, &env_) {
                Ok(v) => console_log!("load-url-2 {}", v.to_string()),
                Err(e) => console_log!("load-url-2 {}", e.get_code()),
            }) as Box<dyn FnMut(_)>);
            let _promise = promise.then(&closure).then(&c);
            c.forget();
        }
        closure.forget();

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // (wasm-time (image-width "sample") 3)
    //--------------------------------------------------------
    env.add_builtin_ext_func("wasm-time", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }

        // std::time::SystemTime::now() causes panic on wasm32
        // https://github.com/rust-lang/rust/issues/48564
        let start = js_sys::Date::now();
        let result = eval(&exp[1], env);
        let end = js_sys::Date::now();

        let t = ((end - start).trunc()) as i64;
        console_log!("{}.{}(s)", t / 1000, t % 1000);
        result
    });
    //--------------------------------------------------------
    // ex. (add-timeout (image-width "sample") 10)
    //--------------------------------------------------------
    env.add_builtin_ext_func("add-timeout", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let t = match eval(&exp[2], env)? {
            Expression::Integer(t) => t as i32,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        };
        if t < 1 {
            return Err(create_error!(ErrCode::E1021));
        }
        let env = env.clone();
        let e = exp[1].clone();

        let timeout = Closure::wrap(Box::new(move || match eval(&e, &env) {
            Ok(_) => {}
            Err(e) => console_log!("add-timeout: {}", e.get_code()),
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout.as_ref().unchecked_ref(),
                t,
            )
            .unwrap();
        timeout.forget();
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Set foreground
    // ex. (set-foreground "blue")
    //--------------------------------------------------------
    let ctx = context.clone();
    env.add_builtin_ext_func("set-foreground", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let color = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        ctx.set_stroke_style(&JsValue::from(color.as_ref()));
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Set background
    // ex. (set-background "black")
    //--------------------------------------------------------
    let ctx = context.clone();
    env.add_builtin_ext_func("set-background", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let color = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        let js = JsValue::from(color.as_ref());
        ctx.set_fill_style(&js);
        graphics.borrow_mut().bg = Some(js);

        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Set line width
    // ex. (set-line-width 1.0)
    //--------------------------------------------------------
    let ctx = context.clone();
    env.add_builtin_ext_func("set-line-width", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let width = match eval(&exp[1], env)? {
            Expression::Float(f) => f,
            e => return Err(create_error_value!(ErrCode::E1003, e)),
        };
        ctx.set_line_width(width);
        Ok(Expression::Nil())
    });

    //--------------------------------------------------------
    // draw arc
    // ex. (draw-arc 75.0 75.0 50.0 0.0)
    //--------------------------------------------------------
    regist_draw_arc("draw-arc", env, create_draw_arc(&context));

    //--------------------------------------------------------
    // Draw string
    // ex. (draw-string "hello,world" 0.0 10.0)
    //--------------------------------------------------------
    let draw_string = create_draw_string(&context);
    env.add_builtin_ext_func("draw-string", move |exp, env| {
        if exp.len() < 4 || 5 < exp.len() {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let text = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        const N: usize = 2;
        let mut prm: [f64; N] = [0.0; N];
        for (i, e) in exp[2..4].iter().enumerate() {
            prm[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                e => return Err(create_error_value!(ErrCode::E1003, e)),
            };
        }
        let font = if exp.len() == 5 {
            match lisp::eval(&exp[4], env)? {
                Expression::String(s) => s.to_string(),
                e => return Err(create_error_value!(ErrCode::E1015, e)),
            }
        } else {
            "bold 20px sans-serif".to_string()
        };
        //ex) "italic bold 20px sans-serif"
        draw_string(text.to_string(), prm[0], prm[1], font);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw string
    // ex. (draw-string "hello,world" 0.0 10.0)
    //--------------------------------------------------------
    let draw_string = create_draw_string(&context);
    env.add_builtin_ext_func("draw-eval", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let result = match lisp::eval(&exp[1], env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_msg(),
        };
        draw_string(result, 30.0, 30.0, "bold 20px sans-serif".to_string());
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// image size
// ----------------------------------------------------------------
fn image_size(exp: &[Expression], env: &Environment, doc: &Document) -> Result<(f64, f64), Error> {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let symbol = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let img = match doc.get_element_by_id(&symbol) {
        Some(e) => e.dyn_into::<HtmlImageElement>().unwrap(),
        None => return Err(create_error!(ErrCode::E9999)),
    };
    Ok((img.width() as f64, img.height() as f64))
}
// ----------------------------------------------------------------
// load scheme program
// ----------------------------------------------------------------
async fn get_program_file(scm: String) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&format!("{}/{}", SCHEME_URL, scm), &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    if resp.status() != 200 {
        return Err(JsValue::from(format!(
            "HTTP Error {} {}",
            resp.status(),
            resp.status_text()
        )));
    }
    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text()?).await?;
    console_log!("http get complete: {}", scm);
    Ok(text)
}
