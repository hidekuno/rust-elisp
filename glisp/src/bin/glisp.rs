/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate glisp;

use elisp::lisp;
use glisp::buildin;
use glisp::draw;

use lisp::Environment;

use buildin::build_demo_function;
use buildin::build_lisp_function;
use draw::{create_image_table, scheme_gtk};

// rustc --explain E0255
use gtk::main as gtk_main;

fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let env = Environment::new();
    let image_table = create_image_table();

    // Create Lisp Function
    build_lisp_function(&env, &image_table);
    build_demo_function(&env, &image_table);

    scheme_gtk(&env, &image_table);

    gtk_main();
}
