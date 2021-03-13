/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
pub mod koch;
pub mod sierpinski;
pub mod tree;

extern crate elisp;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use koch::Koch;
use sierpinski::Sierpinski;
use tree::Tree;

use elisp::create_error;
use elisp::lisp;

use crate::draw::create_draw_line;

use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;
use lisp::Expression;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub trait Fractal {
    fn get_func_name(&self) -> &'static str;
    fn do_demo(&self, c: i32);
}
pub fn build_demo_function(env: &Environment, document: &web_sys::Document) {
    let canvas = document
        .get_element_by_id("drawingarea")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    // ----------------------------------------------------------------
    // create new lisp interface
    // ----------------------------------------------------------------
    fn make_lisp_function(fractal: Box<dyn Fractal>, env: &Environment) {
        env.add_builtin_ext_func(fractal.get_func_name(), move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!(ErrCode::E1007));
            }
            let c = match lisp::eval(&exp[1], env)? {
                Expression::Integer(c) => c,
                _ => return Err(create_error!(ErrCode::E1002)),
            };
            fractal.do_demo(c as i32);
            Ok(Expression::Nil())
        });
    }
    // ----------------------------------------------------------------
    // create each demo program
    // ----------------------------------------------------------------
    make_lisp_function(Box::new(Koch::new(create_draw_line(&context))), env);
    make_lisp_function(Box::new(Tree::new(create_draw_line(&context))), env);
    make_lisp_function(Box::new(Sierpinski::new(create_draw_line(&context))), env);
}
