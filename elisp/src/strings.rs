/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::create_error;
use crate::create_error_value;
use std::vec::Vec;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Expression, ResultExpression};
use crate::lisp::{RsCode, RsError};
use crate::number::Rat;

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("format", format_f);

    b.regist("string=?", |exp, env| strcmp(exp, env, |x, y| x == y));
    b.regist("string<?", |exp, env| strcmp(exp, env, |x, y| x < y));
    b.regist("string>?", |exp, env| strcmp(exp, env, |x, y| x > y));
    b.regist("string<=?", |exp, env| strcmp(exp, env, |x, y| x <= y));
    b.regist("string>=?", |exp, env| strcmp(exp, env, |x, y| x >= y));

    b.regist("string-ci=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() == y.to_lowercase())
    });
    b.regist("string-ci<?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() < y.to_lowercase())
    });
    b.regist("string-ci>?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() > y.to_lowercase())
    });
    b.regist("string-ci<=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() <= y.to_lowercase())
    });
    b.regist("string-ci>=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() >= y.to_lowercase())
    });

    b.regist("string-append", str_append);
    b.regist("string-length", |exp, env| {
        str_length(exp, env, |s| s.chars().count())
    });
    b.regist("string-size", |exp, env| str_length(exp, env, |s| s.len()));
    b.regist("number->string", number_string);
    b.regist("string->number", string_number);
    b.regist("list->string", list_string);
    b.regist("string->list", string_list);

    b.regist("substring", substring);
    b.regist("symbol->string", symbol_string);
    b.regist("string->symbol", string_symbol);
    b.regist("make-string", make_string);
}
fn format_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let s = if let Expression::String(s) = eval(&exp[1], env)? {
        s
    } else {
        return Err(create_error!(RsCode::E1015));
    };
    let i = if let Expression::Integer(i) = eval(&exp[2], env)? {
        i
    } else {
        return Err(create_error!(RsCode::E1002));
    };
    let s = match s.as_str() {
        "~X" => format!("{:X}", i),
        "~x" => format!("{:x}", i),
        n => match n.to_lowercase().as_str() {
            "~d" => format!("{:?}", i),
            "~o" => format!("{:o}", i),
            "~b" => format!("{:b}", i),
            _ => return Err(create_error!(RsCode::E1018)),
        },
    };
    Ok(Expression::String(s))
}
fn strcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &String, y: &String) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut v = Vec::new();
    for e in &exp[1 as usize..] {
        let s = match eval(e, env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(RsCode::E1015)),
        };
        v.push(s);
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn str_append(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 > exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut v = String::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::String(s) => v.push_str(&s.into_boxed_str()),
            _ => return Err(create_error!(RsCode::E1015)),
        };
    }
    Ok(Expression::String(v))
}
fn str_length(
    exp: &[Expression],
    env: &Environment,
    f: fn(s: String) -> usize,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Integer(f(s) as i64)),
        _ => return Err(create_error!(RsCode::E1015)),
    }
}
fn number_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Float(f) => Expression::Float(f),
        Expression::Integer(i) => Expression::Integer(i),
        Expression::Rational(r) => Expression::Rational(r),
        _ => return Err(create_error!(RsCode::E1003)),
    };
    Ok(Expression::String(v.to_string()))
}
fn string_number(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(RsCode::E1015)),
    };
    let v = if let Ok(n) = s.parse::<i64>() {
        Expression::Integer(n)
    } else if let Ok(n) = s.parse::<f64>() {
        Expression::Float(n)
    } else {
        match Rat::from(&s) {
            Ok(n) => Expression::Rational(n),
            Err(n) => {
                return if n.code != RsCode::E1020 {
                    return Err(create_error!(n.code));
                } else {
                    Err(create_error!(RsCode::E1003))
                }
            }
        }
    };
    Ok(v)
}
fn list_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!(RsCode::E1005)),
    };
    let mut v = String::new();

    for e in l.into_iter() {
        v.push(match eval(&e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!(RsCode::E1019)),
        });
    }
    Ok(Expression::String(v))
}
fn string_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(RsCode::E1015)),
    };
    let mut l: Vec<Expression> = Vec::new();
    for c in s.as_str().chars() {
        l.push(Expression::Char(c));
    }
    Ok(Expression::List(l))
}
fn substring(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 4 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(RsCode::E1015)),
    };
    let mut param: [usize; 2] = [0; 2];
    for (i, e) in exp[2..].iter().enumerate() {
        let v = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(RsCode::E1002)),
        };
        if 0 > v {
            return Err(create_error!(RsCode::E1021));
        }
        param[i] = v as usize;
    }
    let (start, end) = (param[0], param[1]);
    if s.chars().count() < end {
        return Err(create_error!(RsCode::E1021));
    }
    if start > end {
        return Err(create_error!(RsCode::E1021));
    }
    // the trait `std::convert::From<str>` is not implemented for `std::string::String`
    // s.as_str()[start..end].to_string()),
    //     => panicked at 'byte index 1 is not a char boundary; it is inside '山' (bytes 0..3)
    let mut v = String::new();
    for c in &s.chars().collect::<Vec<char>>()[start..end] {
        v.push(*c);
    }
    Ok(Expression::String(v))
}
fn symbol_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Symbol(s) => Ok(Expression::String(s)),
        _ => return Err(create_error!(RsCode::E1004)),
    }
}
fn string_symbol(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Symbol(s)),
        _ => Err(create_error!(RsCode::E1015)),
    }
}
fn make_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let n = match eval(&exp[1], env)? {
        Expression::Integer(n) => n,
        _ => return Err(create_error!(RsCode::E1002)),
    };
    if n < 0 {
        return Err(create_error!(RsCode::E1021));
    }
    let c = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(RsCode::E1019)),
    };

    let mut s = String::new();
    for _ in 0..n {
        s.push(c);
    }
    Ok(Expression::String(s))
}
