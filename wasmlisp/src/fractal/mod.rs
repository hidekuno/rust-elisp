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

use crate::draw::create_draw_line;
use elisp::draw::util::make_lisp_function;
use elisp::lisp;
use lisp::Environment;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

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
    // create each demo program
    // ----------------------------------------------------------------
    make_lisp_function(Box::new(Koch::new(create_draw_line(&context))), env);
    make_lisp_function(Box::new(Tree::new(create_draw_line(&context))), env);
    make_lisp_function(Box::new(Sierpinski::new(create_draw_line(&context))), env);
}
