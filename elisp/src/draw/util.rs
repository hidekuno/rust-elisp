/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   this is library for glis,wasmlisp.

   hidekuno@gmail.com
*/
use crate::create_error;
use crate::draw::DrawArc;
use crate::draw::DrawImage;
use crate::draw::DrawLine;
use crate::draw::Fractal;
use crate::lisp::eval;
use crate::lisp::Environment;
use crate::lisp::ErrCode;
use crate::lisp::Error;
use crate::lisp::Expression;

// ----------------------------------------------------------------
// set up for draw_line
// ----------------------------------------------------------------
pub fn regist_draw_line(fname: &'static str, env: &Environment, draw_line: DrawLine) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 5 && exp.len() != 3 {
            return Err(create_error!(ErrCode::E1007));
        }
        const N: usize = 4;
        let mut loc: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut loc, (1, N))?;
        draw_line(loc[0], loc[1], loc[2], loc[3]);
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// set up for draw_image
// ----------------------------------------------------------------
pub fn regist_draw_image(fname: &'static str, env: &Environment, draw_image: DrawImage) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 8 && exp.len() != 5 {
            return Err(create_error!(ErrCode::E1007));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(ErrCode::E1015)),
        };
        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut ctm, (2, N))?;
        draw_image(ctm[2], ctm[3], ctm[4], ctm[5], ctm[0], ctm[1], &symbol)?;
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// set up for draw_arc
// ----------------------------------------------------------------
pub fn regist_draw_arc(fname: &'static str, env: &Environment, draw_arc: DrawArc) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error!(ErrCode::E1007));
        }
        const N: usize = 4;
        let mut prm: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut prm, (1, N))?;
        draw_arc(prm[0], prm[1], prm[2], prm[3]);
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// thisi is function for draw-line, draw-image
// ----------------------------------------------------------------
fn set_loc(
    exp: &[Expression],
    env: &Environment,
    loc: &mut [f64],
    param: (usize, usize),
) -> Result<(), Error> {
    let mut iter = exp[param.0 as usize..].iter();

    if exp.len() == (param.1 + param.0) {
        for i in 0..param.1 {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = eval(&e, env)? {
                    loc[i] = f;
                } else {
                    return Err(create_error!(ErrCode::E1003));
                }
            }
        }
    } else if exp.len() == (param.1 / 2 + param.0) {
        for i in (0..param.1).step_by(2) {
            if let Some(e) = iter.next() {
                if let Expression::Pair(x, y) = eval(&e, env)? {
                    if let Expression::Float(f) = eval(&x, env)? {
                        loc[i] = f;
                    } else {
                        return Err(create_error!(ErrCode::E1003));
                    }
                    if let Expression::Float(f) = eval(&y, env)? {
                        loc[i + 1] = f;
                    } else {
                        return Err(create_error!(ErrCode::E1003));
                    }
                } else {
                    return Err(create_error!(ErrCode::E1005));
                }
            }
        }
    }
    Ok(())
}
// ----------------------------------------------------------------
// create new lisp interface
// ----------------------------------------------------------------
pub fn make_lisp_function(fractal: Box<dyn Fractal>, env: &Environment) {
    env.add_builtin_ext_func(fractal.get_func_name(), move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error!(ErrCode::E1007));
        }
        let c = match eval(&exp[1], env)? {
            Expression::Integer(c) => c,
            _ => return Err(create_error!(ErrCode::E1002)),
        };
        fractal.do_demo(c as i32);
        Ok(Expression::Nil())
    });
}
