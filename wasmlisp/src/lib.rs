/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   1. howto test)
      wasm-pack test --headless --chrome -- --lib
       or
      wasm-pack test --headless --firefox -- --lib

   2. build & run)
      wasm-pack build
      npm install
      npm run lisp

   hidekuno@gmail.com
*/
pub mod draw;
pub mod lisp;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate elisp;
extern crate wasm_bindgen_test;
extern crate web_sys;

use crate::lisp::build_lisp_function;
use elisp::lisp::do_core_logic;
use elisp::lisp::Environment;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, HtmlCanvasElement};

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_many(a: &str, b: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(test)]
const CANVAS_WIDTH: u32 = 720;

#[cfg(test)]
const CANVAS_HEIGHT: u32 = 560;

#[cfg(test)]
const IMG_URL: &'static str =
    "https://github.com/hidekuno/picture-language/blob/master/sicp/sicp.png?raw=true";

#[cfg(test)]
fn do_lisp_env(program: &str, env: &Environment) -> String {
    match do_core_logic(&program.into(), env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[cfg(test)]
fn create_document() -> Document {
    let document = web_sys::window().unwrap().document().unwrap();
    document.create_element("body").unwrap();

    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    canvas.set_id("drawingarea");
    canvas.set_width(CANVAS_WIDTH);
    canvas.set_height(CANVAS_HEIGHT);
    document.body().unwrap().append_child(&canvas).unwrap();
    document
}
#[cfg(test)]
mod tests {
    use super::*;

    const SD_URL: &'static str =
        "https://coverartarchive.org/release-group/9b1acd78-3d19-37bb-8ca0-5816d44da439/front-250.jpg";

    const RV_URL: &'static str =
        "https://coverartarchive.org/release-group/72d15666-99a7-321e-b1f3-a3f8c09dff9f/front-250.jpg";

    const PS_URL: &'static str =
        "https://coverartarchive.org/release-group/fdd96703-7b21-365e-bdea-38029fbeb84e/front-250.jpg";

    wasm_bindgen_test_configure!(run_in_browser);

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
        assert_eq!(
            do_lisp_env("(screen-width)", &env),
            CANVAS_WIDTH.to_string()
        );
    }
    #[wasm_bindgen_test]
    fn screen_height() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(
            do_lisp_env("(screen-height)", &env),
            CANVAS_HEIGHT.to_string()
        );
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
    #[wasm_bindgen_test]
    fn add_timeout() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(add-timeout (+ 1 1) 10)", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn set_foreground() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(set-foreground \"blue\")", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn set_background() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(set-background \"black\")", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn set_line_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(set-line-width 1.0)", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn draw_arc() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-arc 75.0 75.0 50.0 0.0)", &env), "nil");
    }
    #[wasm_bindgen_test]
    fn draw_string() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(
            do_lisp_env("(draw-string \"Hello,World\" 20.0 20.0)", &env),
            "nil"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-string \"Hello,World\" 20.0 20.0 \"italic bold 20px sans-serif\")",
                &env
            ),
            "nil"
        );
    }
    #[wasm_bindgen_test]
    fn draw_eval() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-eval (iota 20))", &env), "nil");
    }
}
#[cfg(test)]
mod error_tests {
    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

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
    #[wasm_bindgen_test]
    fn add_timeout() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(add-timeout)", &env), "E1007");
        assert_eq!(do_lisp_env("(add-timeout (+ 1 1) 10 10)", &env), "E1007");
        assert_eq!(do_lisp_env("(add-timeout (+ 1 1) #t)", &env), "E1002");
        assert_eq!(do_lisp_env("(add-timeout (+ 1 1) 0)", &env), "E1021");
    }
    #[wasm_bindgen_test]
    fn set_foreground() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);

        assert_eq!(do_lisp_env("(set-foreground)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(set-foreground \"black\" \"black\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(set-foreground #t)", &env), "E1015");
    }
    #[wasm_bindgen_test]
    fn set_background() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);

        assert_eq!(do_lisp_env("(set-background)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(set-background \"black\" \"black\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(set-background #t)", &env), "E1015");
    }
    #[wasm_bindgen_test]
    fn set_line_width() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);

        assert_eq!(do_lisp_env("(set-line-width)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-line-width 1.0 1.0)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-line-width #t)", &env), "E1003");
    }
    #[wasm_bindgen_test]
    fn draw_arc() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);

        assert_eq!(do_lisp_env("(draw-arc)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-arc 0.27 0.65 0.02 0.0 1.0)", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(draw-arc #t 0.65 0.02 0.0)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-arc 0.27 0.65 0.02 #t)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-arc 0.27 0.65 #t 0.0)", &env), "E1003");
    }
    #[wasm_bindgen_test]
    fn draw_string() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);
        assert_eq!(do_lisp_env("(draw-string)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-string \"Hello,World\" 20.0)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-string \"Hello,World\" 20.0 20.0 \"italic bold 20px sans-serif\" 10.0)",
                &env
            ),
            "E1007"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-string 10.0 20.0 20.0 \"italic bold 20px sans-serif\")",
                &env
            ),
            "E1015"
        );
        assert_eq!(
            do_lisp_env("(draw-string \"Hello,World\" 20.0 20.0 10.0)", &env),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-string \"Hello,World\" #t 20.0 \"italic bold 20px sans-serif\")",
                &env
            ),
            "E1003"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-string \"Hello,World\" 20.0 #t \"italic bold 20px sans-serif\")",
                &env
            ),
            "E1003"
        );
    }
    #[wasm_bindgen_test]
    fn draw_eval() {
        let document = create_document();
        let env = Environment::new();
        build_lisp_function(&env, &document);

        assert_eq!(do_lisp_env("(draw-eval)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-eval (iota 20) 10)", &env), "E1007");
    }
}
