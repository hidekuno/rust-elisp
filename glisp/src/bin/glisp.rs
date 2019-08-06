/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate glisp;

use elisp::lisp;
use glisp::draw;

use draw::{init_image_table, scheme_gtk};
use lisp::Environment;

// rustc --explain E0255
use gtk::main as gtk_main;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let mut env = Environment::new();
    let image_table = Rc::new(RefCell::new(HashMap::new()));

    init_image_table(&image_table);
    scheme_gtk(&mut env, &image_table);

    gtk_main();
}
