/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate cairo;
extern crate elisp;
extern crate gtk;

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
use crate::draw::DrawTable;
use crate::ui::DRAW_HEIGHT;
use crate::ui::DRAW_WIDTH;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;

use lisp::Environment;
use lisp::Expression;
use lisp::RsCode;
use lisp::RsError;

use cairo::ImageSurface;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub fn build_lisp_function(env: &Environment, draw_table: &DrawTable) {
    //--------------------------------------------------------
    // Draw Clear
    //--------------------------------------------------------
    {
        let draw_table = draw_table.clone();
        env.add_builtin_ext_func("draw-clear", move |exp, _| {
            if exp.len() != 1 {
                return Err(create_error!(RsCode::E1007));
            }
            draw_clear(&draw_table);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // Draw Line
    // ex. (draw-line 0.0 0.0 1.0 1.0)
    //--------------------------------------------------------
    let draw_line = create_draw_line(draw_table, 1);
    env.add_builtin_ext_func("draw-line", move |exp, env| {
        const N: usize = 4;
        if exp.len() != (N + 1) {
            return Err(create_error!(RsCode::E1007));
        }
        let mut loc: [f64; N] = [0.0; N];
        let mut iter = exp[1 as usize..].iter();
        for i in 0..N {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = lisp::eval(e, env)? {
                    loc[i] = f;
                } else {
                    return Err(create_error!(RsCode::E1003));
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
        let draw_table = draw_table.clone();
        env.add_builtin_ext_func("create-image-from-png", move |exp, env| {
            if exp.len() != 3 {
                return Err(create_error!(RsCode::E1007));
            }
            let symbol = match lisp::eval(&exp[1], env)? {
                Expression::String(s) => s,
                _ => return Err(create_error!(RsCode::E1015)),
            };
            let filename = match lisp::eval(&exp[2], env)? {
                Expression::String(s) => s,
                _ => return Err(create_error!(RsCode::E1015)),
            };
            let mut file = match File::open(filename) {
                Ok(f) => f,
                Err(e) => return Err(create_error_value!(RsCode::E9999, e)),
            };
            let surface = match ImageSurface::create_from_png(&mut file) {
                Ok(s) => s,
                Err(e) => return Err(create_error_value!(RsCode::E9999, e)),
            };
            draw_table.regist(symbol, Rc::new(surface));
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // Draw Image (draw-image image xorg yorg x0 y0 x1 y1)
    // ex. (draw-image "roger" 0.0 0.0 0.25 0.0 0.0 0.32142857142857145)
    //--------------------------------------------------------
    let draw_image = create_draw_image(draw_table);
    {
        let draw_table = draw_table.clone();
        env.add_builtin_ext_func("draw-image", move |exp, env| {
            if exp.len() != 8 {
                return Err(create_error!(RsCode::E1007));
            }
            let symbol = match lisp::eval(&exp[1], env)? {
                Expression::String(s) => s,
                _ => return Err(create_error!(RsCode::E1015)),
            };
            const N: usize = 6;
            let mut ctm: [f64; N] = [0.0; N];
            let mut iter = exp[2 as usize..].iter();
            for i in 0..N {
                if let Some(e) = iter.next() {
                    if let Expression::Float(f) = lisp::eval(e, env)? {
                        ctm[i] = f;
                    } else {
                        return Err(create_error!(RsCode::E1003));
                    }
                } else {
                    return Err(create_error!(RsCode::E1007));
                }
            }
            let img = match draw_table.find(&symbol) {
                Some(v) => v.clone(),
                None => return Err(create_error!(RsCode::E1008)),
            };
            draw_image(ctm[2], ctm[3], ctm[4], ctm[5], ctm[0], ctm[1], &img);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // get image width (image-width image)
    // ex. (image-width "roger")
    //--------------------------------------------------------
    {
        let draw_table = draw_table.clone();
        env.add_builtin_ext_func("image-width", move |exp, env| {
            let f = image_size(exp, env, &draw_table, |img| img.get_width())?;
            Ok(Expression::Float(f))
        });
    }
    //--------------------------------------------------------
    // get image width (image-width image)
    // ex. (image-width "roger")
    //--------------------------------------------------------
    {
        let draw_table = draw_table.clone();
        env.add_builtin_ext_func("image-height", move |exp, env| {
            let f = image_size(exp, env, &draw_table, |img| img.get_height())?;
            Ok(Expression::Float(f))
        });
    }
    //--------------------------------------------------------
    // draw string
    // ex. (draw-string 0.04 0.50 0.15 "日本語")
    //--------------------------------------------------------
    let draw_string = create_draw_string(draw_table);
    env.add_builtin_ext_func("draw-string", move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error!(RsCode::E1007));
        }
        const N: usize = 3;
        let mut prm: [f64; N] = [0.0; N];
        for (i, e) in exp[1 as usize..4 as usize].iter().enumerate() {
            prm[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!(RsCode::E1003)),
            };
        }
        let s = match lisp::eval(&exp[4], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        draw_string(prm[0], prm[1], prm[2], s);
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // draw eval
    // ex. (draw-eval (iota 10))
    //--------------------------------------------------------
    let draw_string = create_draw_string(draw_table);
    env.add_builtin_ext_func("draw-eval", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error!(RsCode::E1007));
        }
        let e = lisp::eval(&exp[1], env)?;

        let mut h = 0.04;
        let mut s = String::new();
        let mut j = 0;

        for (i, c) in e
            .to_string()
            .chars()
            .collect::<Vec<char>>()
            .iter()
            .enumerate()
        {
            if i > 0 && i % 54 == 0 {
                draw_string(0.02, h, 0.03, s.to_string());
                h += 0.04;
                s.clear();
                j += 1;
            }
            if j == 22 {
                draw_string(0.02, h, 0.03, "...".to_string());
                break;
            }
            s.push(*c);
        }
        if j != 22 {
            draw_string(0.02, h, 0.03, s.to_string());
        }
        Ok(Expression::Nil())
    });
    //--------------------------------------------------------
    // draw arc
    // ex. (draw-arc 0.27 0.65 0.02 0.0)
    //--------------------------------------------------------
    let draw_arc = create_draw_arc(draw_table);
    env.add_builtin_ext_func("draw-arc", move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error!(RsCode::E1007));
        }
        const N: usize = 4;
        let mut prm: [f64; N] = [0.0; N];
        for (i, e) in exp[1 as usize..].iter().enumerate() {
            prm[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!(RsCode::E1003)),
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
        let draw_table = RefCell::new(draw_table.clone());
        env.add_builtin_ext_func("set-background", move |exp, env| {
            let (r, g, b) = get_color(exp, env)?;
            draw_table.borrow_mut().set_background(r, g, b);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // set foreground
    // ex. (set-foreground 0.0 1.0 0.0)
    //--------------------------------------------------------
    {
        let draw_table = RefCell::new(draw_table.clone());
        env.add_builtin_ext_func("set-foreground", move |exp, env| {
            let (r, g, b) = get_color(exp, env)?;
            draw_table.borrow_mut().set_foreground(r, g, b);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // set line width
    // ex. (set-line-width 0.001)
    //--------------------------------------------------------
    {
        let draw_table = RefCell::new(draw_table.clone());
        env.add_builtin_ext_func("set-line-width", move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!(RsCode::E1007));
            }
            let w = match lisp::eval(&exp[1], env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!(RsCode::E1003)),
            };
            draw_table.borrow_mut().set_line_width(w);
            Ok(Expression::Nil())
        });
    }
    //--------------------------------------------------------
    // ex. (screen-width)
    //--------------------------------------------------------
    {
        env.add_builtin_ext_func("screen-width", move |exp, _env| {
            if exp.len() != 1 {
                return Err(create_error!(RsCode::E1007));
            }
            Ok(Expression::Float(DRAW_WIDTH as f64))
        });
    }
    //--------------------------------------------------------
    // ex. (screen-height)
    //--------------------------------------------------------
    {
        env.add_builtin_ext_func("screen-height", move |exp, _env| {
            if exp.len() != 1 {
                return Err(create_error!(RsCode::E1007));
            }
            Ok(Expression::Float(DRAW_HEIGHT as f64))
        });
    }
    //--------------------------------------------------------
    // ex. (gtk-major-version)
    // ex. (gtk-minor-version)
    // ex. (gtk-micro-version)
    //--------------------------------------------------------
    {
        let version_tbl = [
            ("gtk-major-version", gtk::get_major_version()),
            ("gtk-minor-version", gtk::get_minor_version()),
            ("gtk-micro-version", gtk::get_micro_version()),
        ];
        for (f, v) in version_tbl.into_iter() {
            let x = *v;
            env.add_builtin_ext_func(f, move |exp, _env| {
                if exp.len() != 1 {
                    return Err(create_error!(RsCode::E1007));
                }
                Ok(Expression::Integer(x as i64))
            });
        }
    }
    fn get_color(exp: &[Expression], env: &Environment) -> Result<(f64, f64, f64), RsError> {
        if exp.len() != 4 {
            return Err(create_error!(RsCode::E1007));
        }
        const N: usize = 3;
        let mut rgb: [f64; N] = [0.0; N];
        for (i, e) in exp[1 as usize..].iter().enumerate() {
            rgb[i] = match lisp::eval(e, env)? {
                Expression::Float(f) => f,
                _ => return Err(create_error!(RsCode::E1003)),
            };
        }
        Ok((rgb[0], rgb[1], rgb[2]))
    }
    fn image_size(
        exp: &[Expression],
        env: &Environment,
        draw_table: &DrawTable,
        f: fn(&ImageSurface) -> i32,
    ) -> Result<f64, RsError> {
        if exp.len() != 2 {
            return Err(create_error!(RsCode::E1007));
        }
        let symbol = match lisp::eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        let img = match (*draw_table).find(&symbol) {
            Some(v) => v.clone(),
            None => return Err(create_error!(RsCode::E1008)),
        };
        Ok(f(&img) as f64)
    }
}
pub fn build_demo_function(env: &Environment, draw_table: &DrawTable) {
    // ----------------------------------------------------------------
    // create new lisp interface
    // ----------------------------------------------------------------
    fn make_lisp_function(fractal: Box<dyn Fractal>, env: &Environment) {
        env.add_builtin_ext_func(fractal.get_func_name(), move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!(RsCode::E1007));
            }
            let c = match lisp::eval(&exp[1], env)? {
                Expression::Integer(c) => c,
                _ => return Err(create_error!(RsCode::E1002)),
            };

            fractal.do_demo(c as i32);
            Ok(Expression::Nil())
        });
    }
    // ----------------------------------------------------------------
    // create new lisp interface (mutable)
    // ----------------------------------------------------------------
    fn make_lisp_function_mut<T>(fractal: T, env: &Environment)
    where
        T: FractalMut + 'static,
    {
        let f = fractal.get_func_name();
        let fractal = RefCell::new(fractal);
        env.add_builtin_ext_func(f, move |exp, env| {
            if exp.len() != 2 {
                return Err(create_error!(RsCode::E1007));
            }
            let c = match lisp::eval(&exp[1], env)? {
                Expression::Integer(c) => c,
                _ => return Err(create_error!(RsCode::E1002)),
            };
            fractal.borrow_mut().do_demo(c as i32);
            Ok(Expression::Nil())
        });
    }
    // ----------------------------------------------------------------
    // create each demo program
    // ----------------------------------------------------------------
    const N: usize = 10000;
    make_lisp_function(Box::new(Koch::new(create_draw_line(draw_table, N))), env);
    make_lisp_function(Box::new(Tree::new(create_draw_line(draw_table, N))), env);
    make_lisp_function(
        Box::new(Sierpinski::new(create_draw_line(draw_table, N))),
        env,
    );
    make_lisp_function(Box::new(Dragon::new(create_draw_line(draw_table, N))), env);
    make_lisp_function_mut(Hilvert::new(create_draw_line(draw_table, N)), env);
}
