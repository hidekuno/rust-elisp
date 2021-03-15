/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   this is library for glis,wasmlisp.

   hidekuno@gmail.com
*/
pub mod coord;
pub mod util;

pub trait Fractal {
    fn get_func_name(&self) -> &'static str;
    fn do_demo(&self, c: i32);
}
pub type DrawLine = Box<dyn Fn(f64, f64, f64, f64) + 'static>;
