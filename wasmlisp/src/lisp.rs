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
extern crate wasm_bindgen;
extern crate web_sys;

use crate::alert;
use crate::console_log;
use crate::log;

use elisp::create_error;
use elisp::lisp;

use lisp::do_core_logic;
use lisp::Environment;
use lisp::Expression;
use lisp::RsCode;
use lisp::RsError;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let button = document
        .get_element_by_id("eval")
        .unwrap()
        .dyn_into::<web_sys::Element>()
        .unwrap();

    let text = document
        .get_element_by_id("codearea")
        .unwrap()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap();

    let env = Environment::new();
    build_lisp_function(&env, &document);

    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
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
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
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
                if let Expression::Float(f) = lisp::eval(e, env)? {
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
        Ok(Expression::Integer(2))
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
    // ex. (draw-image "roger" 0.0 0.0 1.0 0.0 0.0 1.0)
    //--------------------------------------------------------
    let doc = document.clone();
    let ctx = context.clone();
    env.add_builtin_ext_func("draw-image", move |exp, env| {
        if exp.len() != 8 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        let mut iter = exp[2 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = lisp::eval(e, env)? {
                    ctm[i] = f;
                } else {
                    return Err(create_error!(RsCode::E1003));
                }
            } else {
                return Err(create_error!(RsCode::E1007));
            }
        }
        let img = match doc.get_element_by_id(&symbol) {
            Some(e) => e.dyn_into::<web_sys::HtmlImageElement>().unwrap(),
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
        if exp.len() != 3 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };

        let url = match lisp::eval(&exp[2], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        // update if it exists
        let img = match doc.get_element_by_id(&symbol) {
            Some(e) => e.dyn_into::<web_sys::HtmlImageElement>().unwrap(),
            None => doc
                .create_element("img")
                .unwrap()
                .dyn_into::<web_sys::HtmlImageElement>()
                .unwrap(),
        };
        img.set_id(&symbol);
        img.style().set_property("display", "none").unwrap();
        img.set_src(&url);
        doc.body().unwrap().append_child(&img).unwrap();

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // (image-width)
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("image-width", move |exp, env| {
        let (w, _) = image_size(exp, env, &doc)?;
        Ok(Expression::Float(w))
    });
    //--------------------------------------------------------
    // (image-height)
    //--------------------------------------------------------
    let doc = document.clone();
    env.add_builtin_ext_func("image-height", move |exp, env| {
        let (_, h) = image_size(exp, env, &doc)?;
        Ok(Expression::Float(h))
    });
    fn image_size(
        exp: &[Expression],
        env: &Environment,
        doc: &web_sys::Document,
    ) -> Result<(f64, f64), RsError> {
        if exp.len() != 2 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };

        let img = match doc.get_element_by_id(&symbol) {
            Some(e) => e.dyn_into::<web_sys::HtmlImageElement>().unwrap(),
            None => return Err(create_error!(RsCode::E9999)),
        };
        Ok((img.width() as f64, img.height() as f64))
    }
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
    fn create_document() -> web_sys::Document {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element("body").unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
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
        assert_eq!(do_lisp_env("(gtk-major-version)", &env), "2");
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
        // why zero?
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
        // why zero?
        assert_eq!(do_lisp_env("(image-height \"roger\")", &env), "0");
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
    fn create_document() -> web_sys::Document {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element("body").unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
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
}
