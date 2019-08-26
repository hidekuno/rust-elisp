/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate cairo;
extern crate gdk;

extern crate elisp;

use super::fractal::dragon::Dragon;
use super::fractal::koch::Koch;
use super::fractal::sierpinski::Sierpinski;
use super::fractal::tree::Tree;
use crate::draw::draw_clear;
use crate::draw::ImageTable;
use crate::draw::DEFALUT_CANVAS;
use crate::draw::DRAW_HEIGHT;
use crate::draw::DRAW_WIDTH;
use crate::get_default_surface;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use lisp::Environment;
use lisp::Expression;
use lisp::RsError;

use cairo::{Context, ImageSurface, Matrix};
use std::fs::File;
use std::rc::Rc;

#[cfg(feature = "animation")]
macro_rules! force_event_loop {
    () => {
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    };
}
pub fn build_lisp_function(env: &Environment, image_table: &ImageTable) {
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    env.add_builtin_ext_func("draw-clear", move |exp, _| {
        if exp.len() != 1 {
            return Err(create_error!("E1007"));
        }
        draw_clear(&Context::new(&*surface));
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // DrawLine
    // ex. (draw-line 0.0 0.0 1.0 1.0)
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    env.add_builtin_ext_func("draw-line", move |exp, env| {
        const N: usize = 4;
        if exp.len() != (N + 1) {
            return Err(create_error!("E1007"));
        }
        let mut loc: [f64; N] = [0.0; N];
        let mut iter = exp[1 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = lisp::eval(e, env)? {
                    loc[i] = f;
                } else {
                    return Err(create_error!("E1003"));
                }
            }
        }
        let (x0, y0, x1, y1) = (loc[0], loc[1], loc[2], loc[3]);

        let cr = Context::new(&*surface);
        cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(0.001);
        cr.move_to(x0, y0);
        cr.line_to(x1, y1);
        cr.stroke();

        #[cfg(feature = "animation")]
        force_event_loop!();

        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Create Image
    // ex. (create-image-from-png "roger" "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png")
    //--------------------------------------------------------
    let image_table_clone = image_table.clone();
    env.add_builtin_ext_func("create-image-from-png", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!("E1007"));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let filename = match lisp::eval(&exp[2], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(create_error_value!("E9999", e)),
        };
        let surface = match ImageSurface::create_from_png(&mut file) {
            Ok(s) => s,
            Err(e) => return Err(create_error_value!("E9999", e)),
        };
        (*image_table_clone)
            .borrow_mut()
            .insert(symbol, Rc::new(surface));
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Draw Image
    // ex. (draw-image "roger" (list -1.0 0.0 0.0 1.0 180.0 0.0))
    // ex. (draw-image "roger" (list 1.0 0.0 0.0 1.0 0.0 0.0))
    //--------------------------------------------------------
    let surface = get_default_surface!(image_table);
    let image_table_clone = image_table.clone();
    env.add_builtin_ext_func("draw-image", move |exp, env| {
        if exp.len() != 3 {
            return Err(create_error!("E1007"));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        let img = match (*image_table_clone).borrow().get(&symbol) {
            Some(v) => v.clone(),
            None => return Err(create_error!("E1008")),
        };

        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        if let Expression::List(l) = lisp::eval(&exp[2], env)? {
            if l.len() != 6 {
                return Err(create_error!("E1007"));
            }
            let mut iter = l.iter();
            for i in 0..N {
                if let Some(e) = iter.next() {
                    if let Expression::Float(f) = lisp::eval(e, env)? {
                        ctm[i] = f;
                    } else {
                        return Err(create_error!("E1003"));
                    }
                }
            }
        } else {
            return Err(create_error!("E1005"));
        }
        let cr = Context::new(&*surface);
        cr.scale(1.0, 1.0);
        cr.move_to(0.0, 0.0);
        let matrix = Matrix {
            xx: ctm[0],
            yx: ctm[1],
            xy: ctm[2],
            yy: ctm[3],
            x0: ctm[4],
            y0: ctm[5],
        };
        cr.transform(matrix);
        cr.set_source_surface(&*img, 0.0, 0.0);
        cr.paint();
        #[cfg(feature = "animation")]
        force_event_loop!();

        Ok(Expression::Nil())
    });
}
pub fn build_demo_function(env: &Environment, image_table: &ImageTable) {
    macro_rules! make_demo_closure {
        ($drawable: ty, $func: expr, $env: expr, $image_table: expr) => {
            let surface = get_default_surface!($image_table);

            $env.add_builtin_ext_func($func, move |exp, env| {
                if exp.len() != 2 {
                    return Err(create_error!("E1007"));
                }
                let c = match lisp::eval(&exp[1], env)? {
                    Expression::Integer(c) => c,
                    _ => return Err(create_error!("E1002")),
                };
                let cr = Context::new(&*surface);
                cr.scale(DRAW_WIDTH as f64, DRAW_HEIGHT as f64);
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(0.001);

                let fractal = <$drawable>::new(
                    c,
                    Box::new(move |x0, y0, x1, y1| {
                        cr.move_to(x0, y0);
                        cr.line_to(x1, y1);
                        cr.stroke();
                        #[cfg(feature = "animation")]
                        force_event_loop!();
                    }),
                );
                fractal.do_demo();
                Ok(Expression::Nil())
            });
        };
    }
    make_demo_closure!(Koch, "draw-koch", env, image_table);
    make_demo_closure!(Tree, "draw-tree", env, image_table);
    make_demo_closure!(Sierpinski, "draw-sierpinski", env, image_table);
    make_demo_closure!(Dragon, "draw-dragon", env, image_table);
}
