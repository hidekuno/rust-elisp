/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use std::env;
use std::time::Instant;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;

use crate::lisp::eval;
use crate::lisp::{Expression, Operation, ResultExpression};
use crate::lisp::{RsCode, RsError, RsFunction};

use crate::chars;
use crate::io;
use crate::list;
use crate::math;
use crate::strings;

use crate::number::Number;
use crate::number::Rat;

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;

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
    b.regist("begin", begin);

    b.regist("delay", delay);
    b.regist("force", force);

    b.regist("quote", |exp, _env| {
        if exp.len() != 2 {
            Err(create_error_value!(RsCode::E1007, exp.len()))
        } else {
            Ok(exp[1].clone())
        }
    });
    b.regist("get-environment-variable", get_env);

    chars::create_function(b);
    list::create_function(b);
    math::create_function(b);
    strings::create_function(b);
    io::create_function(b);
}
fn set_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::Symbol(s) = &exp[1] {
        if let Some(_) = env.find(s) {
            let v = eval(&exp[2], env)?;
            env.update(s, v);
        } else {
            return Err(create_error_value!(RsCode::E1008, s));
        }
        Ok(Expression::Symbol(s.to_string()))
    } else {
        Err(create_error!(RsCode::E1004))
    }
}
fn time_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }

    let start = Instant::now();
    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{:03}(s)", end.as_secs(), end.subsec_nanos() / 1_000_000);
    return result;
}
fn let_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
                    return Err(create_error_value!(RsCode::E1007, p.len()));
                }
                if let Expression::Symbol(s) = &p[0] {
                    param_list.push(Expression::Symbol(s.clone()));
                    param_value_list.push(p[1].clone());
                } else {
                    return Err(create_error!(RsCode::E1004));
                }
            } else {
                return Err(create_error!(RsCode::E1005));
            }
        }
        idx += 1;
    } else {
        return Err(create_error!(RsCode::E1005));
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Boolean(b) => Ok(Expression::Boolean(!b)),
        _ => Err(create_error!(RsCode::E1001)),
    }
}
fn or(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == true {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!(RsCode::E1001));
        }
    }
    Ok(Expression::Boolean(false))
}
fn and(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == false {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!(RsCode::E1001));
        }
    }
    Ok(Expression::Boolean(true))
}
fn expt(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float(x.powf(y))),
            Expression::Integer(y) => Ok(Expression::Float(x.powf(y as f64))),
            _ => Err(create_error!(RsCode::E1003)),
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
            _ => Err(create_error!(RsCode::E1003)),
        },
        _ => Err(create_error!(RsCode::E1003)),
    }
}
fn divide(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &i64, y: &i64) -> i64,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);
    match (a, b) {
        (Expression::Integer(x), Expression::Integer(y)) => {
            if y == 0 {
                Err(create_error!(RsCode::E1013))
            } else {
                Ok(Expression::Integer(f(&x, &y)))
            }
        }
        (_, _) => Err(create_error!(RsCode::E1002)),
    }
}
fn lambda(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::List(l) = &exp[1] {
        for e in l {
            match e {
                Expression::Symbol(_) => {}
                _ => return Err(create_error!(RsCode::E1004)),
            }
        }
    } else {
        return Err(create_error!(RsCode::E1005));
    }
    Ok(Environment::create_func(RsFunction::new(
        exp,
        String::from("lambda"),
        env.clone(),
    )))
}
fn define(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::Symbol(v) = &exp[1] {
        if exp.len() != 3 {
            return Err(create_error_value!(RsCode::E1007, exp.len()));
        }
        let se = eval(&exp[2], env)?;
        env.regist(v.to_string(), se);

        return Ok(Expression::Symbol(v.to_string()));
    }
    if let Expression::List(l) = &exp[1] {
        if l.len() < 1 {
            return Err(create_error_value!(RsCode::E1007, l.len()));
        }
        if let Expression::Symbol(s) = &l[0] {
            let mut param: Vec<Expression> = Vec::new();
            for n in &l[1..] {
                match n {
                    Expression::Symbol(_) => {
                        param.push(n.clone());
                    }
                    _ => return Err(create_error!(RsCode::E1004)),
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
            Err(create_error!(RsCode::E1004))
        }
    } else {
        Err(create_error!(RsCode::E1004))
    }
}
fn if_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
        Err(create_error!(RsCode::E1001))
    }
}
fn cond(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
                return Err(create_error!(RsCode::E1012));
            }
            return begin(&l, env);
        } else {
            return Err(create_error!(RsCode::E1005));
        }
    }
    Ok(Expression::Nil())
}
fn eqv(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);

    match a {
        Expression::Float(x) => match b {
            Expression::Float(y) => Ok(Expression::Boolean(x == y)),
            _ => Ok(Expression::Boolean(false)),
        },
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
        Expression::Boolean(x) => match b {
            Expression::Boolean(y) => Ok(Expression::Boolean(x == y)),
            _ => Ok(Expression::Boolean(false)),
        },
        Expression::Symbol(x) => match b {
            Expression::Symbol(y) => Ok(Expression::Boolean(x == y)),
            _ => Ok(Expression::Boolean(false)),
        },
        Expression::Char(x) => match b {
            Expression::Char(y) => Ok(Expression::Boolean(x == y)),
            _ => Ok(Expression::Boolean(false)),
        },
        _ => Ok(Expression::Boolean(false)),
    }
}
fn case(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
                            return Err(create_error!(RsCode::E1017));
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
                    _ => return Err(create_error!(RsCode::E1017)),
                }
            } else {
                return Err(create_error!(RsCode::E1005));
            }
        }
    }
    Ok(Expression::Nil())
}
fn apply(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::List(l) = eval(&exp[2], env)? {
        let mut se: Vec<Expression> = Vec::new();
        se.push(exp[1].clone());
        se.extend_from_slice(&l);
        eval(&Expression::List(se), env)
    } else {
        Err(create_error_value!(RsCode::E1005, exp.len()))
    }
}
pub fn identity(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    eval(&exp[1], env)
}
fn begin(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut ret = Expression::Nil();
    for e in &exp[1 as usize..] {
        ret = eval(e, env)?;
    }
    return Ok(ret);
}
fn delay(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    Ok(Expression::Promise(Box::new(exp[1].clone()), env.clone()))
}
fn force(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::Promise(p, pe) = v {
        eval(&(*p), &pe)
    } else {
        Ok(v)
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Float(v) => Number::Float(v),
            Expression::Integer(v) => Number::Integer(v),
            Expression::Rational(v) => Number::Rational(v),
            _ => return Err(create_error!(RsCode::E1003)),
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut v: [Number; 2] = [Number::Integer(0); 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Float(f) => Number::Float(f),
            Expression::Integer(i) => Number::Integer(i),
            Expression::Rational(r) => Number::Rational(r),
            _ => return Err(create_error!(RsCode::E1003)),
        }
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn shift(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut x: [i64; 2] = [0; 2];
    for (i, e) in exp[1 as usize..].iter().enumerate() {
        x[i] = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(RsCode::E1002)),
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(RsCode::E1002)),
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
        Err(create_error_value!(RsCode::E1007, exp.len()))
    } else {
        match eval(&exp[1], env)? {
            Expression::Integer(v) => Ok(Expression::Integer(!v)),
            _ => Err(create_error!(RsCode::E1002)),
        }
    }
}
fn odd_even(exp: &[Expression], env: &Environment, f: fn(i64) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Integer(i) => Ok(Expression::Boolean(f(i))),
        _ => return Err(create_error!(RsCode::E1002)),
    }
}
fn is_sign(exp: &[Expression], env: &Environment, f: fn(&Number) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Float(f) => Number::Float(f),
        Expression::Integer(i) => Number::Integer(i),
        Expression::Rational(r) => Number::Rational(r),
        _ => return Err(create_error!(RsCode::E1003)),
    };
    Ok(Expression::Boolean(f(&v)))
}
fn is_type(
    exp: &[Expression],
    env: &Environment,
    f: fn(e: &Expression) -> bool,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    Ok(Expression::Boolean(f(&v)))
}
fn get_env(exp: &[Expression], env: &Environment) -> ResultExpression {
    //srfi-98
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => match env::var(s) {
            Ok(v) => Ok(Expression::String(v)),
            Err(_) => Ok(Expression::Boolean(false)),
        },
        _ => Err(create_error!(RsCode::E1015)),
    }
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
