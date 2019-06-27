/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;

use crate::lisp::{eval, repl};
use crate::lisp::{Expression, Operation, ResultExpression};
use crate::lisp::{RsError, RsFunction, RsLetLoop};

use crate::number::Number;
use crate::number::Rat;

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;
//========================================================================
const SAMPLE_INT: i64 = 10000000000000;
//========================================================================
pub fn create_function(b: &mut HashMap<&'static str, Operation>) {
    b.insert("+", |exp, env| calc(exp, env, |x, y| x + y));
    b.insert("-", |exp, env| calc(exp, env, |x, y| x - y));
    b.insert("*", |exp, env| calc(exp, env, |x, y| x * y));
    b.insert("/", |exp, env| calc(exp, env, |x, y| x / y));
    b.insert("=", |exp, env| cmp(exp, env, |x, y| x == y));
    b.insert("<", |exp, env| cmp(exp, env, |x, y| x < y));
    b.insert("<=", |exp, env| cmp(exp, env, |x, y| x <= y));
    b.insert(">", |exp, env| cmp(exp, env, |x, y| x > y));
    b.insert(">=", |exp, env| cmp(exp, env, |x, y| x >= y));
    b.insert("expt", expt);
    b.insert("modulo", |exp, env| divide(exp, env, |x, y| x % y));
    b.insert("quotient", |exp, env| divide(exp, env, |x, y| x / y));
    b.insert("define", define);
    b.insert("lambda", lambda);
    b.insert("if", if_f);
    b.insert("and", and);
    b.insert("or", or);
    b.insert("not", not);
    b.insert("let", let_f);
    b.insert("time", time_f);
    b.insert("set!", set_f);
    b.insert("cond", cond);
    b.insert("eq?", eqv);
    b.insert("eqv?", eqv);
    b.insert("case", case);
    b.insert("apply", apply);
    b.insert("identity", identity);

    b.insert("list", list);
    b.insert("null?", null_f);
    b.insert("length", length);
    b.insert("car", car);
    b.insert("cdr", cdr);
    b.insert("cadr", cadr);
    b.insert("cons", cons);
    b.insert("append", append);
    b.insert("last", last);
    b.insert("reverse", reverse);
    b.insert("iota", iota);
    b.insert("map", map);
    b.insert("filter", filter);
    b.insert("reduce", reduce);
    b.insert("for-each", for_each);

    b.insert("sqrt", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sqrt()))
    });
    b.insert("sin", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sin()))
    });
    b.insert("cos", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.cos()))
    });
    b.insert("tan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.tan()))
    });
    b.insert("atan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.atan()))
    });
    b.insert("exp", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.exp()))
    });
    b.insert("log", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.log((1.0 as f64).exp())))
    });
    b.insert("rand-integer", rand_integer);
    b.insert("rand-list", rand_list);

    b.insert("load-file", load_file);
    b.insert("display", display);
    b.insert("newline", newline);
    b.insert("begin", begin);

    b.insert("delay", delay);
    b.insert("force", force);
}
fn set_f(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    fn search_symbol(env: &mut Environment, s: &String) -> Option<Expression> {
        return env.find(s);
    }
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Symbol(s) = &exp[1] {
        if let Some(_) = search_symbol(env, s) {
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
fn time_f(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }

    let start = Instant::now();
    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{}(s)", end.as_secs(), end.subsec_nanos() / 1_000_000);
    return result;
}
fn let_f(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }

    // @@@ env.create();
    let mut param = Environment::new_next(env);
    let mut idx = 1;
    if let Expression::Symbol(_) = exp[idx] {
        idx += 1;
    }
    // Parameter Setup
    let mut param_list = Vec::new();
    if let Expression::List(l) = &exp[idx] {
        for plist in l {
            if let Expression::List(p) = plist {
                if p.len() != 2 {
                    return Err(create_error_value!("E1007", p.len()));
                }
                if let Expression::Symbol(s) = &p[0] {
                    param.regist(s.to_string(), eval(&p[1], env)?);
                    param_list.push(s.clone());
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
    // Setup label name let
    if let Expression::Symbol(s) = &exp[1] {
        let mut letloop = RsLetLoop::new(exp, s.to_string(), &param_list);
        letloop.set_tail_recurcieve();
        param.regist(s.to_string(), Environment::create_let_loop(letloop));
    }
    let mut ret = Expression::Nil();
    for e in &exp[idx as usize..] {
        loop {
            let v = eval(e, &mut param)?;
            if let Expression::Loop() = v {
                // tail recurcieve
                continue;
            } else {
                ret = v;
                break;
            }
        }
    }
    Ok(ret)
}
fn not(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Boolean(b) => Ok(Expression::Boolean(!b)),
        _ => Err(create_error!("E1001")),
    }
}
fn or(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn and(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn expt(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    macro_rules! natural_log {
        ($x: expr, $y: expr) => {
            ($x.log((1.0 as f64).exp()) * $y).exp()
        };
    }
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float(natural_log!(x, y))),
            Expression::Integer(y) => Ok(Expression::Float(natural_log!(x, (y as f64)))),
            _ => Err(create_error!("E1003")),
        },
        Expression::Integer(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float(natural_log!((x as f64), y))),
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
    env: &mut Environment,
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
fn lambda(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn define(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Symbol(v) = &exp[1] {
        let se = eval(&exp[2], env)?;
        env.regist(v.to_string(), se);

        return Ok(Expression::Symbol(v.to_string()));
    }
    if let Expression::List(l) = &exp[1] {
        if l.len() < 1 {
            return Err(create_error_value!("E1007", l.len()));
        }
        if let Expression::Symbol(s) = &l[0] {
            let mut f = exp.to_vec();

            let mut param: Vec<Expression> = Vec::new();
            for n in &l[1..] {
                match n {
                    Expression::Symbol(_) => {
                        param.push(n.clone());
                    }
                    _ => return Err(create_error!("E1004")),
                }
            }
            f[1] = Expression::List(param);
            let mut func = RsFunction::new(&f, s.to_string(), env.clone());
            func.set_tail_recurcieve();
            env.regist(s.to_string(), Environment::create_func(func));

            Ok(Expression::Symbol(s.to_string()))
        } else {
            Err(create_error!("E1004"))
        }
    } else {
        Err(create_error!("E1004"))
    }
}
fn if_f(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn cond(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::List(l) = e {
            let mut iter = l.iter();

            if let Some(e) = iter.next() {
                if let Expression::Symbol(s) = e {
                    if s.as_str() != "else" {
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
fn eqv(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn case(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
                        if s.as_str() != "else" {
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
fn apply(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = eval(&exp[2], env)? {
        let mut se: Vec<Expression> = Vec::new();
        se.push(exp[1].clone());
        for e in &l {
            se.push(e.clone());
        }
        eval(&Expression::List(se), env)
    } else {
        Err(create_error_value!("E1005", exp.len()))
    }
}
pub fn identity(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    eval(&exp[1], env)
}
fn list(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    let mut list: Vec<Expression> = Vec::with_capacity(exp.len());
    for e in &exp[1 as usize..] {
        list.push(eval(e, env)?);
    }
    Ok(Expression::List(list))
}
fn null_f(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => Ok(Expression::Boolean(l.len() == 0)),
        _ => Ok(Expression::Boolean(false)),
    }
}
fn length(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        Ok(Expression::Integer(l.len() as i64))
    } else {
        Err(create_error!("E1005"))
    }
}
fn car(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn cdr(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn cadr(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn cons(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn append(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn last(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn reverse(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn iota(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn map(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn filter(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn reduce(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn for_each(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn rand_integer(exp: &[Expression], _env: &mut Environment) -> ResultExpression {
    if 1 < exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    Ok(Expression::Integer(x.abs() / SAMPLE_INT))
}
fn rand_list(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if 2 < exp.len() {
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
fn load_file(exp: &[Expression], env: &mut Environment) -> ResultExpression {
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
fn display(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let v = eval(e, env)?;
        if let Expression::Char(c) = v {
            print!("{} ", c);
        } else {
            print!("{} ", v.value_string());
        }
    }
    Ok(Expression::Nil())
}
fn newline(exp: &[Expression], _env: &mut Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    print!("\n");
    Ok(Expression::Nil())
}

fn begin(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut ret = Expression::Nil();
    for e in &exp[1 as usize..] {
        ret = eval(e, env)?;
    }
    return Ok(ret);
}

fn delay(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    Ok(Expression::Promise(Box::new(exp[1].clone()), env.clone()))
}
fn force(exp: &[Expression], env: &mut Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::Promise(p, mut pe) = v {
        eval(&(*p), &mut pe)
    } else {
        Ok(v)
    }
}
fn to_f64(exp: &[Expression], env: &mut Environment) -> Result<f64, RsError> {
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
    env: &mut Environment,
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
    env: &mut Environment,
    f: fn(x: &Number, y: &Number) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut vec: Vec<Number> = Vec::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::Float(f) => vec.push(Number::Float(f)),
            Expression::Integer(i) => vec.push(Number::Integer(i)),
            Expression::Rational(r) => vec.push(Number::Rational(r)),
            _ => return Err(create_error!("E1003")),
        }
    }
    Ok(Expression::Boolean(f(&vec[0], &vec[1])))
}
