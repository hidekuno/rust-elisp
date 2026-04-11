/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn}; // ex.) export RUST_LOG=debug
use std::vec::Vec;

use crate::buildin::BuildInTable;
use crate::create_error;
use crate::create_error_value;
use crate::lisp::{Environment, Expression, Int, ListRc, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::reference_obj;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("list", |exp, _| list(exp));
    b.regist("length", |exp, _| length(exp));
    b.regist("cadr", |exp, _| cadr(exp));
    b.regist("caar", |exp, _| caar(exp));
    b.regist("cdar", |exp, _| cdar(exp));
    b.regist("append", |exp, _| append(exp));
    b.regist("take", |exp, _| take_drop(exp, |l, n| &l[0..n]));
    b.regist("drop", |exp, _| take_drop(exp, |l, n| &l[n..]));
    b.regist("delete", |exp, _| delete(exp));
    b.regist("last", |exp, _| last(exp));
    b.regist("reverse", |exp, _| reverse(exp));
    b.regist("iota", |exp, _| iota(exp));
}
fn get_sequence(exp: Expression, err: ErrCode) -> Result<ListRc, Box<Error>> {
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
fn list(exp: &[Expression]) -> ResultExpression {
    let l = seq(exp)?;
    Ok(Environment::create_list(l))
}
fn seq(exp: &[Expression]) -> Result<Vec<Expression>, Box<Error>> {
    let mut list: Vec<Expression> = Vec::with_capacity(exp.len());
    for e in &exp[0..] {
        list.push(e.clone());
    }
    Ok(list)
}
fn length(exp: &[Expression]) -> ResultExpression {
    seq_length(exp, ErrCode::E1005)
}
fn seq_length(exp: &[Expression], err: ErrCode) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = get_sequence(exp[0].clone(), err)?;
    let l = &*(reference_obj!(l));
    Ok(Expression::Integer(l.len() as Int))
}
fn cadr(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            if l.len() <= 1 {
                return Err(create_error!(ErrCode::E1011));
            }
            Ok(l[1].clone())
        }
        Expression::Pair(b) => match &b.1 {
            Expression::List(l) => {
                let l = &*(reference_obj!(l));
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E1011));
                }
                Ok(l[0].clone())
            }
            Expression::Pair(b) => Ok(b.0.clone()),
            _ => Err(create_error!(ErrCode::E1005)),
        },
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn caar(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            if l.is_empty() {
                return Err(create_error!(ErrCode::E1011));
            }
            match &l[0] {
                Expression::List(l) => {
                    let l = &*(reference_obj!(l));
                    if l.is_empty() {
                        return Err(create_error!(ErrCode::E1011));
                    }
                    Ok(l[0].clone())
                }
                Expression::Pair(b) => Ok(b.0.clone()),
                _ => Err(create_error!(ErrCode::E1005)),
            }
        }
        Expression::Pair(b) => match &b.0 {
            Expression::List(l) => {
                let l = &*(reference_obj!(l));
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E1011));
                }
                Ok(l[0].clone())
            }
            Expression::Pair(b) => Ok(b.0.clone()),
            _ => Err(create_error!(ErrCode::E1005)),
        },
        _ => Err(create_error!(ErrCode::E1005)),
    }
}
// (cdar '((2 3 4) 1)) -> (3 4)
fn cdar(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            if l.is_empty() {
                return Err(create_error!(ErrCode::E1011));
            }
            match &l[0] {
                Expression::List(l) => {
                    let l = &*(reference_obj!(l));
                    if l.is_empty() {
                        return Err(create_error!(ErrCode::E1011));
                    }
                    Ok(Environment::create_list(l[1..].to_vec()))
                }
                Expression::Pair(b) => Ok(b.1.clone()),
                _ => Err(create_error!(ErrCode::E1005)),
            }
        }
        Expression::Pair(b) => match &b.0 {
            Expression::List(l) => {
                let l = &*(reference_obj!(l));
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E1011));
                }
                Ok(Environment::create_list(l[1..].to_vec()))
            }
            Expression::Pair(b) => Ok(b.1.clone()),
            _ => Err(create_error!(ErrCode::E1005)),
        },
        _ => Err(create_error!(ErrCode::E1005)),
    }
}
fn append(exp: &[Expression]) -> ResultExpression {
    let v = seq_append(exp, ErrCode::E1005)?;
    Ok(Environment::create_list(v))
}
fn seq_append(exp: &[Expression], err: ErrCode) -> Result<Vec<Expression>, Box<Error>> {
    if exp.is_empty() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v: Vec<Expression> = Vec::new();
    for e in &exp[0..] {
        let l = get_sequence(e.clone(), err.clone())?;
        let l = reference_obj!(l);
        v.append(&mut l.to_vec());
    }
    Ok(v)
}

fn take_drop(
    exp: &[Expression],
    func: fn(l: &Vec<Expression>, n: usize) -> &[Expression],
) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match &exp[0] {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let l = reference_obj!(l);

    let n = match &exp[1] {
        Expression::Integer(n) => n,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if l.len() < *n as usize || *n < 0 {
        return Err(create_error!(ErrCode::E1011));
    }
    let mut vec = Vec::new();
    vec.extend_from_slice(func(&l, *n as usize));

    Ok(Environment::create_list(vec))
}
fn delete(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let other = &exp[0];
    let l = match &exp[1] {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };

    let l = &*(reference_obj!(l));
    let mut vec = Vec::new();
    for e in l {
        if Expression::eqv(e, other) {
            continue;
        }
        vec.push(e.clone());
    }
    Ok(Environment::create_list(vec))
}

fn last(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            match l.len() {
                0 => Err(create_error!(ErrCode::E1011)),
                _ => Ok(l[l.len() - 1].clone()),
            }
        }
        Expression::Pair(b) => Ok(b.0.clone()),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn reverse(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::List(l) => {
            let l = &*(reference_obj!(l));
            let mut l = l.to_vec();
            l.reverse();
            Ok(Environment::create_list(l))
        }
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn iota(exp: &[Expression]) -> ResultExpression {
    if exp.is_empty() || 3 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut param: [Int; 4] = [0, 0, 1, 0];
    for (i, e) in exp[0..].iter().enumerate() {
        match e {
            Expression::Integer(v) => {
                param[i] = *v;
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
#[cfg(test)]
mod tests {
    use crate::bytecode::vm::do_vm_logic;
    use crate::lisp::Environment;
    fn make_env() -> Environment {
        let _g = crate::lisp::COMPILE_LOCK.lock().unwrap();
        crate::lisp::set_compile_mode(true);
        let env = Environment::new();
        crate::lisp::set_compile_mode(false);
        env
    }

    fn do_lisp(program: &str) -> String {
        let env = make_env();
        match do_vm_logic(program, &env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }

    #[test]
    fn list() {
        assert_eq!(do_lisp("(list 1 2)"), "(1 2)");
        assert_eq!(do_lisp("(list 0.5 1)"), "(0.5 1)");
        assert_eq!(do_lisp("(list #t #f)"), "(#t #f)");
        assert_eq!(do_lisp("(list (list 1)(list 2))"), "((1) (2))");
        assert_eq!(do_lisp("(define a 10) (define b 20) (list a b)"), "(10 20)");
    }
    #[test]
    fn length() {
        assert_eq!(do_lisp("(length (list))"), "0");
        assert_eq!(do_lisp("(length (list 3))"), "1");
        assert_eq!(do_lisp("(length (iota 10))"), "10");
    }
    #[test]
    fn cadr() {
        assert_eq!(do_lisp("(cadr (list 1 2))"), "2");
        assert_eq!(do_lisp("(cadr (list 1 2 3))"), "2");
        assert_eq!(do_lisp("(cadr (list 1 (list 2 3)))"), "(2 3)");
        assert_eq!(do_lisp("(cadr (cons 2 '(3 4)))"), "3");
        assert_eq!(do_lisp("(cadr (cons 2 (list 3 4)))"), "3");
    }
    #[test]
    fn caar() {
        assert_eq!(do_lisp("(caar '((1 2) 3))"), "1");
        assert_eq!(do_lisp("(caar '((1 . 2) 3))"), "1");
        assert_eq!(do_lisp("(caar '((1 2) . 3))"), "1");
        assert_eq!(do_lisp("(caar '((1 . 2) . 3))"), "1");
    }
    #[test]
    fn cdar() {
        assert_eq!(do_lisp("(cdar '((1 2) (3 4)))"), "(2)");
        assert_eq!(do_lisp("(cdar '((1 . 2) 3))"), "2");
        assert_eq!(do_lisp("(cdar '((1 . 2) . 3))"), "2");
        assert_eq!(do_lisp("(cdar '((1  2) . 3))"), "(2)");
        assert_eq!(do_lisp("(cdar '((1) (3 4)))"), "()");
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
    }
    #[test]
    fn take() {
        assert_eq!(do_lisp("(take (iota 10) 0)"), "()");
        assert_eq!(do_lisp("(take (iota 10) 1)"), "(0)");
        assert_eq!(do_lisp("(take (iota 10) 3)"), "(0 1 2)");
        assert_eq!(do_lisp("(take (iota 10) 10)"), "(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn drop() {
        assert_eq!(do_lisp("(drop (iota 10) 0)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 1)"), "(1 2 3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 3)"), "(3 4 5 6 7 8 9)");
        assert_eq!(do_lisp("(drop (iota 10) 10)"), "()");
    }
    #[test]
    fn delete() {
        assert_eq!(
            do_lisp("(define a (list 10 20 30)) (delete 20 a)"),
            "(10 30)"
        );
        assert_eq!(do_lisp("(define a (list 1 2 3)) (delete 1 a)"), "(2 3)");
        assert_eq!(do_lisp("(define a (list 1 2 3)) (delete 3 a)"), "(1 2)");
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
}
#[cfg(test)]
mod error_tests {
    use crate::bytecode::vm::do_vm_logic;
    use crate::lisp::Environment;
    fn make_env() -> Environment {
        let _g = crate::lisp::COMPILE_LOCK.lock().unwrap();
        crate::lisp::set_compile_mode(true);
        let env = Environment::new();
        crate::lisp::set_compile_mode(false);
        env
    }

    fn do_lisp(program: &str) -> String {
        let env = make_env();
        match do_vm_logic(program, &env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }

    #[test]
    fn length() {
        assert_eq!(do_lisp("(length)"), "E1007");
        assert_eq!(do_lisp("(length (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(length (cons 1 2))"), "E1005");
    }
    #[test]
    fn cadr() {
        assert_eq!(do_lisp("(cadr)"), "E1007");
        assert_eq!(do_lisp("(cadr (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(cadr (list 1))"), "E1011");
        assert_eq!(do_lisp("(cadr 991)"), "E1005");
        assert_eq!(do_lisp("(cadr '(1 . 2))"), "E1005");
    }
    #[test]
    fn caar() {
        assert_eq!(do_lisp("(caar 1 2)"), "E1007");
        assert_eq!(do_lisp("(caar 10)"), "E1005");
        assert_eq!(do_lisp("(caar '(() 3))"), "E1011");
        assert_eq!(do_lisp("(caar '(10 3))"), "E1005");
    }
    #[test]
    fn cdar() {
        assert_eq!(do_lisp("(cdar 1 2)"), "E1007");
        assert_eq!(do_lisp("(cdar 1)"), "E1005");
        assert_eq!(do_lisp("(cdar '())"), "E1011");
        assert_eq!(do_lisp("(cdar '(() (3 4)))"), "E1011");
        assert_eq!(do_lisp("(cdar '(1 (2 3)))"), "E1005");
    }
    #[test]
    fn append() {
        assert_eq!(do_lisp("(append)"), "E1007");
        assert_eq!(do_lisp("(append 10)"), "E1005");
        assert_eq!(do_lisp("(append (list 1) 105)"), "E1005");
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
    }
    #[test]
    fn delete() {
        assert_eq!(do_lisp("(delete)"), "E1007");
        assert_eq!(do_lisp("(delete 10)"), "E1007");
        assert_eq!(do_lisp("(delete 10 (list 10 20) 3)"), "E1007");
        assert_eq!(do_lisp("(delete 10 20)"), "E1005");
    }
    #[test]
    fn last() {
        assert_eq!(do_lisp("(last)"), "E1007");
        assert_eq!(do_lisp("(last (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(last (list))"), "E1011");
        assert_eq!(do_lisp("(last 29)"), "E1005");
    }
    #[test]
    fn reverse() {
        assert_eq!(do_lisp("(reverse)"), "E1007");
        assert_eq!(do_lisp("(reverse (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(reverse 29)"), "E1005");
    }
    #[test]
    fn iota() {
        assert_eq!(do_lisp("(iota)"), "E1007");
        assert_eq!(do_lisp("(iota 1 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(iota 1.5 2)"), "E1002");
        assert_eq!(do_lisp("(iota 1 10.5)"), "E1002");
        assert_eq!(do_lisp("(iota 10 1 10.5)"), "E1002");
    }
}
