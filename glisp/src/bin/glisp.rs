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
use glisp::ui;

use lisp::Environment;

use buildin::build_demo_function;
use buildin::build_lisp_function;
use draw::create_draw_table;
use ui::scheme_gtk;

// rustc --explain E0255
use gtk::main as gtk_main;

fn main() {
    // https://doc.rust-jp.rs/book/second-edition/ch15-05-interior-mutability.html
    let mut env = Environment::new();
    let draw_table = create_draw_table();

    // Create Lisp Function
    build_lisp_function(&mut env, &draw_table);
    build_demo_function(&mut env, &draw_table);

    scheme_gtk(&mut env, &draw_table);

    gtk_main();
}
