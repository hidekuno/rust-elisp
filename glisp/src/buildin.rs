/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate cairo;
extern crate elisp;

use super::fractal::dragon::Dragon;
use super::fractal::hilvert::Hilvert;
use super::fractal::koch::Koch;
use super::fractal::sierpinski::Sierpinski;
use super::fractal::tree::Tree;
use super::fractal::Fractal;
use super::fractal::FractalMut;
use crate::draw::create_draw_arc;
use crate::draw::create_draw_image;
use crate::draw::create_draw_line;
use crate::draw::create_draw_string;
use crate::draw::draw_clear;
use crate::draw::ImageTable;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;

use lisp::Environment;
use lisp::Expression;
use lisp::RsError;

use cairo::ImageSurface;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub fn build_lisp_function(env: &Environment, image_table: &ImageTable) {
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    {
        let image_table = image_table.clone();
        env.add_builtin_ext_func("draw-clear", move |exp, _| {
            if exp.len() != 1 {
                return Err(create_error!("E1007"));
            }
            draw_clear(&image_table);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // DrawLine
    // ex. (draw-line 0.0 0.0 1.0 1.0)
    //--------------------------------------------------------
    let draw_line = create_draw_line(image_table);
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
        draw_line(loc[0], loc[1], loc[2], loc[3]);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // Create Image from png
    // ex. (create-image-from-png "roger" "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png")
    //--------------------------------------------------------
    {
        let image_table = image_table.clone();
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
            (*image_table).borrow_mut().regist(symbol, Rc::new(surface));
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // Draw Image (draw-image image xorg yorg x0 y0 x1 y1)
    // ex. (draw-image "roger" 0.0 0.0 0.25 0.0 0.0 0.32142857142857145)
    //--------------------------------------------------------
    let draw_image = create_draw_image(image_table);
    {
        let image_table = image_table.clone();
        env.add_builtin_ext_func("draw-image", move |exp, env| {
            if exp.len() != 8 {
                return Err(create_error!("E1007"));
            }
            let symbol = match lisp::eval(&exp[1], env)? {
                Expression::String(s) => s,
                _ => return Err(create_error!("E1015")),
            };
            const N: usize = 6;
            let mut ctm: [f64; N] = [0.0; N];
            let mut iter = exp[2 as usize..].iter();
            for i in 0..N {
                if let Some(e) = iter.next() {
                    if let Expression::Float(f) = lisp::eval(e, env)? {
                        ctm[i] = f;
                    } else {
                        return Err(create_error!("E1003"));
                    }
                } else {
                    return Err(create_error!("E1007"));
                }
            }
            let img = match (*image_table).borrow().find(&symbol) {
                Some(v) => v.clone(),
                None => return Err(create_error!("E1008")),
            };
            draw_image(ctm[2], ctm[3], ctm[4], ctm[5], ctm[0], ctm[1], &img);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // draw string
    // ex. (draw-string 0.04 0.50 0.15 "日本語")
    //--------------------------------------------------------
    let draw_string = create_draw_string(image_table);
    env.add_builtin_ext_func("draw-string", move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error!("E1007"));
        }
        const N: usize = 3;
        let mut prm: [f64; N] = [0.0; N];
        for (i, e) in exp[1 as usize..4 as usize].iter().enumerate() {
            prm[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!("E1003")),
            };
        }
        let s = match lisp::eval(&exp[4], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        draw_string(prm[0], prm[1], prm[2], s);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // draw arc
    // ex. (draw-arc 0.27 0.65 0.02 0.0)
    //--------------------------------------------------------
    let draw_arc = create_draw_arc(image_table);
    env.add_builtin_ext_func("draw-arc", move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error!("E1007"));
        }
        const N: usize = 4;
        let mut prm: [f64; N] = [0.0; N];
        for (i, e) in exp[1 as usize..].iter().enumerate() {
            prm[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!("E1003")),
            };
        }
        draw_arc(prm[0], prm[1], prm[2], prm[3]);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // set background
    // ex. (set-background 0.0 0.0 0.0)
    //--------------------------------------------------------
    {
        let image_table = image_table.clone();
        env.add_builtin_ext_func("set-background", move |exp, env| {
            let (r, g, b) = get_color(exp, env)?;
            let mut image_table = image_table.borrow_mut();
            image_table.set_background(r, g, b);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // set foreground
    // ex. (set-foreground 0.0 1.0 0.0)
    //--------------------------------------------------------
    {
        let image_table = image_table.clone();
        env.add_builtin_ext_func("set-foreground", move |exp, env| {
            let (r, g, b) = get_color(exp, env)?;
            let mut image_table = image_table.borrow_mut();
            image_table.set_foreground(r, g, b);
            Ok(Expression::Nil())
        });
    }
}
fn get_color(exp: &[Expression], env: &Environment) -> Result<(f64, f64, f64), RsError> {
    if exp.len() != 4 {
        return Err(create_error!("E1007"));
    }
    const N: usize = 3;
    let mut rgb: [f64; N] = [0.0; N];
    for (i, e) in exp[1 as usize..].iter().enumerate() {
        rgb[i] = match lisp::eval(e, env)? {
            Expression::Float(f) => f,
            _ => return Err(create_error!("E1003")),
        };
    }
    Ok((rgb[0], rgb[1], rgb[2]))
}
pub fn build_demo_function(env: &Environment, image_table: &ImageTable) {
    // ----------------------------------------------------------------
    // create new lisp interface
    // ----------------------------------------------------------------
    fn make_lisp_function(fractal: Box<dyn Fractal>, env: &Environment) {
        env.add_builtin_ext_func(fractal.get_func_name(), move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!("E1007"));
            }
            let c = match lisp::eval(&exp[1], env)? {
                Expression::Integer(c) => c,
                _ => return Err(create_error!("E1002")),
            };

            fractal.do_demo(c as i32);
            Ok(Expression::Nil())
        });
    }
    fn make_lisp_function_mut<T>(fractal: T, env: &Environment)
    where
        T: FractalMut + 'static,
    {
        let f = fractal.get_func_name();
        let fractal = RefCell::new(fractal);
        env.add_builtin_ext_func(f, move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!("E1007"));
            }
            let c = match lisp::eval(&exp[1], env)? {
                Expression::Integer(c) => c,
                _ => return Err(create_error!("E1002")),
            };
            fractal.borrow_mut().do_demo(c as i32);
            Ok(Expression::Nil())
        });
    }
    // ----------------------------------------------------------------
    // create each demo program
    // ----------------------------------------------------------------
    make_lisp_function(Box::new(Koch::new(create_draw_line(image_table))), env);
    make_lisp_function(Box::new(Tree::new(create_draw_line(image_table))), env);
    make_lisp_function(
        Box::new(Sierpinski::new(create_draw_line(image_table))),
        env,
    );
    make_lisp_function(Box::new(Dragon::new(create_draw_line(image_table))), env);
    make_lisp_function_mut(Hilvert::new(create_draw_line(image_table)), env);
}
