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
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{RsCode, RsError};

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
    let n = match eval(&exp[1], env)? {
        Expression::Integer(v) => v,
        _ => return Err(create_error!(RsCode::E1002)),
    };
    if n < 0 {
        return Err(create_error!(RsCode::E1011));
    }
    let v = eval(&exp[2], env)?;

    Ok(Expression::List(vec![v; n as usize]))
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
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn list() {
        assert_eq!(do_lisp("(list 1 2)"), "(1 2)");
        assert_eq!(do_lisp("(list 0.5 1)"), "(0.5 1)");
        assert_eq!(do_lisp("(list #t #f)"), "(#t #f)");
        assert_eq!(do_lisp("(list (list 1)(list 2))"), "((1)(2))");
        assert_eq!(
            do_lisp("(list (list (list 1))(list 2)(list 3))"),
            "(((1))(2)(3))"
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
            "((1 2 3)(1 2 3)(1 2 3)(1 2 3))"
        );
        assert_eq!(do_lisp("(make-list 8 'a)"), "(a a a a a a a a)");
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
    }
    #[test]
    fn map() {
        assert_eq!(
            do_lisp("(map (lambda (n) (* n 10)) (iota 10 1))"),
            "(10 20 30 40 50 60 70 80 90 100)"
        );
        assert_eq!(do_lisp("(map (lambda (n) (car n)) (list))"), "()");

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
            "((1 2 3)(10 20 30))"
        );
        assert_eq!(
            do_lisp_env("(map (lambda (n) (car n)) d)", &env),
            "((1)(2)(3))"
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
        assert_eq!(do_lisp("(append (list 1))"), "E1007");
        assert_eq!(do_lisp("(append (list 1) 105)"), "E1005");
        assert_eq!(do_lisp("(append (list 1) a)"), "E1008");
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
        assert_eq!(do_lisp("(map (iota 10) (lambda (n) n))"), "E1006");
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
        assert_eq!(do_lisp("(filter (iota 10) (lambda (n) n))"), "E1006");
        assert_eq!(do_lisp("(filter (lambda (n) n) 10)"), "E1005");
        assert_eq!(do_lisp("(filter (lambda (n) n) (iota 4))"), "E1001");
    }
    #[test]
    fn reduce() {
        assert_eq!(do_lisp("(reduce)"), "E1007");
        assert_eq!(do_lisp("(reduce (lambda (n) n))"), "E1007");
        assert_eq!(do_lisp("(reduce 1 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(reduce 0 (list) (list))"), "E1006");
        assert_eq!(do_lisp("(reduce (lambda (n) n) 10 10)"), "E1005");
        assert_eq!(do_lisp("(reduce (lambda (n) n) 0 (iota 4))"), "E1007");
    }
    #[test]
    fn for_each() {
        assert_eq!(do_lisp("(for-each)"), "E1007");
        assert_eq!(do_lisp("(for-each (lambda (n) n))"), "E1007");
        assert_eq!(do_lisp("(for-each 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(for-each (list) (list))"), "E1006");
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
}