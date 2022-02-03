/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate wasm_bindgen;

#[macro_use]
extern crate lazy_static;

use elisp::lisp;
use lisp::do_core_logic;
use lisp::Environment;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// error[E0277]: `*const Environment` cannot be shared between threads safely
// static PTR: *const Environment = 0 as *const Environment;
//
// error[E0277]: `std::rc::Rc<Environment>` cannot be shared between threads safely
lazy_static! {
    static ref ENV: Arc<Environment> = Arc::new(Environment::new());
}

#[wasm_bindgen]
pub fn do_scheme(code: String) -> String {
    let env = &ENV;

    match do_core_logic(&code, &env) {
        Ok(r) => r.to_string(),
        Err(e) => e.get_msg(),
    }
}
