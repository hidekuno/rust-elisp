/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};

use crate::number::Number;

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
    b.regist("modulo", |exp, env| divide(exp, env, |x, y| x % y));
    b.regist("quotient", |exp, env| divide(exp, env, |x, y| x / y));
}
fn calc(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {
    let mut result: Number = Number::Integer(0);
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Float(v) => Number::Float(v),
            Expression::Integer(v) => Number::Integer(v),
            Expression::Rational(v) => Number::Rational(v),
            _ => return Err(create_error!(ErrCode::E1003)),
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
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v: [Number; 2] = [Number::Integer(0); 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Float(f) => Number::Float(f),
            Expression::Integer(i) => Number::Integer(i),
            Expression::Rational(r) => Number::Rational(r),
            _ => return Err(create_error!(ErrCode::E1003)),
        }
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn divide(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &i64, y: &i64) -> i64,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);
    match (a, b) {
        (Expression::Integer(x), Expression::Integer(y)) => {
            if y == 0 {
                Err(create_error!(ErrCode::E1013))
            } else {
                Ok(Expression::Integer(f(&x, &y)))
            }
        }
        (_, _) => Err(create_error!(ErrCode::E1002)),
    }
}
fn shift(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut x: [i64; 2] = [0; 2];
    for (i, e) in exp[1 as usize..].iter().enumerate() {
        x[i] = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(ErrCode::E1002)),
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
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        let param = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(ErrCode::E1002)),
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
        Err(create_error_value!(ErrCode::E1007, exp.len()))
    } else {
        match eval(&exp[1], env)? {
            Expression::Integer(v) => Ok(Expression::Integer(!v)),
            _ => Err(create_error!(ErrCode::E1002)),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::do_lisp;

    #[test]
    fn plus() {
        assert_eq!(do_lisp("(+ 1 2)"), "3");
        assert_eq!(do_lisp("(+ 1.25 2.25)"), "3.5");
        assert_eq!(do_lisp("(+ 2.5 1)"), "3.5");
        assert_eq!(do_lisp("(+ 3 1.5)"), "4.5");
        assert_eq!(do_lisp("(+ (* 1 2)(* 3 4))"), "14");
        assert_eq!(do_lisp("(+ 1/2 1)"), "3/2");
    }
    #[test]
    fn minus() {
        assert_eq!(do_lisp("(- 6 1)"), "5");
        assert_eq!(do_lisp("(- 5.75 1.5)"), "4.25");
        assert_eq!(do_lisp("(- 6 1.5)"), "4.5");
        assert_eq!(do_lisp("(- 6.5 3)"), "3.5");
        assert_eq!(do_lisp("(- (* 3 4)(* 1 2))"), "10");
        assert_eq!(do_lisp("(- 1 1/2)"), "1/2");
    }
    #[test]
    fn multi() {
        assert_eq!(do_lisp("(* 3 6)"), "18");
        assert_eq!(do_lisp("(* 0.5 5.75)"), "2.875");
        assert_eq!(do_lisp("(* 3.5 6)"), "21");
        assert_eq!(do_lisp("(* 6 3.5)"), "21");
        assert_eq!(do_lisp("(* (+ 3 4)(+ 1 2))"), "21");
        assert_eq!(do_lisp("(* 1/2 1)"), "1/2");
    }
    #[test]
    fn div() {
        assert_eq!(do_lisp("(/ 4 3)"), "4/3");
        assert_eq!(do_lisp("(/ 1 2)"), "1/2");
        assert_eq!(do_lisp("(/ 9 3)"), "3");
        assert_eq!(do_lisp("(/ 0.75 0.25)"), "3");
        assert_eq!(do_lisp("(/ 9.5 5)"), "1.9");
        assert_eq!(do_lisp("(/ 6 2.5)"), "2.4");
        assert_eq!(do_lisp("(/ 0 0)"), "NaN");
        assert_eq!(do_lisp("(/ 9 0)"), "inf");
        assert_eq!(do_lisp("(/ 10 0.0)"), "inf");
        assert_eq!(do_lisp("(+ 10 (/ 0 0))"), "NaN");
        assert_eq!(do_lisp("(+ 10 (/ 9 0))"), "inf");
        assert_eq!(do_lisp("(/ 0 9)"), "0");
        assert_eq!(do_lisp("(/ 0.0 9)"), "0");
        assert_eq!(do_lisp("(/ (+ 4 4)(+ 2 2))"), "2");
    }
    #[test]
    fn max_f() {
        assert_eq!(do_lisp("(max 10 12 11 1 2)"), "12");
        assert_eq!(do_lisp("(max 10 12 11 1 12)"), "12");
        assert_eq!(do_lisp("(max 10 12 13.5 1 1)"), "13.5");
        assert_eq!(do_lisp("(max 10 123/11 10.5 1 1)"), "123/11");
    }
    #[test]
    fn min_f() {
        assert_eq!(do_lisp("(min 10 12 11 3 9)"), "3");
        assert_eq!(do_lisp("(min 3 12 11 3 12)"), "3");
        assert_eq!(do_lisp("(min 10 12 0.5 1 1)"), "0.5");
        assert_eq!(do_lisp("(min 10 1/11 10.5 1 1)"), "1/11");
    }
    #[test]
    fn eq() {
        assert_eq!(do_lisp("(= 5 5)"), "#t");
        assert_eq!(do_lisp("(= 5.5 5.5)"), "#t");
        assert_eq!(do_lisp("(= 5 5.0)"), "#t");
        assert_eq!(do_lisp("(= 5.0 5)"), "#t");
        assert_eq!(do_lisp("(= 5 6)"), "#f");
        assert_eq!(do_lisp("(= 5.5 6.6)"), "#f");
        assert_eq!(do_lisp("(= 5 6.6)"), "#f");
        assert_eq!(do_lisp("(= 5.0 6)"), "#f");
        assert_eq!(do_lisp("(= (+ 1 1)(+ 0 2))"), "#t");
    }
    #[test]
    fn than() {
        assert_eq!(do_lisp("(> 6 5)"), "#t");
        assert_eq!(do_lisp("(> 6.5 5.5)"), "#t");
        assert_eq!(do_lisp("(> 6.1 6)"), "#t");
        assert_eq!(do_lisp("(> 6 5.9)"), "#t");
        assert_eq!(do_lisp("(> 6 6)"), "#f");
        assert_eq!(do_lisp("(> 4.5 5.5)"), "#f");
        assert_eq!(do_lisp("(> 4 5.5)"), "#f");
        assert_eq!(do_lisp("(> 4.5 5)"), "#f");
        assert_eq!(do_lisp("(> (+ 3 3) 5)"), "#t");
    }
    #[test]
    fn less() {
        assert_eq!(do_lisp("(< 5 6)"), "#t");
        assert_eq!(do_lisp("(< 5.6 6.5)"), "#t");
        assert_eq!(do_lisp("(< 5 6.1)"), "#t");
        assert_eq!(do_lisp("(< 5 6.5)"), "#t");
        assert_eq!(do_lisp("(> 6 6)"), "#f");
        assert_eq!(do_lisp("(> 6.5 6.6)"), "#f");
        assert_eq!(do_lisp("(> 6 6.0)"), "#f");
        assert_eq!(do_lisp("(> 5.9 6)"), "#f");
        assert_eq!(do_lisp("(< 5 (+ 3 3))"), "#t");
    }
    #[test]
    fn than_eq() {
        assert_eq!(do_lisp("(>= 6 6)"), "#t");
        assert_eq!(do_lisp("(>= 6 5)"), "#t");
        assert_eq!(do_lisp("(>= 6.1 5)"), "#t");
        assert_eq!(do_lisp("(>= 7.6 7.6)"), "#t");
        assert_eq!(do_lisp("(>= 6.3 5.2)"), "#t");
        assert_eq!(do_lisp("(>= 6 5.1)"), "#t");
        assert_eq!(do_lisp("(>= 5 6)"), "#f");
        assert_eq!(do_lisp("(>= 5.1 6.2)"), "#f");
        assert_eq!(do_lisp("(>= 5.9 6)"), "#f");
        assert_eq!(do_lisp("(>= 5 6.1)"), "#f");
        assert_eq!(do_lisp("(>= (+ 2 3 1) 6)"), "#t");
    }
    #[test]
    fn less_eq() {
        assert_eq!(do_lisp("(<= 6 6)"), "#t");
        assert_eq!(do_lisp("(<= 6 5)"), "#f");
        assert_eq!(do_lisp("(<= 6.1 5)"), "#f");
        assert_eq!(do_lisp("(<= 7.6 7.6)"), "#t");
        assert_eq!(do_lisp("(<= 6.3 5.2)"), "#f");
        assert_eq!(do_lisp("(<= 6 5.1)"), "#f");
        assert_eq!(do_lisp("(<= 5 6)"), "#t");
        assert_eq!(do_lisp("(<= 5.1 6.2)"), "#t");
        assert_eq!(do_lisp("(<= 5.9 6)"), "#t");
        assert_eq!(do_lisp("(<= 5 6.1)"), "#t");
        assert_eq!(do_lisp("(<= (+ 3 3) 6)"), "#t");
    }
    #[test]
    fn ash() {
        assert_eq!(do_lisp("(ash 10 1)"), "20");
        assert_eq!(do_lisp("(ash 10 -1)"), "5");
        assert_eq!(do_lisp("(ash 10 0)"), "10");
    }
    #[test]
    fn logand() {
        assert_eq!(do_lisp("(logand 10 2)"), "2");
        assert_eq!(do_lisp("(logand 10 2 3)"), "2");
    }
    #[test]
    fn logior() {
        assert_eq!(do_lisp("(logior 10 2)"), "10");
        assert_eq!(do_lisp("(logior 10 2 3)"), "11");
    }
    #[test]
    fn logxor() {
        assert_eq!(do_lisp("(logxor 10 2)"), "8");
        assert_eq!(do_lisp("(logxor 10 2 2)"), "10");
    }
    #[test]
    fn lognot() {
        assert_eq!(do_lisp("(lognot 10)"), "-11");
    }
    #[test]
    fn modulo() {
        assert_eq!(do_lisp("(modulo 11 3)"), "2");
        assert_eq!(do_lisp("(modulo 11 (+ 1 2))"), "2");
        assert_eq!(do_lisp("(modulo  3 5)"), "3");
    }
    #[test]
    fn quotient() {
        assert_eq!(do_lisp("(quotient 11 3)"), "3");
        assert_eq!(do_lisp("(quotient 11 (+ 1 2))"), "3");
        assert_eq!(do_lisp("(quotient 3 5)"), "0");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;

    #[test]
    fn plus() {
        assert_eq!(do_lisp("(+ 1 a)"), "E1008");
        assert_eq!(do_lisp("(+ 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(+ 1)"), "E1007");
    }
    #[test]
    fn minus() {
        assert_eq!(do_lisp("(- 6 a)"), "E1008");
        assert_eq!(do_lisp("(- 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(- 1)"), "E1007");
    }
    #[test]
    fn multi() {
        assert_eq!(do_lisp("(* 6 a)"), "E1008");
        assert_eq!(do_lisp("(* 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(* 1)"), "E1007");
    }
    #[test]
    fn div() {
        assert_eq!(do_lisp("(/ 9 a)"), "E1008");
        assert_eq!(do_lisp("(/ 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(/ 1)"), "E1007");
    }
    #[test]
    fn max_f() {
        assert_eq!(do_lisp("(max 10)"), "E1007");
        assert_eq!(do_lisp("(max 9 a)"), "E1008");
        assert_eq!(do_lisp("(max 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(max 1)"), "E1007");
    }
    #[test]
    fn min_f() {
        assert_eq!(do_lisp("(min 10)"), "E1007");
        assert_eq!(do_lisp("(min 9 a)"), "E1008");
        assert_eq!(do_lisp("(min 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(min 1)"), "E1007");
    }
    #[test]
    fn eq() {
        assert_eq!(do_lisp("(= 5)"), "E1007");
        assert_eq!(do_lisp("(= 5 a)"), "E1008");
        assert_eq!(do_lisp("(= 5 #f)"), "E1003");
    }
    #[test]
    fn than() {
        assert_eq!(do_lisp("(> 6)"), "E1007");
        assert_eq!(do_lisp("(> 6 a)"), "E1008");
        assert_eq!(do_lisp("(> 6 #f)"), "E1003");
    }
    #[test]
    fn less() {
        assert_eq!(do_lisp("(< 5)"), "E1007");
        assert_eq!(do_lisp("(< 5 a)"), "E1008");
        assert_eq!(do_lisp("(< 5 #f)"), "E1003");
    }
    #[test]
    fn than_eq() {
        assert_eq!(do_lisp("(>= 6)"), "E1007");
        assert_eq!(do_lisp("(>= 6 a)"), "E1008");
        assert_eq!(do_lisp("(>= 6 #t)"), "E1003");
    }
    #[test]
    fn less_eq() {
        assert_eq!(do_lisp("(<= 6)"), "E1007");
        assert_eq!(do_lisp("(<= 6 a)"), "E1008");
        assert_eq!(do_lisp("(<= 6 #t)"), "E1003");
    }
    #[test]
    fn ash() {
        assert_eq!(do_lisp("(ash)"), "E1007");
        assert_eq!(do_lisp("(ash 10)"), "E1007");
        assert_eq!(do_lisp("(ash 10 1 1)"), "E1007");
        assert_eq!(do_lisp("(ash a 1)"), "E1008");
        assert_eq!(do_lisp("(ash 10 a)"), "E1008");
        assert_eq!(do_lisp("(ash 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(ash 10 1.5)"), "E1002");
    }
    #[test]
    fn logand() {
        assert_eq!(do_lisp("(logand)"), "E1007");
        assert_eq!(do_lisp("(logand 10)"), "E1007");
        assert_eq!(do_lisp("(logand a 1)"), "E1008");
        assert_eq!(do_lisp("(logand 10 a)"), "E1008");
        assert_eq!(do_lisp("(logand 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logand 10 1.5)"), "E1002");
    }
    #[test]
    fn logior() {
        assert_eq!(do_lisp("(logior)"), "E1007");
        assert_eq!(do_lisp("(logior 10)"), "E1007");
        assert_eq!(do_lisp("(logior a 1)"), "E1008");
        assert_eq!(do_lisp("(logior 10 a)"), "E1008");
        assert_eq!(do_lisp("(logior 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logior 10 1.5)"), "E1002");
    }
    #[test]
    fn logxor() {
        assert_eq!(do_lisp("(logxor)"), "E1007");
        assert_eq!(do_lisp("(logxor 10)"), "E1007");
        assert_eq!(do_lisp("(logxor a 1)"), "E1008");
        assert_eq!(do_lisp("(logxor 10 a)"), "E1008");
        assert_eq!(do_lisp("(logxor 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logxor 10 1.5)"), "E1002");
    }
    #[test]
    fn lognot() {
        assert_eq!(do_lisp("(lognot)"), "E1007");
        assert_eq!(do_lisp("(lognot 10 10)"), "E1007");
        assert_eq!(do_lisp("(lognot a)"), "E1008");
        assert_eq!(do_lisp("(lognot 1.5)"), "E1002");
    }
    #[test]
    fn modulo() {
        assert_eq!(do_lisp("(modulo 10)"), "E1007");
        assert_eq!(do_lisp("(modulo 10 0)"), "E1013");
        assert_eq!(do_lisp("(modulo 13 5.5)"), "E1002");
        assert_eq!(do_lisp("(modulo 10 a)"), "E1008");
    }
    #[test]
    fn quotient() {
        assert_eq!(do_lisp("(quotient 10)"), "E1007");
        assert_eq!(do_lisp("(quotient 10 0)"), "E1013");
        assert_eq!(do_lisp("(quotient 13 5.5)"), "E1002");
        assert_eq!(do_lisp("(quotient 10 a)"), "E1008");
    }
}
