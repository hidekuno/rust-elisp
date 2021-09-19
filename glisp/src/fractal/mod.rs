/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
pub mod dragon;
pub mod hilbert;
pub mod koch;
pub mod sierpinski;
pub mod tree;
use elisp::lisp::Error;

pub trait FractalMut {
    fn get_func_name(&self) -> &'static str;
    fn do_demo(&mut self, c: i32) -> Result<(), Error>;
}
