/*
 Rust study program.
 This is prototype program mini scheme subset what porting from go-scheme.

 hidekuno@gmail.com
*/
use crate::create_error;
use crate::lisp::eval;
use crate::lisp::Environment;
use crate::lisp::ErrCode;
use crate::lisp::Error;
use crate::lisp::Expression;

pub fn regist_draw_line(
    fname: &'static str,
    env: &Environment,
    draw_line: Box<dyn Fn(f64, f64, f64, f64) + 'static>,
) {
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
pub fn set_loc(
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
