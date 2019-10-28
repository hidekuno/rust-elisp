/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rand::Rng;
use std::vec::Vec;

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

const SAMPLE_INT: i64 = 10_000_000_000_000;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("sqrt", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sqrt()))
    });
    b.regist("sin", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sin()))
    });
    b.regist("cos", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.cos()))
    });
    b.regist("tan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.tan()))
    });
    b.regist("asin", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.asin()))
    });
    b.regist("acos", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.acos()))
    });
    b.regist("atan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.atan()))
    });
    b.regist("exp", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.exp()))
    });
    b.regist("log", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.log((1.0 as f64).exp())))
    });
    b.regist("truncate", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.trunc()))
    });
    b.regist("floor", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.floor()))
    });
    b.regist("ceiling", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.ceil()))
    });
    b.regist("round", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.round()))
    });
    b.regist("abs", abs);

    b.regist("rand-integer", rand_integer);
    b.regist("rand-list", rand_list);
}
fn to_f64(exp: &[Expression], env: &Environment) -> Result<f64, RsError> {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(f) => Ok(f),
        Expression::Integer(i) => Ok(i as f64),
        Expression::Rational(r) => Ok(r.div_float()),
        _ => Err(create_error!(RsCode::E1003)),
    }
}
fn abs(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    Ok(match eval(&exp[1], env)? {
        Expression::Float(v) => Expression::Float(v.abs()),
        Expression::Integer(v) => Expression::Integer(v.abs()),
        Expression::Rational(v) => Expression::Rational(v.abs()),
        _ => return Err(create_error!(RsCode::E1003)),
    })
}
fn rand_integer(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if 1 < exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    Ok(Expression::Integer(x.abs() / SAMPLE_INT))
}
fn rand_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::Integer(i) = eval(&exp[1], env)? {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::new();
        for _ in 0..i {
            let x: i64 = rng.gen();
            vec.push(Expression::Integer(x.abs() / SAMPLE_INT));
        }
        Ok(Expression::List(vec))
    } else {
        Err(create_error!(RsCode::E1002))
    }
}
