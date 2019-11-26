/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   1. howto test)
      wasm-pack test --headless --chrome -- --lib

   2. build & run)
      wasm-pack build
      npm install
      npm run lisp

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use crate::alert;
use crate::clearInterval;
use crate::console_log;
use crate::log;
use crate::setInterval;

use elisp::create_error;
use elisp::lisp;

use lisp::do_core_logic;
use lisp::eval;
use lisp::repl;
use lisp::Environment;
use lisp::Expression;
use lisp::RsCode;
use lisp::RsError;

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlImageElement,
    HtmlTextAreaElement, Request, RequestInit, RequestMode, Response,
};
const SCHEME_URL: &'static str =
    "https://raw.githubusercontent.com/hidekuno/picture-language/master";

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

    let closure = Closure::wrap(Box::new(move |_event: Event| {
        let result = match do_core_logic(&text.value(), &env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_msg(),
        };
        alert(&result);
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
fn build_lisp_function(env: &Environment, document: &web_sys::Document) {
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

    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    let ctx = context.clone();
    env.add_builtin_ext_func("draw-clear", move |exp, _| {
        if exp.len() != 1 {
            return Err(create_error!(RsCode::E1007));
        }
        ctx.clear_rect(0.0, 0.0, 720.0, 560.0);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw Line
    // ex. (draw-line 10.0 10.0 100.0 100.0)
    //--------------------------------------------------------
    let ctx = context.clone();
    env.add_builtin_ext_func("draw-line", move |exp, env| {
        const N: usize = 4;
        if exp.len() != (N + 1) {
            return Err(create_error!(RsCode::E1007));
        }
        let mut loc: [f64; N] = [0.0; N];
        let mut iter = exp[1 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = eval(e, env)? {
                    loc[i] = f;
                } else {
                    return Err(create_error!(RsCode::E1003));
                }
            }
        }
        ctx.begin_path();
        ctx.set_line_width(0.8);
        ctx.move_to(loc[0], loc[1]);
        ctx.line_to(loc[2], loc[3]);
        ctx.close_path();
        ctx.stroke();
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // ex. (gtk-major-version)
    //--------------------------------------------------------
    env.add_builtin_ext_func("gtk-major-version", move |exp, _env| {
        if exp.len() != 1 {
            return Err(create_error!(RsCode::E1007));
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
            return Err(create_error!(RsCode::E1007));
        }
        Ok(Expression::Float(c.width() as f64))
    });
    //--------------------------------------------------------
    // ex. (screen-height)
    //--------------------------------------------------------
    let c = canvas.clone();
    env.add_builtin_ext_func("screen-height", move |exp, _env| {
        if exp.len() != 1 {
            return Err(create_error!(RsCode::E1007));
        }
        Ok(Expression::Float(c.height() as f64))
    });
    //--------------------------------------------------------
    // ex. (draw-image "roger" 0.0 0.0 180.0 0.0 0.0 180.0)
    //--------------------------------------------------------
    let doc = document.clone();
    let ctx = context.clone();
    env.add_builtin_ext_func("draw-image", move |exp, env| {
        if exp.len() != 8 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        let mut iter = exp[2 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = eval(e, env)? {
                    ctm[i] = f;
                } else {
                    return Err(create_error!(RsCode::E1003));
                }
            } else {
                return Err(create_error!(RsCode::E1007));
            }
        }
        let img = match doc.get_element_by_id(&symbol) {
            Some(e) => e.dyn_into::<HtmlImageElement>().unwrap(),
            None => return Err(create_error!(RsCode::E1008)),
        };
        let w = img.width() as f64;
        let h = img.height() as f64;
        if let Err(_) = ctx.set_transform(
            ctm[2] / w,
            ctm[3] / h,
            ctm[4] / w,
            ctm[5] / h,
            ctm[0],
            ctm[1],
        ) {
            return Err(create_error!(RsCode::E9999));
        }

        // https://rustwasm.github.io/wasm-bindgen/api/web_sys/
        if let Err(_) = ctx.draw_image_with_html_image_element(&img, 0.0, 0.0) {
            return Err(create_error!(RsCode::E9999));
        }
        if let Err(_) = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0) {
            return Err(create_error!(RsCode::E9999));
        }
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // ex. (load-image "roger"
    //        "https://github.com/hidekuno/picture-language/blob/master/sicp/sicp.png?raw=true")
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("load-image", move |exp, env| {
        if exp.len() != 3 && exp.len() != 4 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        let url = match eval(&exp[2], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
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
        if exists == false {
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
            img.decode().then(&closure);
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
    fn image_size(
        exp: &[Expression],
        env: &Environment,
        doc: &Document,
    ) -> Result<(f64, f64), RsError> {
        if exp.len() != 2 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        let img = match doc.get_element_by_id(&symbol) {
            Some(e) => e.dyn_into::<HtmlImageElement>().unwrap(),
            None => return Err(create_error!(RsCode::E9999)),
        };
        Ok((img.width() as f64, img.height() as f64))
    }
    //--------------------------------------------------------
    // ex. (load-url "sicp/segments-fish.scm")
    //--------------------------------------------------------
    env.add_builtin_ext_func("load-url", move |exp, env| {
        if exp.len() != 2 && exp.len() != 3 {
            return Err(create_error!(RsCode::E1007));
        }
        let scm = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        let env_ = env.clone();
        let closure = Closure::wrap(Box::new(move |v: JsValue| {
            if let Some(s) = v.as_string() {
                let mut cur = Cursor::new(s.into_bytes());
                if let Err(e) = repl(&mut cur, &env_, true) {
                    console_log!("load-url {:?}", e);
                }
            }
        }) as Box<dyn FnMut(_)>);
        let promise = future_to_promise(get_program_file(scm));

        if exp.len() == 2 {
            promise.then(&closure);
        } else {
            let env_ = env.clone();
            let e = exp[2].clone();
            let eval = Closure::wrap(Box::new(move |_: JsValue| match eval(&e, &env_) {
                Ok(v) => console_log!("load-url: {}", v.to_string()),
                Err(e) => console_log!("load-url: {}", e.get_code()),
            }) as Box<dyn FnMut(_)>);
            promise.then(&closure).then(&eval);
            eval.forget();
        }
        closure.forget();

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // (wasm-time (image-width "sample") 3)
    //--------------------------------------------------------
    env.add_builtin_ext_func("wasm-time", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error!(RsCode::E1007));
        }

        // std::time::SystemTime::now() causes panic on wasm32
        // https://github.com/rust-lang/rust/issues/48564
        let start = js_sys::Date::now();
        let result = eval(&exp[1], env);
        let end = js_sys::Date::now();

        let t = ((end - start).trunc()) as i64;
        console_log!("{}.{}(s)", t / 1000, t % 1000);
        return result;
    });
    //--------------------------------------------------------
    // ex. (add-timeout (image-width "sample") 3)
    //--------------------------------------------------------
    env.add_builtin_ext_func("add-timeout", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!(RsCode::E1007));
        }
        let t = match eval(&exp[2], env)? {
            Expression::Integer(t) => (t * 1000) as u32,
            _ => return Err(create_error!(RsCode::E1002)),
        };

        let rid = Rc::new(RefCell::new(10));
        let rid_ = rid.clone();
        let env = env.clone();
        let e = exp[1].clone();

        let timeout = Closure::wrap(Box::new(move || {
            let id = rid_.borrow();
            clearInterval(*id);
            match eval(&e, &env) {
                Ok(v) => console_log!("add-timeout: {}", v.to_string()),
                Err(e) => console_log!("add-timeout: {}", e.get_code()),
            }
        }) as Box<dyn FnMut()>);
        let mut id = rid.borrow_mut();
        *id = setInterval(&timeout, t);
        timeout.forget();
        Ok(Expression::Nil())
    });
}
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
#[cfg(test)]
mod tests {
    use super::*;
    extern crate wasm_bindgen_test;
    use wasm_bindgen_test::*;

    const IMG_URL: &'static str =
        "https://github.com/hidekuno/picture-language/blob/master/sicp/sicp.png?raw=true";

    const SD_URL: &'static str =
        "https://coverartarchive.org/release-group/9b1acd78-3d19-37bb-8ca0-5816d44da439/front-250.jpg";

    const RV_URL: &'static str =
        "https://coverartarchive.org/release-group/72d15666-99a7-321e-b1f3-a3f8c09dff9f/front-250.jpg";

    const PS_URL: &'static str =
        "https://coverartarchive.org/release-group/fdd96703-7b21-365e-bdea-38029fbeb84e/front-250.jpg";

    wasm_bindgen_test_configure!(run_in_browser);

    fn do_lisp_env(program: &str, env: &Environment) -> String {
        match do_core_logic(&program.into(), env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }
    fn create_document() -> Document {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element("body").unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_id("drawingarea");
        canvas.set_width(720);
        canvas.set_height(560);
        document.body().unwrap().append_child(&canvas).unwrap();
        document
    }
    #[wasm_bindgen_test]
    fn draw_clear() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-clear)", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn draw_line() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(
            do_lisp_env("(draw-line 10.0 10.0 100.0 100.0)", &env),
            "nil"
        );
    }
    #[wasm_bindgen_test]
    fn gtk_major_version() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(gtk-major-version)", &env), "-1");
    }
    #[wasm_bindgen_test]
    fn screen_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(screen-width)", &env), "720");
    }
    #[wasm_bindgen_test]
    fn screen_height() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(screen-height)", &env), "560");
    }
    #[wasm_bindgen_test]
    fn load_image() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(
            do_lisp_env(
                format!("(load-image \"roger\" \"{}\")", IMG_URL).as_str(),
                &env
            ),
            "nil"
        );
    }
    #[wasm_bindgen_test]
    fn draw_image() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"roger\" \"{}\")", SD_URL).as_str(),
            &env,
        );
        assert_eq!(
            do_lisp_env("(draw-image \"roger\" 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "nil"
        );
    }
    #[wasm_bindgen_test]
    fn image_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"roger\" \"{}\")", RV_URL).as_str(),
            &env,
        );
        // NG because It's Asynchronous processing
        assert_eq!(do_lisp_env("(image-width \"roger\")", &env), "0");
    }
    #[wasm_bindgen_test]
    fn image_height() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"roger\" \"{}\")", PS_URL).as_str(),
            &env,
        );
        // NG because It's Asynchronous processing
        assert_eq!(do_lisp_env("(image-height \"roger\")", &env), "0");
    }
    #[wasm_bindgen_test]
    fn load_url() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(
            do_lisp_env("(load-url \"sicp/abstract-data.scm\")", &env),
            "nil"
        );
        // NG because It's Asynchronous processing
        // left: `"E1008"`,
        // right: `"Function"`', src/lisp.rs:493:9
        // assert_eq!(do_lisp_env("make-frame", &env), "Function");
    }
}
#[cfg(test)]
mod error_tests {
    use super::*;
    extern crate wasm_bindgen_test;
    use wasm_bindgen_test::*;

    const IMG_URL: &'static str =
        "https://github.com/hidekuno/picture-language/blob/master/sicp/sicp.png?raw=true";

    wasm_bindgen_test_configure!(run_in_browser);
    fn do_lisp_env(program: &str, env: &Environment) -> String {
        match do_core_logic(&program.into(), env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }
    fn create_document() -> Document {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element("body").unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_id("drawingarea");
        canvas.set_width(720);
        canvas.set_height(560);
        document.body().unwrap().append_child(&canvas).unwrap();
        document
    }
    #[wasm_bindgen_test]
    fn draw_clear() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-clear 1)", &env), "E1007");
    }
    #[wasm_bindgen_test]
    fn draw_line() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-line)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");
        assert_eq!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");
    }
    #[wasm_bindgen_test]
    fn gtk_major_version() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(gtk-major-version 1)", &env), "E1007");
    }
    #[wasm_bindgen_test]
    fn screen_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(screen-width 1)", &env), "E1007");
    }
    #[wasm_bindgen_test]
    fn screen_height() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(screen-height 1)", &env), "E1007");
    }
    #[wasm_bindgen_test]
    fn load_image() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(load-image)", &env), "E1007");
        assert_eq!(do_lisp_env("(load-image  \"sample\")", &env), "E1007");
        assert_eq!(
            do_lisp_env("(load-image  \"sample\" 10 20 30)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env(format!("(load-image 10 \"{}\")", IMG_URL).as_str(), &env),
            "E1015"
        );
        assert_eq!(do_lisp_env("(load-image \"sample\" 10)", &env), "E1015");
    }
    #[wasm_bindgen_test]
    fn draw_image() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"sample\" \"{}\")", IMG_URL).as_str(),
            &env,
        );
        assert_eq!(do_lisp_env("(draw-image)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-image 10)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 1 2 3 10)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env("(draw-image 10 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "E1015"
        );
        assert_eq!(
            do_lisp_env("(draw-image \"sample1\" 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "E1008"
        );
        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 0.0 0.0 1.0 1.0 1.0 10)", &env),
            "E1003"
        );
        assert_eq!(
            do_lisp_env("(draw-image \"sample\" a 0.0 1.0 1.0 1.0 10)", &env),
            "E1008"
        );
    }
    #[wasm_bindgen_test]
    fn image_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"sample\" \"{}\")", IMG_URL).as_str(),
            &env,
        );
        assert_eq!(do_lisp_env("(image-width)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(image-width \"sample\" \"sample1\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(image-width 10)", &env), "E1015");
        assert_eq!(do_lisp_env("(image-width a)", &env), "E1008");
    }
    #[wasm_bindgen_test]
    fn image_height() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        do_lisp_env(
            format!("(load-image \"sample\" \"{}\")", IMG_URL).as_str(),
            &env,
        );
        assert_eq!(do_lisp_env("(image-height)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(image-height \"sample\" \"sample1\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(image-height 10)", &env), "E1015");
        assert_eq!(do_lisp_env("(image-height a)", &env), "E1008");
    }
    #[wasm_bindgen_test]
    fn load_url() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(load-url)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(load-url \"sicp/abstract-data.scm\" 10 12)", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(load-url 10)", &env), "E1015");
        assert_eq!(do_lisp_env("(load-url a)", &env), "E1008");
    }
}
