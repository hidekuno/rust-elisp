/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::char;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Expression, ResultExpression};
use crate::lisp::{RsCode, RsError};

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("char=?", |exp, env| charcmp(exp, env, |x, y| x == y));
    b.regist("char<?", |exp, env| charcmp(exp, env, |x, y| x < y));
    b.regist("char>?", |exp, env| charcmp(exp, env, |x, y| x > y));
    b.regist("char<=?", |exp, env| charcmp(exp, env, |x, y| x <= y));
    b.regist("char>=?", |exp, env| charcmp(exp, env, |x, y| x >= y));

    b.regist("integer->char", integer_char);
    b.regist("char->integer", char_integer);
}
fn charcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: char, y: char) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut v: [char; 2] = [' '; 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!(RsCode::E1019)),
        }
    }
    Ok(Expression::Boolean(f(v[0], v[1])))
}
fn integer_char(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let i = match eval(&exp[1], env)? {
        Expression::Integer(i) => i,
        _ => return Err(create_error!(RsCode::E1002)),
    };
    let i = i as u32;
    if let Some(c) = char::from_u32(i) {
        Ok(Expression::Char(c))
    } else {
        Err(create_error!(RsCode::E1019))
    }
}
fn char_integer(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let c = match eval(&exp[1], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(RsCode::E1019)),
    };
    let a = c as u32;
    Ok(Expression::Integer(a as i64))
}
