/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
use elisp::lisp;
use elisp::lisp::EvalResult;

fn main() {
    let mut env = lisp::SimpleEnv::new();
    match lisp::do_core_logic(
        "(let loop ((i 0)) (if (<= 100000 i) i (loop (+ i 1))))".to_string(),
        &mut env,
    ) {
        Ok(r) => println!("{}", r.value_string()),
        Err(e) => println!("{}", e.get_code()),
    }
}
