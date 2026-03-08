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
/// - `program` must be a valid, null-terminated C string (UTF-8).
/// - The caller is responsible for freeing the returned pointer using `free_scheme_result`.
pub unsafe extern "C" fn do_scheme(program: *const c_char) -> *mut c_char {
    if program.is_null() {
        return std::ptr::null_mut();
    }

    let env = &ENV;

    // Convert C string to Rust string safely (though still unsafe due to pointer deref)
    let c_str = CStr::from_ptr(program);
    let program_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
    };

    let value = match elisp::lisp::do_core_logic(program_str, env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_msg(),
    };

    // Use into_raw to transfer ownership to the caller (Python side)
    match CString::new(value) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => CString::new("Error: Result contains null byte").unwrap().into_raw(),
    }
}

#[no_mangle]
/// # Safety
///
/// - `ptr` must be a pointer previously returned by `do_scheme`.
/// - It must not be null and must not have been freed already.
pub unsafe extern "C" fn free_scheme_result(ptr: *mut c_char) {
    if !ptr.is_null() {
        // Re-take ownership of the pointer and let it drop to free memory
        let _ = CString::from_raw(ptr);
    }
}
