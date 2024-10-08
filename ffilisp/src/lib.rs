/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   Refer to the following URL
     https://saidvandeklundert.net/2021-11-06-calling-rust-from-python/

   hidekuno@gmail.com
*/
extern crate elisp;
use elisp::lisp::Environment;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref ENV: Arc<Environment> = Arc::new(Environment::new());
}

// Fix below clippy error.
//
// https://rust-lang.github.io/rust-clippy/master/index.html#/not_unsafe_ptr_arg_deref
// https://rust-lang.github.io/rust-clippy/master/index.html#/missing_safety_doc
#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe extern "C" fn do_scheme(program: *const c_char) -> *mut c_char {
    let env = &ENV;

    let program = CStr::from_ptr(program).to_str().unwrap();
    let value = match elisp::lisp::do_core_logic(program, env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_msg(),
    };
    CString::new(value).unwrap().into_raw()
}
