/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::cmp::Ordering;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;
use crate::mut_obj;
use crate::reference_obj;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, Int, ListRc, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::syntax::quote;

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
    b.regist("append!", append_effect);
    b.regist("take", |exp, env| take_drop(exp, env, |l, n| &l[0..n]));
    b.regist("drop", |exp, env| take_drop(exp, env, |l, n| &l[n..]));
    b.regist("delete", delete);
    b.regist("delete!", delete_effect);
    b.regist("last", last);
    b.regist("reverse", reverse);
    b.regist("iota", iota);
    b.regist("map", map);
    b.regist("filter", filter);
    b.regist("reduce", reduce);
    b.regist("for-each", for_each);
    b.regist("list-ref", list_ref);
    b.regist("list-set!", list_set);
    b.regist("set-car!", set_car);
    b.regist("set-cdr!", set_cdr);

    b.regist("sort", sort);
    b.regist("sort!", sort_effect);
    b.regist("merge", merge);
    b.regist("stable-sort", sort_stable);
    b.regist("stable-sort!", sort_stable_effect);
    b.regist("sorted?", is_sorted);

    b.regist("vector", vector);
    b.regist("make-vector", make_vector);
    b.regist("vector-length", vector_length);
    b.regist("vector->list", vector_list);
    b.regist("list->vector", list_vector);
    b.regist("vector-append", vector_append);
    b.regist("vector-append!", vector_append_effect);
    b.regist("vector-ref", vector_ref);
    b.regist("vector-set!", vector_set);
}
fn get_sequence(exp: Expression, err: ErrCode) -> Result<ListRc, Error> {
    if let Expression::List(l) = exp {
        if err != ErrCode::E1005 {
            Err(create_error!(err))
        } else {
            Ok(l)
        }
    } else if let Expression::Vector(l) = exp {
        if err != ErrCode::E1022 {
            Err(create_error!(err))
        } else {
            Ok(l)
        }
    } else {
        Err(create_error!(err))
    }
}
fn list(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = seq(exp, env)?;
    Ok(Environment::create_list(l))
}
fn seq(exp: &[Expression], env: &Environment) -> Result<Vec<Expression>, Error> {
    let mut list: Vec<Expression> = Vec::with_capacity(exp.len());
    for e in &exp[1..] {
        list.push(eval(e, env)?);
    }
    Ok(list)
}
fn make_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = make_seq(exp, env)?;
    Ok(Environment::create_list(l))
}
fn make_seq(exp: &[Expression], env: &Environment) -> Result<Vec<Expression>, Error> {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let n = match eval(&exp[1], env)? {
        Expression::Integer(v) => v,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if n < 0 {
        return Err(create_error!(ErrCode::E1011));
    }
    let v = eval(&exp[2], env)?;
    Ok(vec![v; n as usize])
}
fn null_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            Ok(Expression::Boolean(l.is_empty()))
        }
        _ => Ok(Expression::Boolean(false)),
    }
}
fn length(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_length(exp, env, ErrCode::E1005)
}
fn seq_length(exp: &[Expression], env: &Environment, err: ErrCode) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = get_sequence(eval(&exp[1], env)?, err)?;
    let l = &*(reference_obj!(l));
    Ok(Expression::Integer(l.len() as Int))
}
fn car(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            if l.is_empty() {
                return Err(create_error!(ErrCode::E1011));
            }
            Ok(l[0].clone())
        }
        Expression::Pair(car, _cdr) => Ok((*car).clone()),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn cdr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            match l.len() {
                0 => Err(create_error!(ErrCode::E1011)),
                1 => Ok(Environment::create_list(Vec::new())),
                _ => Ok(Environment::create_list(l[1..].to_vec())),
            }
        }
        Expression::Pair(_car, cdr) => Ok((*cdr).clone()),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn cadr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::List(l) = eval(&exp[1], env)? {
        let l = &*(reference_obj!(l));
        if l.len() <= 1 {
            return Err(create_error!(ErrCode::E1011));
        }
        Ok(l[1].clone())
    } else {
        Err(create_error!(ErrCode::E1005))
    }
}
fn cons(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let car = eval(&exp[1], env)?;
    let cdr = eval(&exp[2], env)?;

    if let Expression::List(l) = cdr {
        let l = reference_obj!(l);
        let mut v: Vec<Expression> = vec![car];
        v.append(&mut l.to_vec());
        Ok(Environment::create_list(v))
    } else {
        Ok(Expression::Pair(Box::new(car), Box::new(cdr)))
    }
}
fn append(exp: &[Expression], env: &Environment) -> ResultExpression {
    let v = seq_append(exp, env, ErrCode::E1005)?;
    Ok(Environment::create_list(v))
}
fn seq_append(
    exp: &[Expression],
    env: &Environment,
    err: ErrCode,
) -> Result<Vec<Expression>, Error> {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v: Vec<Expression> = Vec::new();
    for e in &exp[1..] {
        let l = get_sequence(eval(e, env)?, err.clone())?;
        let l = reference_obj!(l);
        v.append(&mut l.to_vec());
    }
    Ok(v)
}
fn append_effect(exp: &[Expression], env: &Environment) -> ResultExpression {
    let v = seq_append_effect(exp, env, ErrCode::E1005)?;
    Ok(Expression::List(v))
}
fn seq_append_effect(exp: &[Expression], env: &Environment, err: ErrCode) -> Result<ListRc, Error> {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let rc = get_sequence(eval(&exp[1], env)?, err.clone())?;

    let mut v = mut_obj!(&rc);
    for e in &exp[2..] {
        let l = get_sequence(eval(e, env)?, err.clone())?;
        let l = reference_obj!(l);
        v.append(&mut l.to_vec());
    }
    Ok(rc.clone())
}
fn take_drop(
    exp: &[Expression],
    env: &Environment,
    func: fn(l: &Vec<Expression>, n: usize) -> &[Expression],
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let l = reference_obj!(l);

    let n = match eval(&exp[2], env)? {
        Expression::Integer(n) => n,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if l.len() < n as usize || n < 0 {
        return Err(create_error!(ErrCode::E1011));
    }
    let mut vec = Vec::new();
    vec.extend_from_slice(func(&l, n as usize));

    Ok(Environment::create_list(vec))
}
fn delete(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let other = eval(&exp[1], env)?;
    let l = match eval(&exp[2], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let l = &*(reference_obj!(l));
    let mut vec = Vec::new();
    for e in l {
        if Expression::eqv(e, &other) {
            continue;
        }
        vec.push(e.clone());
    }
    Ok(Environment::create_list(vec))
}
fn delete_effect(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let other = eval(&exp[1], env)?;
    let rc = match eval(&exp[2], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let mut l = mut_obj!(&rc);
    let mut vec = Vec::new();
    for e in l.iter() {
        if Expression::eqv(e, &other) {
            continue;
        }
        vec.push(e.clone());
    }
    l.clear();
    l.extend_from_slice(&vec);
    Ok(Expression::List(rc.clone()))
}
fn last(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            match l.len() {
                0 => Err(create_error!(ErrCode::E1011)),
                _ => Ok(l[l.len() - 1].clone()),
            }
        }
        Expression::Pair(car, _) => Ok(*car),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn reverse(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            let mut l = l.to_vec();
            l.reverse();
            Ok(Environment::create_list(l))
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn iota(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() <= 1 || 4 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut param: [Int; 4] = [0, 0, 1, 0];
    for (i, e) in exp[1..].iter().enumerate() {
        match eval(e, env)? {
            Expression::Integer(v) => {
                param[i] = v;
            }
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        }
    }
    let (to, from, step) = (param[0] + param[1], param[1], param[2]);
    let mut l = if to > 16 {
        Vec::with_capacity(to as usize)
    } else {
        Vec::new()
    };
    let mut v = from;
    for _ in from..to {
        l.push(Expression::Integer(v));
        v += step;
    }
    Ok(Environment::create_list(l))
}
fn map(exp: &[Expression], env: &Environment) -> ResultExpression {
    fn func(
        sexp: Vec<Expression>,
        env: &Environment,
        result: &mut Vec<Expression>,
        _e: &Expression,
    ) -> ResultExpression {
        result.push(eval(&Environment::create_list(sexp), env)?);
        Ok(Expression::Nil())
    }

    do_list_proc(exp, env, func)
}
fn filter(exp: &[Expression], env: &Environment) -> ResultExpression {
    fn func(
        sexp: Vec<Expression>,
        env: &Environment,
        result: &mut Vec<Expression>,
        e: &Expression,
    ) -> ResultExpression {
        match eval(&Environment::create_list(sexp), env)? {
            Expression::Boolean(b) => {
                if b {
                    result.push(e.clone());
                }
            }
            e => return Err(create_error_value!(ErrCode::E1001, e)),
        }
        Ok(Expression::Nil())
    }
    do_list_proc(exp, env, func)
}
fn for_each(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }

    let callable = eval(&exp[1], env)?;

    match eval(&exp[2], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));

            for e in l {
                eval(
                    &Environment::create_list(make_evaled_list(&callable, &[e.clone()], &None)),
                    env,
                )?;
            }
            Ok(Expression::Nil())
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn reduce(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let callable = eval(&exp[1], env)?;

    if let Expression::List(l) = eval(&exp[3], env)? {
        let l = &*(reference_obj!(l));
        if l.is_empty() {
            return eval(&exp[2], env);
        }
        let mut result = l[0].clone();
        // not carfully length,  safety
        for e in &l[1..] {
            result = eval(
                &Environment::create_list(make_evaled_list(&callable, &[e.clone()], &Some(result))),
                env,
            )?;
        }
        Ok(result)
    } else {
        Err(create_error!(ErrCode::E1005))
    }
}
fn list_ref(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_list_ref(exp, env, ErrCode::E1005)
}
fn seq_list_ref(exp: &[Expression], env: &Environment, err: ErrCode) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = get_sequence(eval(&exp[1], env)?, err)?;
    let l = &*(reference_obj!(l));
    match eval(&exp[2], env)? {
        Expression::Integer(i) => {
            if i < 0 || l.len() <= i as usize {
                Err(create_error!(ErrCode::E1011))
            } else {
                Ok(l[i as usize].clone())
            }
        }
        e => Err(create_error_value!(ErrCode::E1002, e)),
    }
}
fn list_set(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_list_set(exp, env, ErrCode::E1005)
}

fn seq_list_set(exp: &[Expression], env: &Environment, err: ErrCode) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = get_sequence(eval(&exp[1], env)?, err)?;
    let i = match eval(&exp[2], env)? {
        Expression::Integer(i) => i,
        _ => {
            return Err(create_error!(ErrCode::E1002));
        }
    };
    let mut l = mut_obj!(l);
    if i < 0 || l.len() <= i as usize {
        return Err(create_error!(ErrCode::E1011));
    }
    l[i as usize] = eval(&exp[3], env)?;

    Ok(Expression::Nil())
}
pub fn make_evaled_list(
    callable: &Expression,
    exp: &[Expression],
    result: &Option<Expression>,
) -> Vec<Expression> {
    let mut sexp: Vec<Expression> = Vec::new();

    fn set_evaled_list_inner(sexp: &mut Vec<Expression>, exp: &Expression) {
        let ql: Vec<Expression> = vec![
            Expression::BuildInFunction("quote".to_string(), quote),
            exp.clone(),
        ];
        sexp.push(Environment::create_list(ql));
    }

    sexp.push(callable.clone());

    if let Some(e) = result {
        match e {
            Expression::List(_) | Expression::Symbol(_) => {
                set_evaled_list_inner(&mut sexp, e);
            }
            _ => sexp.push(e.clone()),
        }
    }
    for e in exp {
        match e {
            Expression::List(_) | Expression::Symbol(_) => {
                set_evaled_list_inner(&mut sexp, e);
            }
            _ => sexp.push(e.clone()),
        }
    }
    sexp
}
fn do_list_proc(
    exp: &[Expression],
    env: &Environment,
    func: fn(Vec<Expression>, &Environment, &mut Vec<Expression>, &Expression) -> ResultExpression,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let callable = eval(&exp[1], env)?;

    match eval(&exp[2], env)? {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));

            let mut result: Vec<Expression> = Vec::new();

            for e in l {
                func(
                    make_evaled_list(&callable, &[e.clone()], &None),
                    env,
                    &mut result,
                    e,
                )?;
            }
            Ok(Environment::create_list(result))
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn set_car(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(r) => {
            let mut l = mut_obj!(r);
            if l.is_empty() {
                return Err(create_error!(ErrCode::E1011));
            }
            l[0] = eval(&exp[2], env)?;
            Ok(Expression::Nil())
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn set_cdr(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(r) => {
            let mut l = mut_obj!(r);
            if l.is_empty() {
                return Err(create_error!(ErrCode::E1011));
            }

            let e = eval(&exp[2], env)?;
            let tmp = l[0].clone();
            l.clear();
            l.push(tmp);
            match e {
                Expression::List(m) => {
                    let m = &*(reference_obj!(m));
                    l.extend_from_slice(m);
                }
                _ => {
                    l.push(e);
                }
            }
            Ok(Expression::Nil())
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
enum ListProcKind {
    Copy,
    Effect,
}
enum SortKind {
    Stable(ListProcKind),
    Unstable(ListProcKind),
}
fn sort(exp: &[Expression], env: &Environment) -> ResultExpression {
    sort_impl(exp, env, SortKind::Unstable(ListProcKind::Copy))
}
fn sort_effect(exp: &[Expression], env: &Environment) -> ResultExpression {
    sort_impl(exp, env, SortKind::Unstable(ListProcKind::Effect))
}
fn sort_stable(exp: &[Expression], env: &Environment) -> ResultExpression {
    sort_impl(exp, env, SortKind::Stable(ListProcKind::Copy))
}
fn sort_stable_effect(exp: &[Expression], env: &Environment) -> ResultExpression {
    sort_impl(exp, env, SortKind::Stable(ListProcKind::Effect))
}
fn sort_impl(exp: &[Expression], env: &Environment, kind: SortKind) -> ResultExpression {
    fn _sort_impl(
        exp: &[Expression],
        env: &Environment,
        kind: SortKind,
        v: &mut [Expression],
    ) -> Result<(), Error> {
        if exp.len() == 2 {
            match &kind {
                SortKind::Stable(_) => v.sort(),
                SortKind::Unstable(_) => v.sort_unstable(),
            };
            Ok(())
        } else {
            let func = eval(&exp[2], env)?;
            match func {
                Expression::BuildInFunction(ref s, _) => match &s[..] {
                    "string>?" | "string>=?" => {
                        return {
                            v.sort_by(|a, b| b.cmp(a));
                            Ok(())
                        }
                    }
                    "string<?" | "string<=?" => {
                        return {
                            v.sort();
                            Ok(())
                        }
                    }
                    "char>?" | "char>=?" => {
                        return {
                            v.sort_by(|a, b| b.cmp(a));
                            Ok(())
                        }
                    }
                    "char<?" | "char<=?" => {
                        return {
                            v.sort();
                            Ok(())
                        }
                    }
                    ">=" | ">" => {
                        return {
                            v.sort_by(|a, b| b.cmp(a));
                            Ok(())
                        }
                    }
                    "<" | "<=" => {
                        return {
                            v.sort();
                            Ok(())
                        }
                    }
                    _ => {}
                },
                Expression::Function(_) => {}
                e => return Err(create_error_value!(ErrCode::E1006, e)),
            }
            let sort_by_impl = |a: &Expression, b: &Expression| {
                let v = vec![func.clone(), a.clone(), b.clone()];

                let e = match &func {
                    Expression::BuildInFunction(_, f) => f(&v, env),
                    Expression::Function(f) => f.execute(&v, env),
                    _ => Ok(Expression::Nil()),
                };
                if let Ok(Expression::Boolean(b)) = e {
                    return if b { Ordering::Less } else { Ordering::Greater };
                }
                Ordering::Less
            };
            match kind {
                SortKind::Stable(_) => v.sort_by(sort_by_impl),
                SortKind::Unstable(_) => v.sort_unstable_by(sort_by_impl),
            };
            Ok(())
        }
    }

    if 2 > exp.len() || 3 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let rc = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let effect = match &kind {
        SortKind::Stable(k) => k,
        SortKind::Unstable(k) => k,
    };
    match &effect {
        ListProcKind::Copy => {
            let l = &*(reference_obj!(rc));
            let mut v = Vec::new();
            v.extend_from_slice(&l[..]);
            _sort_impl(exp, env, kind, &mut v)?;
            Ok(Environment::create_list(v))
        }
        ListProcKind::Effect => {
            _sort_impl(exp, env, kind, &mut mut_obj!(rc))?;
            Ok(Expression::List(rc))
        }
    }
}
fn merge(exp: &[Expression], env: &Environment) -> ResultExpression {
    fn merge_iter(l1: &[Expression], l2: &[Expression]) -> Expression {
        let mut v = Vec::new();
        let (mut i, mut j) = (0, 0);

        loop {
            if l1.len() <= i || l2.len() <= j {
                break;
            }
            match l1[i].cmp(&l2[j]) {
                Ordering::Less | Ordering::Equal => {
                    v.push(l1[i].clone());
                    i += 1;
                }
                Ordering::Greater => {
                    v.push(l2[j].clone());
                    j += 1;
                }
            }
        }
        v.extend_from_slice(&l1[i..]);
        v.extend_from_slice(&l2[j..]);
        Environment::create_list(v)
    }
    fn merge_iter_by(
        l1: &[Expression],
        l2: &[Expression],
        env: &Environment,
        func: Expression,
    ) -> ResultExpression {
        let mut vec = Vec::new();
        let (mut i, mut j) = (0, 0);

        let mut ql = vec![func.clone()];

        loop {
            if l1.len() <= i || l2.len() <= j {
                break;
            }
            ql.push(l1[i].clone());
            ql.push(l2[j].clone());
            let result = match &func {
                Expression::BuildInFunction(_, f) => f(&ql, env),
                Expression::Function(f) => f.execute(&ql, env),
                e => return Err(create_error_value!(ErrCode::E1006, e)),
            };
            match result {
                Ok(e) => match e {
                    Expression::Boolean(b) => {
                        if b {
                            vec.push(l1[i].clone());
                            i += 1;
                        } else {
                            vec.push(l2[j].clone());
                            j += 1;
                        }
                    }
                    e => return Err(create_error_value!(ErrCode::E1001, e)),
                },
                Err(e) => return Err(e),
            };
            ql.pop();
            ql.pop();
        }
        vec.extend_from_slice(&l1[i..]);
        vec.extend_from_slice(&l2[j..]);
        Ok(Environment::create_list(vec))
    }

    if 3 > exp.len() || 4 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l1 = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l1 = &*(reference_obj!(l1));

    let l2 = match eval(&exp[2], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l2 = &*(reference_obj!(l2));

    if exp.len() == 4 {
        let func = eval(&exp[3], env)?;
        match func {
            Expression::BuildInFunction(_, _) => {}
            Expression::Function(_) => {}
            e => return Err(create_error_value!(ErrCode::E1006, e)),
        }
        merge_iter_by(l1, l2, env, func)
    } else {
        Ok(merge_iter(l1, l2))
    }
}
fn is_sorted(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 > exp.len() || 3 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let l = &*(reference_obj!(l));
    if exp.len() == 2 {
        let b = &l[..].windows(2).all(|w| w[0] <= w[1]);
        Ok(Expression::Boolean(*b))
    } else {
        let func = eval(&exp[2], env)?;
        let cmp = |a: &Expression, b: &Expression| {
            let v = vec![func.clone(), a.clone(), b.clone()];

            let e = match &func {
                Expression::BuildInFunction(_, f) => f(&v, env),
                Expression::Function(f) => f.execute(&v, env),
                _ => return false,
            };
            match e {
                Ok(v) => match v {
                    Expression::Boolean(b) => b,
                    _ => false,
                },
                Err(_) => false,
            }
        };
        match func {
            Expression::BuildInFunction(ref s, _) => match &s[..] {
                "string>?" | "string>=?" | "char>?" | "char>=?" | ">=" | ">" => {
                    let b = &l[..].windows(2).all(|w| w[0] >= w[1]);
                    Ok(Expression::Boolean(*b))
                }
                "string<?" | "string<=?" | "char<?" | "char<=?" | "<" | "<=" => {
                    let b = &l[..].windows(2).all(|w| w[0] <= w[1]);
                    Ok(Expression::Boolean(*b))
                }
                _ => {
                    let b = &l[..].windows(2).all(|w| cmp(&w[0], &w[1]));
                    Ok(Expression::Boolean(*b))
                }
            },
            Expression::Function(_) => {
                let b = &l[..].windows(2).all(|w| cmp(&w[0], &w[1]));
                Ok(Expression::Boolean(*b))
            }
            e => Err(create_error_value!(ErrCode::E1006, e)),
        }
    }
}
fn vector(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = seq(exp, env)?;
    Ok(Environment::create_vector(l))
}
fn make_vector(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = make_seq(exp, env)?;
    Ok(Environment::create_vector(l))
}
fn vector_length(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_length(exp, env, ErrCode::E1022)
}
fn list_vector(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::List(l) => Ok(Expression::Vector(l)),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn vector_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Vector(l) => Ok(Expression::List(l)),
        e => Err(create_error_value!(ErrCode::E1022, e)),
    }
}
fn vector_append(exp: &[Expression], env: &Environment) -> ResultExpression {
    let v = seq_append(exp, env, ErrCode::E1022)?;
    Ok(Environment::create_vector(v))
}
fn vector_append_effect(exp: &[Expression], env: &Environment) -> ResultExpression {
    let v = seq_append_effect(exp, env, ErrCode::E1022)?;
    Ok(Expression::Vector(v))
}
fn vector_ref(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_list_ref(exp, env, ErrCode::E1022)
}
fn vector_set(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_list_set(exp, env, ErrCode::E1022)
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn list() {
        assert_eq!(do_lisp("(list 1 2)"), "(1 2)");
        assert_eq!(do_lisp("(list 0.5 1)"), "(0.5 1)");
        assert_eq!(do_lisp("(list #t #f)"), "(#t #f)");
        assert_eq!(do_lisp("(list (list 1)(list 2))"), "((1) (2))");
        assert_eq!(
            do_lisp("(list (list (list 1))(list 2)(list 3))"),
            "(((1)) (2) (3))"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 10)", &env);
        do_lisp_env("(define b 20)", &env);
        assert_eq!(do_lisp_env("(list a b)", &env), "(10 20)");
    }
    #[test]
    fn make_list() {
        assert_eq!(do_lisp("(make-list 10 0)"), "(0 0 0 0 0 0 0 0 0 0)");
        assert_eq!(
            do_lisp("(make-list 4 (list 1 2 3))"),
            "((1 2 3) (1 2 3) (1 2 3) (1 2 3))"
        );
        assert_eq!(do_lisp("(make-list 8 'a)"), "(a a a a a a a a)");
        assert_eq!(do_lisp("(make-list 0 'a)"), "()");
    }
    #[test]
    fn null_f() {
        assert_eq!(do_lisp("(null? (list))"), "#t");
        assert_eq!(do_lisp("(null? (list 10))"), "#f");
        assert_eq!(do_lisp("(null? 10)"), "#f");
    }
    #[test]
    fn length() {
        assert_eq!(do_lisp("(length (list))"), "0");
        assert_eq!(do_lisp("(length (list 3))"), "1");
        assert_eq!(do_lisp("(length (iota 10))"), "10");
    }
    #[test]
    fn car() {
        assert_eq!(do_lisp("(car (list 1))"), "1");
        assert_eq!(do_lisp("(car (list (list 2)))"), "(2)");
        assert_eq!(
            do_lisp("(car (list (list (list 1))(list 2)(list 3)))"),
            "((1))"
        );
        assert_eq!(do_lisp("(car (cons 10 20))"), "10");
    }
    #[test]
    fn cdr() {
        assert_eq!(do_lisp("(cdr (list 1 2))"), "(2)");
        assert_eq!(do_lisp("(cdr (list 1 0.5))"), "(0.5)");
        assert_eq!(do_lisp("(cdr (list 1 (list 3)))"), "((3))");
        assert_eq!(do_lisp("(cdr (cons 1 2))"), "2");
        assert_eq!(do_lisp("(cdr (list 1))"), "()");
    }
    #[test]
    fn cadr() {
        assert_eq!(do_lisp("(cadr (list 1 2))"), "2");
        assert_eq!(do_lisp("(cadr (list 1 2 3))"), "2");
    }
    #[test]
    fn cons() {
        assert_eq!(do_lisp("(cons  1 2)"), "(1 . 2)");
        assert_eq!(do_lisp("(cons 1.5 2.5)"), "(1.5 . 2.5)");
        assert_eq!(do_lisp("(cons  1 1.5)"), "(1 . 1.5)");
        assert_eq!(do_lisp("(cons 1 (list 2))"), "(1 2)");
        assert_eq!(do_lisp("(cons (list 1)(list 2))"), "((1) 2)");

        let env = lisp::Environment::new();
        do_lisp_env("(define a (iota 10))", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(do_lisp_env("(cons #t a)", &env), "(#t 0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp_env("a", &env), "(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp_env("b", &env), "(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn append() {
        assert_eq!(do_lisp("(append (list 1)(list 2))"), "(1 2)");
        assert_eq!(do_lisp("(append (list 1)(list 2)(list 3))"), "(1 2 3)");
        assert_eq!(
            do_lisp("(append (list (list 10))(list 2)(list 3))"),
            "((10) 2 3)"
        );
        assert_eq!(do_lisp("(append (iota 5) (list 100))"), "(0 1 2 3 4 100)");

        let env = lisp::Environment::new();
        do_lisp_env("(define a (iota 5))", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(
            do_lisp_env("(append (iota 5 5) a)", &env),
            "(5 6 7 8 9 0 1 2 3 4)"
        );
        assert_eq!(do_lisp_env("a", &env), "(0 1 2 3 4)");
        assert_eq!(do_lisp_env("b", &env), "(0 1 2 3 4)");
    }
    #[test]
    fn append_effect() {
        assert_eq!(do_lisp("(append! (list 1)(list 2))"), "(1 2)");
        assert_eq!(do_lisp("(append! (list 1)(list 2)(list 3))"), "(1 2 3)");
        assert_eq!(
            do_lisp("(append! (list (list 10))(list 2)(list 3))"),
            "((10) 2 3)"
        );
        assert_eq!(do_lisp("(append! (iota 5) (list 100))"), "(0 1 2 3 4 100)");

        let env = lisp::Environment::new();
        do_lisp_env("(define a (iota 5))", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(
            do_lisp_env("(append! a (iota 5 5))", &env),
            "(0 1 2 3 4 5 6 7 8 9)"
        );
        assert_eq!(do_lisp_env("a", &env), "(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp_env("b", &env), "(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn take() {
        assert_eq!(do_lisp("(take (iota 10) 0)"), "()");
        assert_eq!(do_lisp("(take (iota 10) 1)"), "(0)");
        assert_eq!(do_lisp("(take (iota 10) 3)"), "(0 1 2)");
        assert_eq!(do_lisp("(take (iota 10) 9)"), "(0 1 2 3 4 5 6 7 8)");
        assert_eq!(do_lisp("(take (iota 10) 10)"), "(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn drop() {
        assert_eq!(do_lisp("(drop (iota 10) 0)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 1)"), "(1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 3)"), "(3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 9)"), "(9)");
        assert_eq!(do_lisp("(drop (iota 10) 10)"), "()");
    }
    #[test]
    fn delete() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 10 10.5 3/5 \"ABC\" #\\a #t))", &env);
        assert_eq!(
            do_lisp_env("(delete 10 a)", &env),
            "(10.5 3/5 \"ABC\" #\\a #t)"
        );
        assert_eq!(
            do_lisp_env("(delete 10.5 a)", &env),
            "(10 3/5 \"ABC\" #\\a #t)"
        );
        assert_eq!(
            do_lisp_env("(delete 3/5 a)", &env),
            "(10 10.5 \"ABC\" #\\a #t)"
        );
        assert_eq!(
            do_lisp_env("(delete \"ABC\" a)", &env),
            "(10 10.5 3/5 #\\a #t)"
        );
        assert_eq!(
            do_lisp_env("(delete #\\a a)", &env),
            "(10 10.5 3/5 \"ABC\" #t)"
        );
        assert_eq!(
            do_lisp_env("(delete #t a)", &env),
            "(10 10.5 3/5 \"ABC\" #\\a)"
        );
    }
    #[test]
    fn delete_effect() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 10 10.5 3/5 \"ABC\" #\\a #t))", &env);

        do_lisp_env("(delete! 10 a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(10.5 3/5 \"ABC\" #\\a #t)");

        assert_eq!(
            do_lisp_env("(delete! 10.5 a)", &env),
            "(3/5 \"ABC\" #\\a #t)"
        );
        assert_eq!(do_lisp_env("(delete! 3/5 a)", &env), "(\"ABC\" #\\a #t)");

        do_lisp_env("(delete! \"ABC\" a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(#\\a #t)");

        do_lisp_env("(delete! #\\a a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(#t)");

        do_lisp_env("(delete! #f a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(#t)");

        do_lisp_env("(delete! #t a)", &env);
        assert_eq!(do_lisp_env("a", &env), "()");
    }
    #[test]
    fn last() {
        assert_eq!(do_lisp("(last (list 1))"), "1");
        assert_eq!(do_lisp("(last (list 1 2))"), "2");
        assert_eq!(do_lisp("(last (cons 1 2))"), "1");
    }
    #[test]
    fn reverse() {
        assert_eq!(do_lisp("(reverse (list 10))"), "(10)");
        assert_eq!(do_lisp("(reverse (iota 10))"), "(9 8 7 6 5 4 3 2 1 0)");
        assert_eq!(do_lisp("(reverse (list))"), "()");
    }
    #[test]
    fn iota() {
        assert_eq!(do_lisp("(iota 10)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(iota 10 1)"), "(1 2 3 4 5 6 7 8 9 10)");
        assert_eq!(do_lisp("(iota 1 10)"), "(10)");
        assert_eq!(do_lisp("(iota 10 1 2)"), "(1 3 5 7 9 11 13 15 17 19)");
        assert_eq!(do_lisp("(iota 10 1 -1)"), "(1 0 -1 -2 -3 -4 -5 -6 -7 -8)");
        assert_eq!(do_lisp("(iota -10 0 1)"), "()");
    }
    #[test]
    fn map() {
        assert_eq!(
            do_lisp("(map (lambda (n) (* n 10)) (iota 10 1))"),
            "(10 20 30 40 50 60 70 80 90 100)"
        );
        assert_eq!(do_lisp("(map list (list 1 2 3))"), "((1) (2) (3))");
        assert_eq!(do_lisp("(map (lambda (n) (car n)) (list))"), "()");
        assert_eq!(
            do_lisp("(map car (list (list 1 2 3)(list 4 5 6) (list 7 8 9)))"),
            "(1 4 7)"
        );
        assert_eq!(
            do_lisp("(map (lambda (n) (car n)) (list (list 1)(list 2)(list 3)))"),
            "(1 2 3)"
        );
        assert_eq!(
            do_lisp("(map (lambda (n) (car n)) (list (list (list 1))(list 2)(list 3)))"),
            "((1) 2 3)"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        do_lisp_env(
            "(define d (list (list (list 1))(list (list 2))(list (list 3))))",
            &env,
        );

        assert_eq!(
            do_lisp_env(
                "(map (lambda (n)(map (lambda (m)(/ m 10)) n))(list (list 10 20 30)(list a b c)))",
                &env
            ),
            "((1 2 3) (10 20 30))"
        );
        assert_eq!(
            do_lisp_env("(map (lambda (n) (car n)) d)", &env),
            "((1) (2) (3))"
        );
    }
    #[test]
    fn filter() {
        assert_eq!(
            do_lisp("(filter (lambda (n) (= 0 (modulo n 2))) (iota 10 1))"),
            "(2 4 6 8 10)"
        );
        assert_eq!(
            do_lisp("(filter (lambda (n) (not (= 0 (modulo n 2)))) (iota 10 1))"),
            "(1 3 5 7 9)"
        );
        assert_eq!(do_lisp("(filter odd? (iota 10))",), "(1 3 5 7 9)");
        assert_eq!(
            do_lisp("(filter integer? (list (list 1 2 3) #f 10))"),
            "(10)"
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        assert_eq!(
            do_lisp_env("(filter (lambda (n) (= n 100)) (list a b c))", &env),
            "(100)"
        );
        assert_eq!(
            do_lisp_env("(filter (lambda (n) (not (= n 100))) (list a b c))", &env),
            "(200 300)"
        );
    }
    #[test]
    fn reduce() {
        assert_eq!(do_lisp("(reduce (lambda (a b) (+ a b))0(list 1 2 3))"), "6");
        assert_eq!(
            do_lisp("(reduce (lambda (a b) (append a b))(list)(list (list 1) (list 2) (list 3)))"),
            "(1 2 3)"
        );
        assert_eq!(
            do_lisp("(reduce (lambda (a b) (+ a b))(* 10 10)(list))"),
            "100"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        assert_eq!(
            do_lisp_env("(reduce (lambda (a b) (+ a b))0(list a b c))", &env),
            "600"
        );
        assert_eq!(do_lisp("(reduce 0 (list) (list))"), "()");
        assert_eq!(do_lisp("(reduce + 10 (list 1 2 3))"), "6");
    }
    #[test]
    fn for_each() {
        let env = lisp::Environment::new();
        do_lisp_env("(define c 0)", &env);
        do_lisp_env("(for-each (lambda (n) (set! c (+ c n)))(iota 5))", &env);
        assert_eq!(do_lisp_env("c", &env), "10");
    }
    #[test]
    fn list_ref() {
        assert_eq!(do_lisp("(list-ref (iota 10) 0)"), "0");
        assert_eq!(do_lisp("(list-ref (iota 10) 1)"), "1");
        assert_eq!(do_lisp("(list-ref (iota 10) 8)"), "8");
        assert_eq!(do_lisp("(list-ref (iota 10) 9)"), "9");
        assert_eq!(do_lisp("(list-ref '(#\\a #\\b #\\c) 1)"), "#\\b");
        assert_eq!(do_lisp("(list-ref (list (list 0 1) 1 2 3) 0)"), "(0 1)");
    }
    #[test]
    fn list_set() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 1 2 3 4 5))", &env);
        do_lisp_env("(define b a)", &env);
        do_lisp_env("(list-set! a 0 100)", &env);
        assert_eq!(do_lisp_env("a", &env), "(100 2 3 4 5)");
        assert_eq!(do_lisp_env("b", &env), "(100 2 3 4 5)");
    }
    #[test]
    fn set_car() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 1 2 3 4 5))", &env);
        do_lisp_env("(set-car! a 100)", &env);
        assert_eq!(do_lisp_env("a", &env), "(100 2 3 4 5)");

        do_lisp_env("(set-car! a (list 10 20))", &env);
        assert_eq!(do_lisp_env("a", &env), "((10 20) 2 3 4 5)");
    }
    #[test]
    fn set_cdr() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 1 2 3 4 5))", &env);
        do_lisp_env("(set-cdr! a 100)", &env);
        assert_eq!(do_lisp_env("a", &env), "(1 100)");

        do_lisp_env("(set-cdr! a (list 10 20))", &env);
        assert_eq!(do_lisp_env("a", &env), "(1 10 20)");
    }
    #[test]
    fn sort() {
        assert_eq!(
            do_lisp("(sort (list 10 1 9 5 3 4 7 6 5))"),
            "(1 3 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1.5 9 5 2/3 4 7 6 5))"),
            "(2/3 1.5 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(sort (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\"))"),
            "(\"0\" \"A\" \"a\" \"b\" \"c\" \"d\" \"l\" \"m\" \"z\")"
        );
        assert_eq!(
            do_lisp("(sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))"),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );
        assert_eq!(
            do_lisp("(sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0))"),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1 9 5 3 4 7 6 5) <)"),
            "(1 3 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1 9 5 3 4 7 6 5) >)"),
            "(10 9 7 6 5 5 4 3 1)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1 9 5 3 4 7 6 5) <=)"),
            "(1 3 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1 9 5 3 4 7 6 5) >=)"),
            "(10 9 7 6 5 5 4 3 1)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1.5 9 5 2/3 4 7 6 5) >)"),
            "(10 9 7 6 5 5 4 1.5 2/3)"
        );
        assert_eq!(
            do_lisp("(sort (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\") string>?)"),
            "(\"z\" \"m\" \"l\" \"d\" \"c\" \"b\" \"a\" \"A\" \"0\")"
        );
        assert_eq!(
            do_lisp("(sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\") string>?)"),
            "(\"CA\" \"BB\" \"AZ\" \"AB\" \"AA\")"
        );
        assert_eq!(
            do_lisp("(sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\") string>=?)"),
            "(\"CA\" \"BB\" \"AZ\" \"AB\" \"AA\")"
        );
        assert_eq!(
            do_lisp("(sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\") string<?)"),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );
        assert_eq!(
            do_lisp("(sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\") string<=?)"),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );
        assert_eq!(
            do_lisp("(sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) char>?)"),
            "(#\\z #\\m #\\l #\\d #\\c #\\b #\\a #\\A #\\0)"
        );
        assert_eq!(
            do_lisp("(sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) char>=?)"),
            "(#\\z #\\m #\\l #\\d #\\c #\\b #\\a #\\A #\\0)"
        );
        assert_eq!(
            do_lisp("(sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) char<?)"),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );
        assert_eq!(
            do_lisp("(sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) char<=?)"),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );
        assert_eq!(
            do_lisp("(sort (list 10 1.5 9 5 2/3 4 7 6 5) (lambda (a b) (> a b)))"),
            "(10 9 7 6 5 5 4 1.5 2/3)"
        );
    }
    #[test]
    fn sort_effect() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 10 1 9 5 3 4 7 6 5))", &env);
        do_lisp_env("(sort! a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(1 3 4 5 5 6 7 9 10)");

        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(sort! a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(2/3 1.5 4 5 5 6 7 9 10)");

        do_lisp_env(
            "(define a (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\"))",
            &env,
        );
        do_lisp_env("(sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"0\" \"A\" \"a\" \"b\" \"c\" \"d\" \"l\" \"m\" \"z\")"
        );

        do_lisp_env("(define a (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))", &env);
        do_lisp_env("(sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );

        do_lisp_env(
            "(define a (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0))",
            &env,
        );
        do_lisp_env("(sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );

        do_lisp_env("(define a (list 10 1 9 5 3 4 7 6 5))", &env);
        do_lisp_env("(sort! a >)", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 3 1)");

        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(sort! a >)", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 1.5 2/3)");

        do_lisp_env(
            "(define a (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\") )",
            &env,
        );
        do_lisp_env("(sort! a string>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"z\" \"m\" \"l\" \"d\" \"c\" \"b\" \"a\" \"A\" \"0\")"
        );

        do_lisp_env("(define a (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))", &env);
        do_lisp_env("(sort! a string>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"CA\" \"BB\" \"AZ\" \"AB\" \"AA\")"
        );

        do_lisp_env(
            "(define a (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) )",
            &env,
        );
        do_lisp_env("(sort! a char>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(#\\z #\\m #\\l #\\d #\\c #\\b #\\a #\\A #\\0)"
        );

        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(sort! a (lambda (a b)(> a b)))", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 1.5 2/3)");
    }
    #[test]
    fn sort_stable() {
        assert_eq!(
            do_lisp("(stable-sort (list 10 1 9 5 3 4 7 6 5))"),
            "(1 3 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list 10 1.5 9 5 2/3 4 7 6 5))"),
            "(2/3 1.5 4 5 5 6 7 9 10)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\"))"),
            "(\"0\" \"A\" \"a\" \"b\" \"c\" \"d\" \"l\" \"m\" \"z\")"
        );
        assert_eq!(
            do_lisp("(stable-sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))"),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );
        assert_eq!(
            do_lisp("(stable-sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0))"),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list 10 1 9 5 3 4 7 6 5) >)"),
            "(10 9 7 6 5 5 4 3 1)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list 10 1.5 9 5 2/3 4 7 6 5) >)"),
            "(10 9 7 6 5 5 4 1.5 2/3)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\") string>?)"),
            "(\"z\" \"m\" \"l\" \"d\" \"c\" \"b\" \"a\" \"A\" \"0\")"
        );
        assert_eq!(
            do_lisp("(stable-sort (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\") string>?)"),
            "(\"CA\" \"BB\" \"AZ\" \"AB\" \"AA\")"
        );
        assert_eq!(
            do_lisp("(stable-sort (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) char>?)"),
            "(#\\z #\\m #\\l #\\d #\\c #\\b #\\a #\\A #\\0)"
        );
        assert_eq!(
            do_lisp("(stable-sort (list 10 1.5 9 5 2/3 4 7 6 5) (lambda (a b) (> a b)))"),
            "(10 9 7 6 5 5 4 1.5 2/3)"
        );
    }
    #[test]
    fn sort_stable_effect() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 10 1 9 5 3 4 7 6 5))", &env);
        do_lisp_env("(stable-sort! a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(1 3 4 5 5 6 7 9 10)");

        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(stable-sort! a)", &env);
        assert_eq!(do_lisp_env("a", &env), "(2/3 1.5 4 5 5 6 7 9 10)");

        do_lisp_env(
            "(define a (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\"))",
            &env,
        );
        do_lisp_env("(stable-sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"0\" \"A\" \"a\" \"b\" \"c\" \"d\" \"l\" \"m\" \"z\")"
        );

        do_lisp_env("(define a (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))", &env);
        do_lisp_env("(stable-sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"AA\" \"AB\" \"AZ\" \"BB\" \"CA\")"
        );

        do_lisp_env(
            "(define a (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0))",
            &env,
        );
        do_lisp_env("(stable-sort! a)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(#\\0 #\\A #\\a #\\b #\\c #\\d #\\l #\\m #\\z)"
        );

        do_lisp_env("(define a (list 10 1 9 5 3 4 7 6 5))", &env);
        do_lisp_env("(stable-sort! a >)", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 3 1)");

        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(stable-sort! a >)", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 1.5 2/3)");

        do_lisp_env(
            "(define a (list \"z\" \"a\" \"b\" \"m\" \"l\" \"d\" \"A\" \"c\" \"0\") )",
            &env,
        );
        do_lisp_env("(stable-sort! a string>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"z\" \"m\" \"l\" \"d\" \"c\" \"b\" \"a\" \"A\" \"0\")"
        );

        do_lisp_env("(define a (list \"AZ\" \"AA\" \"AB\" \"CA\" \"BB\"))", &env);
        do_lisp_env("(stable-sort! a string>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(\"CA\" \"BB\" \"AZ\" \"AB\" \"AA\")"
        );

        do_lisp_env(
            "(define a (list #\\z #\\a #\\b #\\m #\\l #\\d #\\A #\\c #\\0) )",
            &env,
        );
        do_lisp_env("(stable-sort! a char>?)", &env);
        assert_eq!(
            do_lisp_env("a", &env),
            "(#\\z #\\m #\\l #\\d #\\c #\\b #\\a #\\A #\\0)"
        );
        do_lisp_env("(define a (list 10 1.5 9 5 2/3 4 7 6 5))", &env);
        do_lisp_env("(stable-sort! a (lambda (a b)(> a b)))", &env);
        assert_eq!(do_lisp_env("a", &env), "(10 9 7 6 5 5 4 1.5 2/3)");
    }
    #[test]
    fn merge() {
        assert_eq!(
            do_lisp("(merge (iota 10 1 2) (iota 10 2 2))"),
            "(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20)"
        );
        assert_eq!(
            do_lisp("(merge (list 1/3 1 2) (list 1/2 3/2 1.75))"),
            "(1/3 1/2 1 3/2 1.75 2)"
        );
        assert_eq!(
            do_lisp("(merge (list \"a\" \"c\" \"e\" \"g\")(list \"b\" \"d\" \"f\" \"h\"))"),
            "(\"a\" \"b\" \"c\" \"d\" \"e\" \"f\" \"g\" \"h\")"
        );
        assert_eq!(
            do_lisp("(merge (list #\\a #\\c #\\e #\\g)(list #\\b #\\d #\\f #\\h))"),
            "(#\\a #\\b #\\c #\\d #\\e #\\f #\\g #\\h)"
        );
        assert_eq!(
            do_lisp("(merge (reverse (iota 10 1 2)) (reverse (iota 10 2 2)) >)"),
            "(20 19 18 17 16 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1)"
        );
        assert_eq!(
            do_lisp(
                "(merge (list \"g\" \"e\" \"c\" \"a\")(list \"h\" \"f\" \"d\" \"b\") string>?)"
            ),
            "(\"h\" \"g\" \"f\" \"e\" \"d\" \"c\" \"b\" \"a\")"
        );
        assert_eq!(
            do_lisp("(merge (list #\\g #\\e #\\c #\\a)(list #\\h #\\f #\\d #\\b) char>?)"),
            "(#\\h #\\g #\\f #\\e #\\d #\\c #\\b #\\a)"
        );
    }
    #[test]
    fn is_sorted() {
        assert_eq!(do_lisp("(sorted? (list 1 2 3))"), "#t");
        assert_eq!(do_lisp("(sorted? (list 1 2 3) <)"), "#t");
        assert_eq!(do_lisp("(sorted? (list 3 2 1) >)"), "#t");
        assert_eq!(do_lisp("(sorted? (list 1 2 3) >)"), "#f");
        assert_eq!(do_lisp("(sorted? (list 3 2 1) <)"), "#f");
        assert_eq!(
            do_lisp("(sorted? (list 1 2 3) (lambda (a b)(< a b)))"),
            "#t"
        );
        assert_eq!(
            do_lisp("(sorted? (list 1 2 3) (lambda (a b)(> a b)))"),
            "#f"
        );
        assert_eq!(do_lisp("(sorted? (list \"a\" \"b\" \"c\") string<?)"), "#t");
        assert_eq!(do_lisp("(sorted? (list \"c\" \"b\" \"a\") string>?)"), "#t");
        assert_eq!(do_lisp("(sorted? (list #\\a #\\b #\\c) char<?)"), "#t");
        assert_eq!(do_lisp("(sorted? (list #\\c #\\b #\\a) char>?)"), "#t");
        assert_eq!(do_lisp("(sorted? (list #\\a #\\b #\\c) char-ci<?)"), "#t");
        assert_eq!(do_lisp("(sorted? (list #\\c #\\b #\\a) char-ci>?)"), "#t");
    }
    #[test]
    fn vector() {
        assert_eq!(do_lisp("#(1 2)"), "#(1 2)");
        assert_eq!(do_lisp("(vector 1 2)"), "#(1 2)");
        assert_eq!(do_lisp("(vector 0.5 1)"), "#(0.5 1)");
        assert_eq!(do_lisp("(vector #t #f)"), "#(#t #f)");
        assert_eq!(do_lisp("(vector (list 1)(list 2))"), "#((1) (2))");
        assert_eq!(
            do_lisp("(vector (vector (vector 1))(vector 2)(vector 3))"),
            "#(#(#(1)) #(2) #(3))"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 10)", &env);
        do_lisp_env("(define b 20)", &env);
        assert_eq!(do_lisp_env("(vector a b)", &env), "#(10 20)");
    }
    #[test]
    fn make_vector() {
        assert_eq!(do_lisp("(make-vector 10 0)"), "#(0 0 0 0 0 0 0 0 0 0)");
        assert_eq!(
            do_lisp("(make-vector 4 (list 1 2 3))"),
            "#((1 2 3) (1 2 3) (1 2 3) (1 2 3))"
        );
        assert_eq!(do_lisp("(make-vector 8 'a)"), "#(a a a a a a a a)");
        assert_eq!(do_lisp("(make-vector 0 'a)"), "#()");
    }
    #[test]
    fn vector_length() {
        assert_eq!(do_lisp("(vector-length (vector))"), "0");
        assert_eq!(do_lisp("(vector-length (vector 3))"), "1");
        assert_eq!(do_lisp("(vector-length (list->vector (iota 10)))"), "10");
    }
    #[test]
    fn vector_list() {
        assert_eq!(do_lisp("(vector->list #(1 2 3))"), "(1 2 3)");
        assert_eq!(do_lisp("(vector->list (vector 1 2 3))"), "(1 2 3)");
        assert_eq!(do_lisp("(vector->list #())"), "()");
    }
    #[test]
    fn list_vector() {
        assert_eq!(do_lisp("(list->vector '(1 2 3))"), "#(1 2 3)");
        assert_eq!(do_lisp("(list->vector (list 1 2 3))"), "#(1 2 3)");
        assert_eq!(do_lisp("(list->vector '())"), "#()");
    }
    #[test]
    fn vector_append() {
        assert_eq!(do_lisp("(vector-append (vector 1)(vector 2))"), "#(1 2)");
        assert_eq!(
            do_lisp("(vector-append (vector 1)(vector 2)(vector 3))"),
            "#(1 2 3)"
        );
        assert_eq!(
            do_lisp("(vector-append (vector (vector 10))(vector 2)(vector 3))"),
            "#(#(10) 2 3)"
        );
        assert_eq!(
            do_lisp("(vector-append (list->vector (iota 5)) (vector 100))"),
            "#(0 1 2 3 4 100)"
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (list->vector (iota 5)))", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(
            do_lisp_env("(vector-append (list->vector(iota 5 5)) a)", &env),
            "#(5 6 7 8 9 0 1 2 3 4)"
        );
        assert_eq!(do_lisp_env("a", &env), "#(0 1 2 3 4)");
        assert_eq!(do_lisp_env("b", &env), "#(0 1 2 3 4)");
    }
    #[test]
    fn vector_append_effect() {
        assert_eq!(do_lisp("(vector-append! (vector 1)(vector 2))"), "#(1 2)");
        assert_eq!(
            do_lisp("(vector-append! (vector 1)(vector 2)(vector 3))"),
            "#(1 2 3)"
        );
        assert_eq!(
            do_lisp("(vector-append! (vector (vector 10))(vector 2)(vector 3))"),
            "#(#(10) 2 3)"
        );
        assert_eq!(
            do_lisp("(vector-append! (list->vector (iota 5)) (vector 100))"),
            "#(0 1 2 3 4 100)"
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (list->vector(iota 5)))", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(
            do_lisp_env("(vector-append! a (list->vector (iota 5 5)))", &env),
            "#(0 1 2 3 4 5 6 7 8 9)"
        );
        assert_eq!(do_lisp_env("a", &env), "#(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp_env("b", &env), "#(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn vector_ref() {
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) 0)"), "0");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) 1)"), "1");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) 8)"), "8");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) 9)"), "9");
        assert_eq!(do_lisp("(vector-ref #(#\\a #\\b #\\c) 1)"), "#\\b");
        assert_eq!(do_lisp("(vector-ref #((vector 0 1) 1 2 3) 0)"), "#(0 1)");
    }
    #[test]
    fn vector_set() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a #(1 2 3 4 5))", &env);
        do_lisp_env("(define b a)", &env);
        do_lisp_env("(vector-set! a 0 100)", &env);
        assert_eq!(do_lisp_env("a", &env), "#(100 2 3 4 5)");
        assert_eq!(do_lisp_env("b", &env), "#(100 2 3 4 5)");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;
    #[test]
    fn list() {
        assert_eq!(do_lisp("(list c 10)"), "E1008");
    }
    #[test]
    fn make_list() {
        assert_eq!(do_lisp("(make-list)"), "E1007");
        assert_eq!(do_lisp("(make-list 10)"), "E1007");
        assert_eq!(do_lisp("(make-list 10 0 1)"), "E1007");
        assert_eq!(do_lisp("(make-list #t 0)"), "E1002");
        assert_eq!(do_lisp("(make-list -1 0)"), "E1011");
        assert_eq!(do_lisp("(make-list 10 c)"), "E1008");
    }
    #[test]
    fn null_f() {
        assert_eq!(do_lisp("(null?)"), "E1007");
        assert_eq!(do_lisp("(null? (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(null? c)"), "E1008");
    }
    #[test]
    fn length() {
        assert_eq!(do_lisp("(length)"), "E1007");
        assert_eq!(do_lisp("(length (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(length (cons 1 2))"), "E1005");
        assert_eq!(do_lisp("(length (vector 1 2))"), "E1005");
        assert_eq!(do_lisp("(length a)"), "E1008");
    }
    #[test]
    fn car() {
        assert_eq!(do_lisp("(car)"), "E1007");
        assert_eq!(do_lisp("(car (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(car l)"), "E1008");
        assert_eq!(do_lisp("(car (list))"), "E1011");
        assert_eq!(do_lisp("(car 10)"), "E1005");
    }
    #[test]
    fn cdr() {
        assert_eq!(do_lisp("(cdr)"), "E1007");
        assert_eq!(do_lisp("(cdr (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(cdr (list c))"), "E1008");
        assert_eq!(do_lisp("(cdr (list))"), "E1011");
        assert_eq!(do_lisp("(cdr 200)"), "E1005");
    }
    #[test]
    fn cadr() {
        assert_eq!(do_lisp("(cadr)"), "E1007");
        assert_eq!(do_lisp("(cadr (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(cadr c)"), "E1008");
        assert_eq!(do_lisp("(cadr (list 1))"), "E1011");
        assert_eq!(do_lisp("(cadr 991)"), "E1005");
    }
    #[test]
    fn cons() {
        assert_eq!(do_lisp("(cons)"), "E1007");
        assert_eq!(do_lisp("(cons (list 1)(list 2)(list 3))"), "E1007");
        assert_eq!(do_lisp("(cons a 10)"), "E1008");
    }
    #[test]
    fn append() {
        assert_eq!(do_lisp("(append)"), "E1007");
        assert_eq!(do_lisp("(append 10)"), "E1005");
        assert_eq!(do_lisp("(append (list 1) 105)"), "E1005");
        assert_eq!(do_lisp("(append (list 1) a)"), "E1008");
    }
    #[test]
    fn append_effect() {
        assert_eq!(do_lisp("(append!)"), "E1007");
        assert_eq!(do_lisp("(append! 10)"), "E1005");
        assert_eq!(do_lisp("(append! (list 1) 105)"), "E1005");
        assert_eq!(do_lisp("(append! (list 1) a)"), "E1008");
    }
    #[test]
    fn take() {
        assert_eq!(do_lisp("(take)"), "E1007");
        assert_eq!(do_lisp("(take (list 10 20))"), "E1007");
        assert_eq!(do_lisp("(take (list 10 20) 1 2)"), "E1007");
        assert_eq!(do_lisp("(take 1 (list 1 2))"), "E1005");
        assert_eq!(do_lisp("(take (list 1 2) 10.5)"), "E1002");
        assert_eq!(do_lisp("(take (list 1 2) 3)"), "E1011");
        assert_eq!(do_lisp("(take (list 1 2) -1)"), "E1011");
        assert_eq!(do_lisp("(take a 1)"), "E1008");
    }
    #[test]
    fn drop() {
        assert_eq!(do_lisp("(drop)"), "E1007");
        assert_eq!(do_lisp("(drop (list 10 20))"), "E1007");
        assert_eq!(do_lisp("(drop (list 10 20) 1 2)"), "E1007");
        assert_eq!(do_lisp("(drop 1 (list 1 2))"), "E1005");
        assert_eq!(do_lisp("(drop (list 1 2) 10.5)"), "E1002");
        assert_eq!(do_lisp("(drop (list 1 2) 3)"), "E1011");
        assert_eq!(do_lisp("(drop (list 1 2) -1)"), "E1011");
        assert_eq!(do_lisp("(drop a 1)"), "E1008");
    }
    #[test]
    fn delete() {
        assert_eq!(do_lisp("(delete)"), "E1007");
        assert_eq!(do_lisp("(delete 10)"), "E1007");
        assert_eq!(do_lisp("(delete 10 (list 10 20) 3)"), "E1007");
        assert_eq!(do_lisp("(delete 10 20)"), "E1005");
        assert_eq!(do_lisp("(delete 10 a)"), "E1008");
    }
    #[test]
    fn delete_effect() {
        assert_eq!(do_lisp("(delete!)"), "E1007");
        assert_eq!(do_lisp("(delete! 10)"), "E1007");
        assert_eq!(do_lisp("(delete! 10 (list 10 20) 3)"), "E1007");
        assert_eq!(do_lisp("(delete! 10 20)"), "E1005");
        assert_eq!(do_lisp("(delete! 10 a)"), "E1008");
    }
    #[test]
    fn last() {
        assert_eq!(do_lisp("(last)"), "E1007");
        assert_eq!(do_lisp("(last (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(last (list))"), "E1011");
        assert_eq!(do_lisp("(last 29)"), "E1005");
        assert_eq!(do_lisp("(last a)"), "E1008");
    }
    #[test]
    fn reverse() {
        assert_eq!(do_lisp("(reverse)"), "E1007");
        assert_eq!(do_lisp("(reverse (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(reverse 29)"), "E1005");
        assert_eq!(do_lisp("(reverse a)"), "E1008");
    }
    #[test]
    fn iota() {
        assert_eq!(do_lisp("(iota)"), "E1007");
        assert_eq!(do_lisp("(iota 1 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(iota 1.5 2)"), "E1002");
        assert_eq!(do_lisp("(iota 1 10.5)"), "E1002");
        assert_eq!(do_lisp("(iota 10 1 10.5)"), "E1002");
        assert_eq!(do_lisp("(iota a)"), "E1008");
    }
    #[test]
    fn map() {
        assert_eq!(do_lisp("(map)"), "E1007");
        assert_eq!(do_lisp("(map (lambda (n) n))"), "E1007");
        assert_eq!(
            do_lisp("(map (lambda (a b) (* 10 a)) (list 1 2 3))"),
            "E1007"
        );
        assert_eq!(do_lisp("(map 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(map (iota 10) (iota 10))"), "E1006");
        assert_eq!(do_lisp("(map  (lambda (n) n) 10)"), "E1005");
    }
    #[test]
    fn filter() {
        assert_eq!(do_lisp("(filter)"), "E1007");
        assert_eq!(do_lisp("(filter (lambda (n) n))"), "E1007");
        assert_eq!(do_lisp("(filter 1 2 3)"), "E1007");
        assert_eq!(
            do_lisp("(filter (lambda (a b) (= 0 a))(iota 10 1))"),
            "E1007"
        );
        assert_eq!(do_lisp("(filter (iota 10) (iota 10))"), "E1006");
        assert_eq!(do_lisp("(filter (lambda (n) n) 10)"), "E1005");
        assert_eq!(do_lisp("(filter (lambda (n) n) (iota 4))"), "E1001");
    }
    #[test]
    fn reduce() {
        assert_eq!(do_lisp("(reduce)"), "E1007");
        assert_eq!(do_lisp("(reduce (lambda (n) n))"), "E1007");
        assert_eq!(do_lisp("(reduce 1 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(reduce (lambda (n) n) 10 10)"), "E1005");
        assert_eq!(do_lisp("(reduce (lambda (n) n) 0 (iota 4))"), "E1007");
    }
    #[test]
    fn for_each() {
        assert_eq!(do_lisp("(for-each)"), "E1007");
        assert_eq!(do_lisp("(for-each (lambda (n) n))"), "E1007");
        assert_eq!(do_lisp("(for-each 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(for-each 10 (list 1))"), "E1006");
        assert_eq!(do_lisp("(for-each (lambda (n) n) 10)"), "E1005");
    }
    #[test]
    fn list_ref() {
        assert_eq!(do_lisp("(list-ref)"), "E1007");
        assert_eq!(do_lisp("(list-ref (iota 10))"), "E1007");
        assert_eq!(do_lisp("(list-ref (iota 10) 1 2)"), "E1007");
        assert_eq!(do_lisp("(list-ref 10 -1)"), "E1005");
        assert_eq!(do_lisp("(list-ref (iota 10) #t)"), "E1002");

        assert_eq!(do_lisp("(list-ref a #t)"), "E1008");
        assert_eq!(do_lisp("(list-ref (iota 10) a)"), "E1008");

        assert_eq!(do_lisp("(list-ref (iota 10) -1)"), "E1011");
        assert_eq!(do_lisp("(list-ref (iota 10) 10)"), "E1011");
    }
    #[test]
    fn list_set() {
        assert_eq!(do_lisp("(list-set!)"), "E1007");
        assert_eq!(do_lisp("(list-set! (iota 10))"), "E1007");
        assert_eq!(do_lisp("(list-set! (iota 10) 1 2 3)"), "E1007");

        assert_eq!(do_lisp("(list-set! 10 0 -1)"), "E1005");
        assert_eq!(do_lisp("(list-set! (iota 10) #t 0)"), "E1002");

        assert_eq!(do_lisp("(list-set! a 0 #t)"), "E1008");
        assert_eq!(do_lisp("(list-set! (iota 10) 0 a)"), "E1008");

        assert_eq!(do_lisp("(list-set! (iota 10) -1 0)"), "E1011");
        assert_eq!(do_lisp("(list-set! (iota 10) 10 0)"), "E1011");
    }
    #[test]
    fn set_car() {
        assert_eq!(do_lisp("(set-car!)"), "E1007");
        assert_eq!(do_lisp("(set-car! (list 1))"), "E1007");
        assert_eq!(do_lisp("(set-car! c a)"), "E1008");
        assert_eq!(do_lisp("(set-car! 10 20)"), "E1005");
        assert_eq!(do_lisp("(set-car! () 20)"), "E1011");
    }
    #[test]
    fn set_cdr() {
        assert_eq!(do_lisp("(set-cdr!)"), "E1007");
        assert_eq!(do_lisp("(set-cdr! (list 1))"), "E1007");
        assert_eq!(do_lisp("(set-cdr! c a)"), "E1008");
        assert_eq!(do_lisp("(set-cdr! 100 200)"), "E1005");
        assert_eq!(do_lisp("(set-cdr! () 20)"), "E1011");
    }
    #[test]
    fn sort() {
        assert_eq!(do_lisp("(sort)"), "E1007");
        assert_eq!(do_lisp("(sort (list 1) + +)"), "E1007");
        assert_eq!(do_lisp("(sort +)"), "E1005");
        assert_eq!(do_lisp("(sort (list 1) 10)"), "E1006");
    }
    #[test]
    fn sort_effect() {
        assert_eq!(do_lisp("(sort!)"), "E1007");
        assert_eq!(do_lisp("(sort! (list 1) + +)"), "E1007");
        assert_eq!(do_lisp("(sort! +)"), "E1005");
        assert_eq!(do_lisp("(sort! (list 1) 10)"), "E1006");
    }
    #[test]
    fn sort_stable() {
        assert_eq!(do_lisp("(stable-sort)"), "E1007");
        assert_eq!(do_lisp("(stable-sort (list 1) + +)"), "E1007");
        assert_eq!(do_lisp("(stable-sort +)"), "E1005");
        assert_eq!(do_lisp("(stable-sort (list 1) 10)"), "E1006");
    }
    #[test]
    fn sort_stable_effect() {
        assert_eq!(do_lisp("(stable-sort!)"), "E1007");
        assert_eq!(do_lisp("(stable-sort! (list 1) + +)"), "E1007");
        assert_eq!(do_lisp("(stable-sort! +)"), "E1005");
        assert_eq!(do_lisp("(stable-sort! (list 1) 10)"), "E1006");
    }
    #[test]
    fn merge() {
        assert_eq!(do_lisp("(merge)"), "E1007");
        assert_eq!(do_lisp("(merge (list 1))"), "E1007");
        assert_eq!(do_lisp("(merge (list 1)(list 2) + +)"), "E1007");
        assert_eq!(do_lisp("(merge + (list 2) +)"), "E1005");
        assert_eq!(do_lisp("(merge (list 1) + +)"), "E1005");
        assert_eq!(do_lisp("(merge (list 1)(list 2)(list 3))"), "E1006");
        assert_eq!(do_lisp("(merge (list 1)(list 2) +)"), "E1001");
    }
    #[test]
    fn is_sorted() {
        assert_eq!(do_lisp("(sorted?)"), "E1007");
        assert_eq!(do_lisp("(sorted? (list 1) + +)"), "E1007");
        assert_eq!(do_lisp("(sorted? +)"), "E1005");
        assert_eq!(do_lisp("(sorted? (list 1) 10)"), "E1006");
    }
    #[test]
    fn vector() {
        assert_eq!(do_lisp("(vector c 10)"), "E1008");
        assert_eq!(do_lisp("#"), "E1008");
    }
    #[test]
    fn make_vector() {
        assert_eq!(do_lisp("(make-vector)"), "E1007");
        assert_eq!(do_lisp("(make-vector 10)"), "E1007");
        assert_eq!(do_lisp("(make-vector 10 0 1)"), "E1007");
        assert_eq!(do_lisp("(make-vector #t 0)"), "E1002");
        assert_eq!(do_lisp("(make-vector -1 0)"), "E1011");
        assert_eq!(do_lisp("(make-vector 10 c)"), "E1008");
    }
    #[test]
    fn vector_length() {
        assert_eq!(do_lisp("(vector-length)"), "E1007");
        assert_eq!(do_lisp("(vector-length (vector 1)(vector 2))"), "E1007");
        assert_eq!(do_lisp("(vector-length (list 1 2))"), "E1022");
        assert_eq!(do_lisp("(vector-length (cons 1 2))"), "E1022");
        assert_eq!(do_lisp("(vector-length a)"), "E1008");
    }
    #[test]
    fn vector_list() {
        assert_eq!(do_lisp("(vector->list)"), "E1007");
        assert_eq!(do_lisp("(vector->list #(1 2 3) #(1 2 3))"), "E1007");
        assert_eq!(do_lisp("(vector->list '(1 2 3))"), "E1022");
    }
    #[test]
    fn list_vector() {
        assert_eq!(do_lisp("(list->vector)"), "E1007");
        assert_eq!(do_lisp("(list->vector (list 1 2 3) '(1 2 3))"), "E1007");
        assert_eq!(do_lisp("(list->vector #(1 2 3))"), "E1005");
    }
    #[test]
    fn vector_append() {
        assert_eq!(do_lisp("(vector-append)"), "E1007");
        assert_eq!(do_lisp("(vector-append 10)"), "E1022");
        assert_eq!(do_lisp("(vector-append (vector 1) 105)"), "E1022");
        assert_eq!(do_lisp("(vector-append (vector 1) a)"), "E1008");
    }
    #[test]
    fn vector_append_effect() {
        assert_eq!(do_lisp("(vector-append!)"), "E1007");
        assert_eq!(do_lisp("(vector-append! 10)"), "E1022");
        assert_eq!(do_lisp("(vector-append! (vector 1) 105)"), "E1022");
        assert_eq!(do_lisp("(vector-append! (vector 1) a)"), "E1008");
    }
    #[test]
    fn vector_ref() {
        assert_eq!(do_lisp("(vector-ref)"), "E1007");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)))"), "E1007");
        assert_eq!(
            do_lisp("(vector-ref (list->vector (iota 10)) 1 2)"),
            "E1007"
        );
        assert_eq!(do_lisp("(vector-ref 10 -1)"), "E1022");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) #t)"), "E1002");

        assert_eq!(do_lisp("(vector-ref a #t)"), "E1008");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) a)"), "E1008");

        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) -1)"), "E1011");
        assert_eq!(do_lisp("(vector-ref (list->vector (iota 10)) 10)"), "E1011");
    }
    #[test]
    fn vector_set() {
        assert_eq!(do_lisp("(vector-set!)"), "E1007");
        assert_eq!(do_lisp("(vector-set! (list->vector (iota 10)))"), "E1007");
        assert_eq!(
            do_lisp("(vector-set! (list->vector (iota 10)) 1 2 3)"),
            "E1007"
        );

        assert_eq!(do_lisp("(vector-set! 10 0 -1)"), "E1022");
        assert_eq!(
            do_lisp("(vector-set! (list->vector (iota 10)) #t 0)"),
            "E1002"
        );

        assert_eq!(do_lisp("(vector-set! a 0 #t)"), "E1008");
        assert_eq!(
            do_lisp("(vector-set! (list->vector (iota 10)) 0 a)"),
            "E1008"
        );

        assert_eq!(
            do_lisp("(vector-set! (list->vector (iota 10)) -1 0)"),
            "E1011"
        );
        assert_eq!(
            do_lisp("(vector-set! (list->vector (iota 10)) 10 0)"),
            "E1011"
        );
    }
}
