/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  hidekuno@gmail.com
*/
use crate::lisp::Environment;

static mut CTRLC: bool = false;
const SIGINTR: u32 = 2;

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
        signal(SIGINTR, interrupt);
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

//  You can execute test only it.
//  cargo test --lib --features signal unix
#[cfg(all(test, feature = "signal"))]
mod tests {
    use super::*;
    use crate::do_lisp;
    use std::thread;
    use std::time::Duration;

    extern "C" {
        fn getpid() -> u32;
        fn kill(pid: u32, sig: u32) -> u32;
    }

    #[test]
    fn test_force_stop() {
        init_sig_intr();

        let t = thread::spawn(|| unsafe {
            thread::sleep(Duration::from_millis(2000));
            let pid = getpid();
            kill(pid, SIGINTR);
        });
        assert_eq!(
            do_lisp("(let loop ((i 0)) (if (< i 0) i (loop (+ i 1))))"),
            "E9000"
        );
        t.join().unwrap();
    }
}
