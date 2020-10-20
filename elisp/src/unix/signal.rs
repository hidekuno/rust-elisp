/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  hidekuno@gmail.com
*/
use crate::lisp::Environment;

static mut CTRLC: bool = false;

#[allow(improper_ctypes)]
extern "C" {
    fn signal(sig: u32, cb: extern "C" fn(u32)) -> fn(u32);
}

extern "C" fn interrupt(_sig: u32) {
    unsafe {
        CTRLC = true;
    }
}
pub fn init_sig_intr() {
    unsafe {
        signal(2, interrupt);
    }
}
pub fn catch_sig_intr_status(env: &Environment) {
    unsafe {
        if CTRLC {
            CTRLC = false;
            env.set_force_stop(true);
        }
    }
}
