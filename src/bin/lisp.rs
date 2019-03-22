/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
use elisp::{lisp};

fn main() {
    lisp::do_interactive();
}
