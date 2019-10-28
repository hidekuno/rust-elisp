/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
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

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::Integer(v) => v,
        _ => return Err(create_error!(RsCode::E1002)),
    };
    if l < 0 {
        return Err(create_error!(RsCode::E1011));
    }
    let v = eval(&exp[2], env)?;

    Ok(Expression::List(vec![v; l as usize]))
}
fn null_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => Ok(Expression::Boolean(l.len() == 0)),
        _ => Ok(Expression::Boolean(false)),
    }
}
fn length(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        Ok(Expression::Integer(l.len() as i64))
    } else {
        Err(create_error!(RsCode::E1005))
    }
}
fn car(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            if l.len() <= 0 {
                return Err(create_error!(RsCode::E1011));
            }
            Ok(l[0].clone())
        }
        Expression::Pair(car, _cdr) => Ok((*car).clone()),
        _ => Err(create_error!(RsCode::E1005)),
    }
}
fn cdr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match l.len() {
            0 => Err(create_error!(RsCode::E1011)),
            1 => Ok(Expression::List(Vec::new())),
            _ => Ok(Expression::List(l[1 as usize..].to_vec())),
        },
        Expression::Pair(_car, cdr) => Ok((*cdr).clone()),
        _ => Err(create_error!(RsCode::E1005)),
    }
}
fn cadr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        if l.len() <= 1 {
            return Err(create_error!(RsCode::E1011));
        }
        Ok(l[1].clone())
    } else {
        Err(create_error!(RsCode::E1005))
    }
}
fn cons(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut v: Vec<Expression> = Vec::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::List(mut l) => v.append(&mut l),
            _ => return Err(create_error!(RsCode::E1005)),
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!(RsCode::E1005)),
    };
    let n = match eval(&exp[2], env)? {
        Expression::Integer(n) => n,
        _ => return Err(create_error!(RsCode::E1002)),
    };
    if l.len() < n as usize || n < 0 {
        return Err(create_error!(RsCode::E1011));
    }
    let mut vec = Vec::new();
    vec.extend_from_slice(f(&l, n as usize));

    Ok(Expression::List(vec))
}
fn delete(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let other = eval(&exp[1], env)?;
    let l = match eval(&exp[2], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!(RsCode::E1005)),
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
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match l.len() {
            0 => Err(create_error!(RsCode::E1011)),
            _ => Ok(l[l.len() - 1].clone()),
        },
        Expression::Pair(car, _) => Ok(*car.clone()),
        _ => Err(create_error!(RsCode::E1005)),
    }
}
fn reverse(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let mut v = l.clone();
            v.reverse();
            Ok(Expression::List(v))
        }
        _ => Err(create_error!(RsCode::E1005)),
    }
}
fn iota(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() <= 1 || 4 < exp.len() {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let mut param: [i64; 4] = [0, 0, 1, 0];
    for (i, e) in exp[1 as usize..].iter().enumerate() {
        match eval(e, env)? {
            Expression::Integer(v) => {
                param[i] = v;
            }
            _ => return Err(create_error!(RsCode::E1002)),
        }
    }
    let (to, from, step) = (param[0] + param[1], param[1], param[2]);
    let mut l = Vec::with_capacity(to as usize);
    let mut v = from;
    for _ in from..to {
        l.push(Expression::Integer(v));
        v += step;
    }
    Ok(Expression::List(l))
}
fn map(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
            _ => Err(create_error!(RsCode::E1005)),
        },
        _ => Err(create_error!(RsCode::E1006)),
    }
}
fn filter(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
                        _ => return Err(create_error!(RsCode::E1001)),
                    }
                }
                Ok(Expression::List(result))
            }
            _ => Err(create_error!(RsCode::E1005)),
        },
        _ => Err(create_error!(RsCode::E1006)),
    }
}
fn reduce(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
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
            Err(create_error!(RsCode::E1005))
        }
    } else {
        Err(create_error!(RsCode::E1006))
    }
}
fn for_each(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    if let Expression::Function(f) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[2], env)? {
            for e in l {
                f.execute_noeval(&[e.clone()].to_vec())?;
            }
        } else {
            return Err(create_error!(RsCode::E1005));
        }
        Ok(Expression::Nil())
    } else {
        Err(create_error!(RsCode::E1006))
    }
}
fn list_ref(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => match eval(&exp[2], env)? {
            Expression::Integer(i) => {
                if i < 0 || l.len() <= i as usize {
                    Err(create_error!(RsCode::E1011))
                } else {
                    Ok(l[i as usize].clone())
                }
            }
            _ => Err(create_error!(RsCode::E1002)),
        },
        _ => Err(create_error!(RsCode::E1005)),
    }
}
