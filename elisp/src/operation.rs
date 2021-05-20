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
    b.regist("+", |exp, env| calc(exp, env, |x, y| x + y, 0));
    b.regist("-", |exp, env| calc(exp, env, |x, y| x - y, 0));
    b.regist("*", |exp, env| calc(exp, env, |x, y| x * y, 1));
    b.regist("/", |exp, env| calc(exp, env, |x, y| x / y, 1));
    b.regist("max", |exp, env| {
        select_one(exp, env, |x, y| if x > y { x } else { y })
    });
    b.regist("min", |exp, env| {
        select_one(exp, env, |x, y| if x < y { x } else { y })
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
    b.regist("logcount", |exp, env| {
        bitcount(exp, env, |v, i| (1 & (v >> i)) > 0)
    });
    b.regist("integer-length", |exp, env| {
        bitcount(exp, env, |v, i| (v >> i) > 0)
    });

    b.regist("modulo", |exp, env| divide(exp, env, |x, y| x % y));
    b.regist("quotient", |exp, env| divide(exp, env, |x, y| x / y));
    b.regist("twos-exponent", twos_exponent);
}
fn calc(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: Number, y: Number) -> Number,
    x: i64,
) -> ResultExpression {
    if 1 >= exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut result = Expression::to_number(&eval(&exp[1], env)?)?;

    if 2 == exp.len() {
        result = f(Number::Integer(x), result);
    } else {
        for e in &exp[2..] {
            let param = Expression::to_number(&eval(e, env)?)?;
            result = f(result, param);
        }
    }
    Ok(Number::to_expression(&result))
}
fn select_one(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {
    if 1 >= exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut result = Expression::to_number(&eval(&exp[1], env)?)?;

    for e in &exp[2..] {
        let param = Expression::to_number(&eval(e, env)?)?;
        result = f(result, param);
    }
    Ok(Number::to_expression(&result))
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

    for (i, e) in exp[1..].iter().enumerate() {
        v[i] = Expression::to_number(&eval(e, env)?)?;
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
    for (i, e) in exp[1..].iter().enumerate() {
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

    if 1 >= exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1..] {
        let param = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(ErrCode::E1002)),
        };
        if first {
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
fn bitcount(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: i64, y: i64) -> bool,
) -> ResultExpression {
    if exp.len() != 2 {
        Err(create_error_value!(ErrCode::E1007, exp.len()))
    } else {
        match eval(&exp[1], env)? {
            Expression::Integer(v) => {
                // https://practical-scheme.net/gauche/man/gauche-refe/Numbers.html
                // (If n is negative, returns the number of 0’s in the bits of 2’s complement)
                let x = if v >= 0 { v } else { !v };

                let mut n = 0;
                for i in 0..64 {
                    if f(x, i) {
                        n += 1;
                    }
                }

                Ok(Expression::Integer(n))
            }
            _ => Err(create_error!(ErrCode::E1002)),
        }
    }
}
fn twos_exponent(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Integer(v) => v,
        _ => return Err(create_error!(ErrCode::E1002)),
    };
    if 0 >= v {
        return Ok(Expression::Boolean(false));
    }
    let m = 1;
    for i in 0..63 {
        if v == (m << i) {
            return Ok(Expression::Integer(i));
        }
        if v < (m << i) {
            break;
        }
    }
    Ok(Expression::Boolean(false))
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
        assert_eq!(do_lisp("(+ 10)"), "10");
    }
    #[test]
    fn minus() {
        assert_eq!(do_lisp("(- 6 1)"), "5");
        assert_eq!(do_lisp("(- 5.75 1.5)"), "4.25");
        assert_eq!(do_lisp("(- 6 1.5)"), "4.5");
        assert_eq!(do_lisp("(- 6.5 3)"), "3.5");
        assert_eq!(do_lisp("(- (* 3 4)(* 1 2))"), "10");
        assert_eq!(do_lisp("(- 1 1/2)"), "1/2");
        assert_eq!(do_lisp("(- 10)"), "-10");
    }
    #[test]
    fn multi() {
        assert_eq!(do_lisp("(* 3 6)"), "18");
        assert_eq!(do_lisp("(* 0.5 5.75)"), "2.875");
        assert_eq!(do_lisp("(* 3.5 6)"), "21");
        assert_eq!(do_lisp("(* 6 3.5)"), "21");
        assert_eq!(do_lisp("(* (+ 3 4)(+ 1 2))"), "21");
        assert_eq!(do_lisp("(* 1/2 1)"), "1/2");
        assert_eq!(do_lisp("(* 10)"), "10");
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
        assert_eq!(do_lisp("(/ 10)"), "1/10");
    }
    #[test]
    fn max_f() {
        assert_eq!(do_lisp("(max 10 12 11 1 2)"), "12");
        assert_eq!(do_lisp("(max 10 12 11 1 12)"), "12");
        assert_eq!(do_lisp("(max 10 12 13.5 1 1)"), "13.5");
        assert_eq!(do_lisp("(max 10 123/11 10.5 1 1)"), "123/11");
        assert_eq!(do_lisp("(max 10)"), "10");
    }
    #[test]
    fn min_f() {
        assert_eq!(do_lisp("(min 10 12 11 3 9)"), "3");
        assert_eq!(do_lisp("(min 3 12 11 3 12)"), "3");
        assert_eq!(do_lisp("(min 10 12 0.5 1 1)"), "0.5");
        assert_eq!(do_lisp("(min 10 1/11 10.5 1 1)"), "1/11");
        assert_eq!(do_lisp("(min 10)"), "10");
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
        assert_eq!(do_lisp("(logand 10)"), "10");
    }
    #[test]
    fn logior() {
        assert_eq!(do_lisp("(logior 10 2)"), "10");
        assert_eq!(do_lisp("(logior 10 2 3)"), "11");
        assert_eq!(do_lisp("(logior 10)"), "10");
    }
    #[test]
    fn logxor() {
        assert_eq!(do_lisp("(logxor 10 2)"), "8");
        assert_eq!(do_lisp("(logxor 10 2 2)"), "10");
        assert_eq!(do_lisp("(logxor 10)"), "10");
    }
    #[test]
    fn lognot() {
        assert_eq!(do_lisp("(lognot 10)"), "-11");
    }
    #[test]
    fn logcount() {
        assert_eq!(do_lisp("(logcount 0)"), "0");
        assert_eq!(do_lisp("(logcount 11)"), "3");
        assert_eq!(do_lisp("(logcount 18)"), "2");
        assert_eq!(do_lisp("(logcount -1)"), "0");
        assert_eq!(do_lisp("(logcount -2)"), "1");
        assert_eq!(do_lisp("(logcount -256)"), "8");
        assert_eq!(do_lisp("(logcount -257)"), "1");
    }
    #[test]
    fn integer_length() {
        assert_eq!(do_lisp("(integer-length 0)"), "0");
        assert_eq!(do_lisp("(integer-length 11)"), "4");
        assert_eq!(do_lisp("(integer-length 18)"), "5");
        assert_eq!(do_lisp("(integer-length -1)"), "0");
        assert_eq!(do_lisp("(integer-length -2)"), "1");
        assert_eq!(do_lisp("(integer-length -256)"), "8");
        assert_eq!(do_lisp("(integer-length -257)"), "9");
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
    #[test]
    fn twos_exponent() {
        assert_eq!(do_lisp("(twos-exponent -1)"), "#f");
        assert_eq!(do_lisp("(twos-exponent 0)"), "#f");
        assert_eq!(do_lisp("(twos-exponent 1)"), "0");
        assert_eq!(do_lisp("(twos-exponent 2)"), "1");
        assert_eq!(do_lisp("(twos-exponent 9)"), "#f");
        assert_eq!(do_lisp("(twos-exponent 10)"), "#f");
        assert_eq!(do_lisp("(twos-exponent 16)"), "4");
        assert_eq!(do_lisp("(twos-exponent 9223372036854775807)"), "#f");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;

    #[test]
    fn plus() {
        assert_eq!(do_lisp("(+ 1 a)"), "E1008");
        assert_eq!(do_lisp("(+ 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(+)"), "E1007");
    }
    #[test]
    fn minus() {
        assert_eq!(do_lisp("(- 6 a)"), "E1008");
        assert_eq!(do_lisp("(- 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(-)"), "E1007");
    }
    #[test]
    fn multi() {
        assert_eq!(do_lisp("(* 6 a)"), "E1008");
        assert_eq!(do_lisp("(* 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(*)"), "E1007");
    }
    #[test]
    fn div() {
        assert_eq!(do_lisp("(/ 9 a)"), "E1008");
        assert_eq!(do_lisp("(/ 1 3.4 #t)"), "E1003");
        assert_eq!(do_lisp("(/)"), "E1007");
    }
    #[test]
    fn max_f() {
        assert_eq!(do_lisp("(max)"), "E1007");
        assert_eq!(do_lisp("(max 9 a)"), "E1008");
        assert_eq!(do_lisp("(max 1 3.4 #t)"), "E1003");
    }
    #[test]
    fn min_f() {
        assert_eq!(do_lisp("(min)"), "E1007");
        assert_eq!(do_lisp("(min 9 a)"), "E1008");
        assert_eq!(do_lisp("(min 1 3.4 #t)"), "E1003");
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
        assert_eq!(do_lisp("(logand a 1)"), "E1008");
        assert_eq!(do_lisp("(logand 10 a)"), "E1008");
        assert_eq!(do_lisp("(logand 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logand 10 1.5)"), "E1002");
    }
    #[test]
    fn logior() {
        assert_eq!(do_lisp("(logior)"), "E1007");
        assert_eq!(do_lisp("(logior a 1)"), "E1008");
        assert_eq!(do_lisp("(logior 10 a)"), "E1008");
        assert_eq!(do_lisp("(logior 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logior 10 1.5)"), "E1002");
    }
    #[test]
    fn logxor() {
        assert_eq!(do_lisp("(logxor)"), "E1007");
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
    fn logcount() {
        assert_eq!(do_lisp("(logcount)"), "E1007");
        assert_eq!(do_lisp("(logcount 10 10)"), "E1007");
        assert_eq!(do_lisp("(logcount a)"), "E1008");
        assert_eq!(do_lisp("(logcount 1.5)"), "E1002");
    }
    #[test]
    fn integer_length() {
        assert_eq!(do_lisp("(integer-length)"), "E1007");
        assert_eq!(do_lisp("(integer-length 10 10)"), "E1007");
        assert_eq!(do_lisp("(integer-length a)"), "E1008");
        assert_eq!(do_lisp("(integer-length 1.5)"), "E1002");
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
    #[test]
    fn twos_exponent() {
        assert_eq!(do_lisp("(twos-exponent)"), "E1007");
        assert_eq!(do_lisp("(twos-exponent #f)"), "E1002");
        assert_eq!(do_lisp("(twos-exponent a)"), "E1008");
    }
}
