/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   1. wasm-pack build -t web
   2. python -m http.server

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate js_sys;
extern crate wasm_bindgen;

#[macro_use]
extern crate lazy_static;

use elisp::create_error_value;
use elisp::lisp;
use lisp::do_core_logic;
use lisp::eval;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;

use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// error[E0277]: `*const Environment` cannot be shared between threads safely
// static PTR: *const Environment = 0 as *const Environment;
//
// error[E0277]: `std::rc::Rc<Environment>` cannot be shared between threads safely
lazy_static! {
    static ref ENV: Arc<Environment> = Arc::new(Environment::new());
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let env = &ENV;

    env.add_builtin_ext_func("wasm-time", move |exp, env| {
        match exp.len() {
            2 => {
                // std::time::SystemTime::now() causes panic on wasm32
                // https://github.com/rust-lang/rust/issues/48564
                let start = js_sys::Date::now();
                let result = eval(&exp[1], env);
                let end = js_sys::Date::now();

                log(&format!("{}(ms)", (end - start)));
                result
            }
            _ => Err(create_error_value!(ErrCode::E1007, exp.len())),
        }
    });

    Ok(())
}
#[wasm_bindgen]
pub fn do_scheme(code: String) -> String {
    let env = &ENV;

    match do_core_logic(&code, &env) {
        Ok(r) => r.to_string(),
        Err(e) => e.get_msg(),
    }
}
