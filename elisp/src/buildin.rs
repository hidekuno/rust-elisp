/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rand::Rng;
use std::char;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;

use crate::lisp::{eval, repl};
use crate::lisp::{Expression, Operation, ResultExpression};
use crate::lisp::{RsError, RsFunction};

use crate::number::Number;
use crate::number::Rat;

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;

//========================================================================
const SAMPLE_INT: i64 = 10_000_000_000_000;
//========================================================================
pub trait BuildInTable {
    fn regist(&mut self, symbol: &'static str, func: Operation);
}
pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("+", |exp, env| calc(exp, env, |x, y| x + y));
    b.regist("-", |exp, env| calc(exp, env, |x, y| x - y));
    b.regist("*", |exp, env| calc(exp, env, |x, y| x * y));
    b.regist("/", |exp, env| calc(exp, env, |x, y| x / y));
    b.regist("max", |exp, env| {
        calc(exp, env, |x, y| if x > y { x } else { y })
    });
    b.regist("min", |exp, env| {
        calc(exp, env, |x, y| if x < y { x } else { y })
    });
    b.regist("=", |exp, env| cmp(exp, env, |x, y| x == y));
    b.regist("<", |exp, env| cmp(exp, env, |x, y| x < y));
    b.regist("<=", |exp, env| cmp(exp, env, |x, y| x <= y));
    b.regist(">", |exp, env| cmp(exp, env, |x, y| x > y));
    b.regist(">=", |exp, env| cmp(exp, env, |x, y| x >= y));

    b.regist("ash", shift);
    b.regist("logand", |exp, env| bit(exp, env, |x, y| x & y));
    b.regist("logior", |exp, env| bit(exp, env, |x, y| x | y));
    b.regist("logxor", |exp, env| bit(exp, env, |x, y| x ^ y));
    b.regist("lognot", lognot);

    b.regist("even?", |exp, env| odd_even(exp, env, |x| x % 2 == 0));
    b.regist("odd?", |exp, env| odd_even(exp, env, |x| x % 2 != 0));
    b.regist("zero?", |exp, env| {
        is_sign(exp, env, |x| x == &Number::Integer(0))
    });
    b.regist("positive?", |exp, env| {
        is_sign(exp, env, |x| x > &Number::Integer(0))
    });
    b.regist("negative?", |exp, env| {
        is_sign(exp, env, |x| x < &Number::Integer(0))
    });

    b.regist("list?", |exp, env| is_type(exp, env, Expression::is_list));
    b.regist("pair?", |exp, env| is_type(exp, env, Expression::is_pair));
    b.regist("char?", |exp, env| is_type(exp, env, Expression::is_char));
    b.regist("string?", |exp, env| {
        is_type(exp, env, Expression::is_string)
    });
    b.regist("procedure?", |exp, env| {
        is_type(exp, env, Expression::is_procedure)
    });
    b.regist("integer?", |exp, env| {
        is_type(exp, env, Expression::is_integer)
    });
    b.regist("number?", |exp, env| {
        is_type(exp, env, Expression::is_number)
    });
    b.regist("expt", expt);
    b.regist("modulo", |exp, env| divide(exp, env, |x, y| x % y));
    b.regist("quotient", |exp, env| divide(exp, env, |x, y| x / y));
    b.regist("define", define);
    b.regist("lambda", lambda);
    b.regist("if", if_f);
    b.regist("and", and);
    b.regist("or", or);
    b.regist("not", not);
    b.regist("let", let_f);
    b.regist("time", time_f);
    b.regist("set!", set_f);
    b.regist("cond", cond);
    b.regist("eq?", eqv);
    b.regist("eqv?", eqv);
    b.regist("case", case);
    b.regist("apply", apply);
    b.regist("identity", identity);

    b.regist("list", list);
    b.regist("make-list", make_list);
    b.regist("null?", null_f);
    b.regist("length", length);
    b.regist("car", car);
    b.regist("cdr", cdr);
    b.regist("cadr", cadr);
    b.regist("cons", cons);
    b.regist("append", append);
    b.regist("take", |exp, env| take_drop(exp, env, |l, n| &l[0..n]));
    b.regist("drop", |exp, env| take_drop(exp, env, |l, n| &l[n..]));
    b.regist("delete", delete);
    b.regist("last", last);
    b.regist("reverse", reverse);
    b.regist("iota", iota);
    b.regist("map", map);
    b.regist("filter", filter);
    b.regist("reduce", reduce);
    b.regist("for-each", for_each);
    b.regist("list-ref", list_ref);

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

    b.regist("load-file", load_file);
    b.regist("display", display);
    b.regist("newline", newline);
    b.regist("begin", begin);

    b.regist("delay", delay);
    b.regist("force", force);
    b.regist("format", format_f);

    b.regist("string=?", |exp, env| strcmp(exp, env, |x, y| x == y));
    b.regist("string<?", |exp, env| strcmp(exp, env, |x, y| x < y));
    b.regist("string>?", |exp, env| strcmp(exp, env, |x, y| x > y));
    b.regist("string<=?", |exp, env| strcmp(exp, env, |x, y| x <= y));
    b.regist("string>=?", |exp, env| strcmp(exp, env, |x, y| x >= y));

    b.regist("char=?", |exp, env| charcmp(exp, env, |x, y| x == y));
    b.regist("char<?", |exp, env| charcmp(exp, env, |x, y| x < y));
    b.regist("char>?", |exp, env| charcmp(exp, env, |x, y| x > y));
    b.regist("char<=?", |exp, env| charcmp(exp, env, |x, y| x <= y));
    b.regist("char>=?", |exp, env| charcmp(exp, env, |x, y| x >= y));
    b.regist("string-append", str_append);
    b.regist("string-length", |exp, env| {
        str_length(exp, env, |s| s.chars().count())
    });
    b.regist("string-size", |exp, env| str_length(exp, env, |s| s.len()));
    b.regist("number->string", number_string);
    b.regist("string->number", string_number);
    b.regist("list->string", list_string);
    b.regist("string->list", string_list);
    b.regist("integer->char", integer_char);
    b.regist("char->integer", char_integer);

    b.regist("quote", |exp, _env| {
        if exp.len() != 2 {
            Err(create_error_value!("E1007", exp.len()))
        } else {
            Ok(exp[1].clone())
        }
    });
}
fn set_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Symbol(s) = &exp[1] {
        if let Some(_) = env.find(s) {
            let v = eval(&exp[2], env)?;
            env.update(s, v);
        } else {
            return Err(create_error_value!("E1008", s));
        }
        Ok(Expression::Symbol(s.to_string()))
    } else {
        Err(create_error!("E1004"))
    }
}
fn time_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }

    let start = Instant::now();
    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{:03}(s)", end.as_secs(), end.subsec_nanos() / 1_000_000);
    return result;
}
fn let_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    // @@@ env.create();
    let mut param = Environment::new_next(env);
    let mut idx = 1;
    let mut name = String::from("lambda");

    if let Expression::Symbol(s) = &exp[idx] {
        name = s.to_string();
        idx += 1;
    }
    // Parameter Setup
    let mut param_list: Vec<Expression> = Vec::new();
    let mut param_value_list: Vec<Expression> = Vec::new();
    param_value_list.push(Expression::String(String::from("dummy")));

    if let Expression::List(l) = &exp[idx] {
        for plist in l {
            if let Expression::List(p) = plist {
                if p.len() != 2 {
                    return Err(create_error_value!("E1007", p.len()));
                }
                if let Expression::Symbol(s) = &p[0] {
                    param_list.push(Expression::Symbol(s.clone()));
                    param_value_list.push(p[1].clone());
                } else {
                    return Err(create_error!("E1004"));
                }
            } else {
                return Err(create_error!("E1005"));
            }
        }
        idx += 1;
    } else {
        return Err(create_error!("E1005"));
    }

    // Setup Function
    let mut vec = Vec::new();
    vec.push(Expression::String(name.to_string()));
    vec.push(Expression::List(param_list));
    vec.extend_from_slice(&exp[idx as usize..]);
    let mut f = RsFunction::new(&vec[..], name.to_string(), param.clone());

    // Setup label name let
    if let Expression::Symbol(s) = &exp[1] {
        if env.is_tail_recursion() == true {
            f.set_tail_recurcieve();
            if f.get_tail_recurcieve() == false {
                param.regist(s.to_string(), Environment::create_func(f.clone()));
            }
        } else {
            param.regist(s.to_string(), Environment::create_func(f.clone()));
        }
    }
    f.execute(&param_value_list, &mut param)
}
fn not(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Boolean(b) => Ok(Expression::Boolean(!b)),
        _ => Err(create_error!("E1001")),
    }
}
fn or(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == true {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    Ok(Expression::Boolean(false))
}
fn and(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == false {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    Ok(Expression::Boolean(true))
}
fn expt(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float(x.powf(y))),
            Expression::Integer(y) => Ok(Expression::Float(x.powf(y as f64))),
            _ => Err(create_error!("E1003")),
        },
        Expression::Integer(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float((x as f64).powf(y))),
            Expression::Integer(y) => {
                if y >= 0 {
                    Ok(Expression::Integer(x.pow(y as u32)))
                } else {
                    Ok(Expression::Rational(Rat::new(1, x.pow(y.abs() as u32))))
                }
            }
            _ => Err(create_error!("E1003")),
        },
        _ => Err(create_error!("E1003")),
    }
}
fn divide(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &i64, y: &i64) -> i64,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);
    match (a, b) {
        (Expression::Integer(x), Expression::Integer(y)) => {
            if y == 0 {
                Err(create_error!("E1013"))
            } else {
                Ok(Expression::Integer(f(&x, &y)))
            }
        }
        (_, _) => Err(create_error!("E1002")),
    }
}
fn lambda(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = &exp[1] {
        for e in l {
            match e {
                Expression::Symbol(_) => {}
                _ => return Err(create_error!("E1004")),
            }
        }
    } else {
        return Err(create_error!("E1005"));
    }
    Ok(Environment::create_func(RsFunction::new(
        exp,
        String::from("lambda"),
        env.clone(),
    )))
}
fn define(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Symbol(v) = &exp[1] {
        if exp.len() != 3 {
            return Err(create_error_value!("E1007", exp.len()));
        }
        let se = eval(&exp[2], env)?;
        env.regist(v.to_string(), se);

        return Ok(Expression::Symbol(v.to_string()));
    }
    if let Expression::List(l) = &exp[1] {
        if l.len() < 1 {
            return Err(create_error_value!("E1007", l.len()));
        }
        if let Expression::Symbol(s) = &l[0] {
            let mut param: Vec<Expression> = Vec::new();
            for n in &l[1..] {
                match n {
                    Expression::Symbol(_) => {
                        param.push(n.clone());
                    }
                    _ => return Err(create_error!("E1004")),
                }
            }

            let mut f = exp.to_vec();
            f[1] = Expression::List(param);
            let mut func = RsFunction::new(&f, s.to_string(), env.clone());
            if env.is_tail_recursion() == true {
                func.set_tail_recurcieve();
            }
            env.regist(s.to_string(), Environment::create_func(func));

            Ok(Expression::Symbol(s.to_string()))
        } else {
            Err(create_error!("E1004"))
        }
    } else {
        Err(create_error!("E1004"))
    }
}
fn if_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Boolean(b) = eval(&exp[1], env)? {
        if b == true {
            eval(&exp[2], env)
        } else if 4 <= exp.len() {
            eval(&exp[3], env)
        } else {
            Ok(Expression::Nil())
        }
    } else {
        Err(create_error!("E1001"))
    }
}
fn cond(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::List(l) = e {
            let mut iter = l.iter();

            if let Some(e) = iter.next() {
                if let Expression::Symbol(s) = e {
                    if s != "else" {
                        eval(&e, env)?;
                    }
                } else {
                    let v = eval(&e, env)?;
                    if let Expression::Boolean(b) = v {
                        if b == false {
                            continue;
                        }
                        if l.len() == 1 {
                            return Ok(v);
                        }
                    }
                }
            } else {
                return Err(create_error!("E1012"));
            }
            return begin(&l, env);
        } else {
            return Err(create_error!("E1005"));
        }
    }
    Ok(Expression::Nil())
}
fn eqv(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);
    if let (Expression::Float(x), Expression::Float(y)) = (&a, &b) {
        return Ok(Expression::Boolean(*x == *y));
    }
    match a {
        Expression::Integer(x) => match b {
            Expression::Integer(y) => Ok(Expression::Boolean(x == y)),
            Expression::Rational(y) => Ok(Expression::Boolean(
                Number::Integer(x) == Number::Rational(y),
            )),
            _ => Ok(Expression::Boolean(false)),
        },
        Expression::Rational(x) => match b {
            Expression::Integer(y) => Ok(Expression::Boolean(
                Number::Rational(x) == Number::Integer(y),
            )),
            Expression::Rational(y) => Ok(Expression::Boolean(
                Number::Rational(x) == Number::Rational(y),
            )),
            _ => Ok(Expression::Boolean(false)),
        },
        _ => Ok(Expression::Boolean(false)),
    }
}
fn case(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut param: Vec<Expression> = Vec::new();
    param.push(Expression::Nil());
    param.push(eval(&exp[1], env)?);
    param.push(Expression::Nil());

    if 3 <= exp.len() {
        for e in &exp[2 as usize..] {
            if let Expression::List(l) = e {
                if l.len() == 0 {
                    continue;
                }
                match &l[0] {
                    Expression::Symbol(s) => {
                        if s != "else" {
                            return Err(create_error!("E1017"));
                        }
                        if 1 < l.len() {
                            return begin(&l, env);
                        }
                    }
                    Expression::List(c) => {
                        for e in c {
                            param[2] = eval(&e, env)?;
                            if let Expression::Boolean(b) = eqv(&param, env)? {
                                if b == true {
                                    return begin(&l, env);
                                }
                            }
                        }
                    }
                    _ => return Err(create_error!("E1017")),
                }
            } else {
                return Err(create_error!("E1005"));
            }
        }
    }
    Ok(Expression::Nil())
}
fn apply(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = eval(&exp[2], env)? {
        let mut se: Vec<Expression> = Vec::new();
        se.push(exp[1].clone());
        se.extend_from_slice(&l);
        eval(&Expression::List(se), env)
    } else {
        Err(create_error_value!("E1005", exp.len()))
    }
}
pub fn identity(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    eval(&exp[1], env)
}
fn list(exp: &[Expression], env: &Environment) -> ResultExpression {
    let mut list: Vec<Expression> = Vec::with_capacity(exp.len());
    for e in &exp[1 as usize..] {
        list.push(eval(e, env)?);
    }
    Ok(Expression::List(list))
}
fn make_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::Integer(v) => v,
        _ => return Err(create_error!("E1002")),
    };
    if l < 0 {
        return Err(create_error!("E1011"));
    }
    let v = match eval(&exp[2], env) {
        Ok(v) => v,
        Err(e) => return Err(create_error!(e.code)),
    };
    Ok(Expression::List(vec![v; l as usize]))
}
fn null_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => Ok(Expression::Boolean(l.len() == 0)),
        _ => Ok(Expression::Boolean(false)),
    }
}
fn length(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        Ok(Expression::Integer(l.len() as i64))
    } else {
        Err(create_error!("E1005"))
    }
}
fn car(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            if l.len() <= 0 {
                return Err(create_error!("E1011"));
            }
            Ok(l[0].clone())
        }
        Expression::Pair(car, _cdr) => Ok((*car).clone()),
        _ => Err(create_error!("E1005")),
    }
}
fn cdr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match l.len() {
            0 => Err(create_error!("E1011")),
            1 => Ok(Expression::List(Vec::new())),
            _ => Ok(Expression::List(l[1 as usize..].to_vec())),
        },
        Expression::Pair(_car, cdr) => Ok((*cdr).clone()),
        _ => Err(create_error!("E1005")),
    }
}
fn cadr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        if l.len() <= 1 {
            return Err(create_error!("E1011"));
        }
        Ok(l[1].clone())
    } else {
        Err(create_error!("E1005"))
    }
}
fn cons(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let car = eval(&exp[1], env)?;
    let cdr = eval(&exp[2], env)?;

    if let Expression::List(mut l) = cdr {
        let mut v: Vec<Expression> = Vec::new();
        v.push(car);
        v.append(&mut l);
        Ok(Expression::List(v))
    } else {
        Ok(Expression::Pair(Box::new(car), Box::new(cdr)))
    }
}
fn append(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() <= 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v: Vec<Expression> = Vec::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::List(mut l) => v.append(&mut l),
            _ => return Err(create_error!("E1005")),
        }
    }
    Ok(Expression::List(v))
}
fn take_drop(
    exp: &[Expression],
    env: &Environment,
    f: fn(l: &Vec<Expression>, n: usize) -> &[Expression],
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!("E1005")),
    };
    let n = match eval(&exp[2], env)? {
        Expression::Integer(n) => n,
        _ => return Err(create_error!("E1002")),
    };
    if l.len() < n as usize || n < 0 {
        return Err(create_error!("E1011"));
    }
    let mut vec = Vec::new();
    vec.extend_from_slice(f(&l, n as usize));

    Ok(Expression::List(vec))
}
fn delete(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let other = eval(&exp[1], env)?;
    let l = match eval(&exp[2], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!("E1005")),
    };
    let mut vec = Vec::new();
    for e in &l {
        if true == Expression::eq_integer(e, &other)
            || true == Expression::eq_float(e, &other)
            || true == Expression::eq_rat(e, &other)
            || true == Expression::eq_string(e, &other)
            || true == Expression::eq_char(e, &other)
            || true == Expression::eq_boolean(e, &other)
            || true == Expression::eq_symbol(e, &other)
        {
            continue;
        }
        vec.push(e.clone());
    }
    Ok(Expression::List(vec))
}
fn last(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match l.len() {
            0 => Err(create_error!("E1011")),
            _ => Ok(l[l.len() - 1].clone()),
        },
        Expression::Pair(car, _) => Ok(*car.clone()),
        _ => Err(create_error!("E1005")),
    }
}
fn reverse(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let mut v = l.clone();
            v.reverse();
            Ok(Expression::List(v))
        }
        _ => Err(create_error!("E1005")),
    }
}
fn iota(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() <= 1 || 4 <= exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut from = 0;
    let mut to = 0;

    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::Integer(i) => {
                if exp.len() == 3 {
                    from = i;
                }
                to += i;
            }
            _ => return Err(create_error!("E1002")),
        }
    }
    let mut l = Vec::with_capacity(to as usize);
    for v in from..to {
        l.push(Expression::Integer(v));
    }
    Ok(Expression::List(l))
}
fn map(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Function(f) => match eval(&exp[2], env)? {
            Expression::List(l) => {
                let mut result: Vec<Expression> = Vec::new();
                for e in l {
                    result.push(f.execute_noeval(&[e.clone()].to_vec())?);
                }
                Ok(Expression::List(result))
            }
            _ => Err(create_error!("E1005")),
        },
        _ => Err(create_error!("E1006")),
    }
}
fn filter(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Function(f) => match eval(&exp[2], env)? {
            Expression::List(l) => {
                let mut result: Vec<Expression> = Vec::new();
                for e in &l {
                    match f.execute_noeval(&[e.clone()].to_vec())? {
                        Expression::Boolean(b) => {
                            if b {
                                result.push(e.clone());
                            }
                        }
                        _ => return Err(create_error!("E1001")),
                    }
                }
                Ok(Expression::List(result))
            }
            _ => Err(create_error!("E1005")),
        },
        _ => Err(create_error!("E1006")),
    }
}
fn reduce(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(f) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[3], env)? {
            if l.len() == 0 {
                return eval(&exp[2], env);
            }
            let mut result = l[0].clone();
            // not carfully length,  safety
            for e in &l[1 as usize..] {
                result = f.execute_noeval(&[result.clone(), e.clone()].to_vec())?;
            }
            Ok(result)
        } else {
            Err(create_error!("E1005"))
        }
    } else {
        Err(create_error!("E1006"))
    }
}
fn for_each(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(f) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[2], env)? {
            for e in l {
                f.execute_noeval(&[e.clone()].to_vec())?;
            }
        } else {
            return Err(create_error!("E1005"));
        }
        Ok(Expression::Nil())
    } else {
        Err(create_error!("E1006"))
    }
}
fn list_ref(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match eval(&exp[2], env)? {
            Expression::Integer(i) => {
                if i < 0 || l.len() <= i as usize {
                    Err(create_error!("E1011"))
                } else {
                    Ok(l[i as usize].clone())
                }
            }
            _ => Err(create_error!("E1002")),
        },
        _ => Err(create_error!("E1005")),
    }
}
fn rand_integer(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if 1 < exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    Ok(Expression::Integer(x.abs() / SAMPLE_INT))
}
fn rand_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
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
        Err(create_error!("E1002"))
    }
}
fn load_file(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::String(s) = v {
        if false == Path::new(&s).exists() {
            return Err(create_error!("E1014"));
        }
        let file = match File::open(s) {
            Err(e) => return Err(create_error_value!("E1014", e)),
            Ok(file) => file,
        };
        let meta = match file.metadata() {
            Err(e) => return Err(create_error_value!("E9999", e)),
            Ok(meta) => meta,
        };
        if true == meta.is_dir() {
            return Err(create_error!("E1016"));
        }
        let mut stream = BufReader::new(file);
        match repl(&mut stream, env, true) {
            Err(e) => return Err(create_error_value!("E9999", e)),
            Ok(_) => return Ok(Expression::Nil()),
        }
    }
    Err(create_error!("E1015"))
}
fn display(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let v = eval(e, env)?;
        if let Expression::Char(c) = v {
            print!("{} ", c);
        } else {
            print!("{} ", v.to_string());
        }
    }
    Ok(Expression::Nil())
}
fn newline(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    print!("\n");
    Ok(Expression::Nil())
}

fn begin(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut ret = Expression::Nil();
    for e in &exp[1 as usize..] {
        ret = eval(e, env)?;
    }
    return Ok(ret);
}

fn delay(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    Ok(Expression::Promise(Box::new(exp[1].clone()), env.clone()))
}
fn force(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::Promise(p, pe) = v {
        eval(&(*p), &pe)
    } else {
        Ok(v)
    }
}
fn format_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let s = if let Expression::String(s) = eval(&exp[1], env)? {
        s
    } else {
        return Err(create_error!("E1015"));
    };
    let i = if let Expression::Integer(i) = eval(&exp[2], env)? {
        i
    } else {
        return Err(create_error!("E1002"));
    };
    let s = match s.as_str() {
        "~X" => format!("{:X}", i),
        "~x" => format!("{:x}", i),
        n => match n.to_lowercase().as_str() {
            "~d" => format!("{:?}", i),
            "~o" => format!("{:o}", i),
            "~b" => format!("{:b}", i),
            _ => return Err(create_error!("E1018")),
        },
    };
    Ok(Expression::String(s))
}
fn to_f64(exp: &[Expression], env: &Environment) -> Result<f64, RsError> {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(f) => Ok(f),
        Expression::Integer(i) => Ok(i as f64),
        Expression::Rational(r) => Ok(r.div_float()),
        _ => Err(create_error!("E1003")),
    }
}
fn calc(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {
    let mut result: Number = Number::Integer(0);
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Float(v) => Number::Float(v),
            Expression::Integer(v) => Number::Integer(v),
            Expression::Rational(v) => Number::Rational(v),
            _ => return Err(create_error!("E1003")),
        };
        if first == true {
            result = param;
            first = false;
            continue;
        }
        result = f(result, param);
    }
    match result {
        Number::Integer(a) => Ok(Expression::Integer(a)),
        Number::Float(a) => Ok(Expression::Float(a)),
        Number::Rational(a) => Ok(Expression::Rational(a)),
    }
}
fn cmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &Number, y: &Number) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v: [Number; 2] = [Number::Integer(0); 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Float(f) => Number::Float(f),
            Expression::Integer(i) => Number::Integer(i),
            Expression::Rational(r) => Number::Rational(r),
            _ => return Err(create_error!("E1003")),
        }
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn shift(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut x: [i64; 2] = [0; 2];
    for (i, e) in exp[1 as usize..].iter().enumerate() {
        x[i] = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!("E1002")),
        };
    }
    Ok(Expression::Integer(if x[1] >= 0 {
        x[0] << x[1]
    } else {
        x[0] >> x[1].abs()
    }))
}
fn bit(exp: &[Expression], env: &Environment, f: fn(x: i64, y: i64) -> i64) -> ResultExpression {
    let mut result: i64 = 0;
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!("E1002")),
        };
        if first == true {
            result = param;
            first = false;
            continue;
        }
        result = f(result, param);
    }
    Ok(Expression::Integer(result))
}
fn lognot(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        Err(create_error_value!("E1007", exp.len()))
    } else {
        match eval(&exp[1], env)? {
            Expression::Integer(v) => Ok(Expression::Integer(!v)),
            _ => Err(create_error!("E1002")),
        }
    }
}
fn abs(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    Ok(match eval(&exp[1], env)? {
        Expression::Float(v) => Expression::Float(v.abs()),
        Expression::Integer(v) => Expression::Integer(v.abs()),
        Expression::Rational(v) => Expression::Rational(v.abs()),
        _ => return Err(create_error!("E1003")),
    })
}
fn odd_even(exp: &[Expression], env: &Environment, f: fn(i64) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Integer(i) => Ok(Expression::Boolean(f(i))),
        _ => return Err(create_error!("E1002")),
    }
}
fn is_sign(exp: &[Expression], env: &Environment, f: fn(&Number) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Float(f) => Number::Float(f),
        Expression::Integer(i) => Number::Integer(i),
        Expression::Rational(r) => Number::Rational(r),
        _ => return Err(create_error!("E1003")),
    };
    Ok(Expression::Boolean(f(&v)))
}
fn is_type(
    exp: &[Expression],
    env: &Environment,
    f: fn(e: &Expression) -> bool,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    Ok(Expression::Boolean(f(&v)))
}
fn strcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &String, y: &String) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v = Vec::new();
    for e in &exp[1 as usize..] {
        let s = match eval(e, env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!("E1015")),
        };
        v.push(s);
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn charcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: char, y: char) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v: [char; 2] = [' '; 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!("E1019")),
        }
    }
    Ok(Expression::Boolean(f(v[0], v[1])))
}
fn str_append(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 > exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v = String::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::String(s) => v.push_str(&s.into_boxed_str()),
            _ => return Err(create_error!("E1015")),
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
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Integer(f(s) as i64)),
        _ => return Err(create_error!("E1015")),
    }
}
fn number_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Float(f) => Expression::Float(f),
        Expression::Integer(i) => Expression::Integer(i),
        Expression::Rational(r) => Expression::Rational(r),
        _ => return Err(create_error!("E1003")),
    };
    Ok(Expression::String(v.to_string()))
}
fn string_number(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!("E1015")),
    };
    let v = if let Ok(n) = s.parse::<i64>() {
        Expression::Integer(n)
    } else if let Ok(n) = s.parse::<f64>() {
        Expression::Float(n)
    } else {
        match Rat::from(&s) {
            Ok(n) => Expression::Rational(n),
            Err(n) => {
                return if n.code != "E1020" {
                    Err(create_error!(n.code))
                } else {
                    Err(create_error!("E1003"))
                }
            }
        }
    };
    Ok(v)
}
fn list_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!("E1005")),
    };
    let mut v = String::new();

    for e in l.into_iter() {
        v.push(match eval(&e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!("E1019")),
        });
    }
    Ok(Expression::String(v))
}
fn string_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!("E1015")),
    };
    let mut l: Vec<Expression> = Vec::new();
    for c in s.as_str().chars() {
        l.push(Expression::Char(c));
    }
    Ok(Expression::List(l))
}
fn integer_char(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let i = match eval(&exp[1], env)? {
        Expression::Integer(i) => i,
        _ => return Err(create_error!("E1002")),
    };
    let i = i as u32;
    if let Some(c) = char::from_u32(i) {
        Ok(Expression::Char(c))
    } else {
        Err(create_error!("E1019"))
    }
}
fn char_integer(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let c = match eval(&exp[1], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!("E1019")),
    };
    let a = c as u32;
    Ok(Expression::Integer(a as i64))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::collections::HashMap;

    fn create_function_dyn_dispatch(b: &mut dyn BuildInTable) {
        create_function(b);
    }

    #[cfg(not(feature = "thread"))]
    impl BuildInTable for HashMap<&'static str, Operation> {
        fn regist(&mut self, symbol: &'static str, func: Operation) {
            self.insert(symbol, func);
        }
    }

    #[cfg(feature = "thread")]
    impl BuildInTable for BTreeMap<&'static str, Operation> {
        fn regist(&mut self, symbol: &'static str, func: Operation) {
            self.insert(symbol, func);
        }
    }
    #[test]
    fn test_dyn_dispatch() {
        let mut b = BTreeMap::new();
        let mut h = HashMap::new();
        create_function_dyn_dispatch(&mut b);
        create_function_dyn_dispatch(&mut h);
    }
}
