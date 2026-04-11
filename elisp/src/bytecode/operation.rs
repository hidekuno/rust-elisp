/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn}; // ex.) export RUST_LOG=debug

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::{ErrCode, Error};
use crate::lisp::{Expression, Int, ResultExpression};

use crate::number::Number;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("max", |exp, _| {
        select_one(exp, |x, y| if x > y { x } else { y })
    });
    b.regist("min", |exp, _| {
        select_one(exp, |x, y| if x < y { x } else { y })
    });

    b.regist("ash", |exp, _| shift(exp));
    b.regist("logand", |exp, _| bit(exp, |x, y| x & y));
    b.regist("logior", |exp, _| bit(exp, |x, y| x | y));
    b.regist("logxor", |exp, _| bit(exp, |x, y| x ^ y));
    b.regist("lognot", |exp, _| lognot(exp));
    b.regist("logcount", |exp, _| {
        bitcount(exp, |v, i| (1 & (v >> i)) > 0)
    });
    b.regist("integer-length", |exp, _| {
        bitcount(exp, |v, i| (v >> i) > 0)
    });

    b.regist("modulo", |exp, _| divide(exp, |x, y| x % y));
    b.regist("quotient", |exp, _| divide(exp, |x, y| x / y));
    b.regist("twos-exponent", |exp, _| twos_exponent(exp));
}
// Fast path: resolve and convert to Number without going through eval's
// is_limit_stop / is_force_stop checks for the common cases of numeric
// literals and simple variable lookups.
#[inline(always)]
fn eval_to_number(e: &Expression) -> Result<Number, Box<Error>> {
    match e {
        Expression::Integer(n) => Ok(Number::Integer(*n)),
        Expression::Float(n) => Ok(Number::Float(*n)),
        Expression::Rational(r) => Ok(Number::Rational(**r)),
        _ => Expression::to_number(e),
    }
}
fn select_one(exp: &[Expression], func: fn(x: Number, y: Number) -> Number) -> ResultExpression {
    if exp.is_empty() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut result = eval_to_number(&exp[0])?;

    for e in &exp[1..] {
        result = func(result, eval_to_number(e)?);
    }
    Ok(Number::to_expression(result))
}
fn divide(exp: &[Expression], func: fn(x: &Int, y: &Int) -> Int) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let (a, b) = (&exp[0], &exp[1]);
    match (a, b) {
        (Expression::Integer(x), Expression::Integer(y)) => {
            if *y == 0 {
                Err(create_error!(ErrCode::E1013))
            } else {
                Ok(Expression::Integer(func(x, y)))
            }
        }
        (_, _) => Err(create_error!(ErrCode::E1002)),
    }
}
fn shift(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut x: [Int; 2] = [0; 2];
    for (i, e) in exp[0..].iter().enumerate() {
        x[i] = match e {
            Expression::Integer(v) => *v,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        };
    }
    Ok(Expression::Integer(if x[1] >= 0 {
        x[0] << x[1]
    } else {
        x[0] >> x[1].abs()
    }))
}
fn bit(exp: &[Expression], func: fn(x: Int, y: Int) -> Int) -> ResultExpression {
    let mut result: Int = 0;
    let mut first: bool = true;

    if exp.is_empty() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[0..] {
        let param = match e {
            Expression::Integer(v) => v,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        };
        if first {
            result = *param;
            first = false;
            continue;
        }
        result = func(result, *param);
    }
    Ok(Expression::Integer(result))
}
fn lognot(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        Err(create_error_value!(ErrCode::E1007, exp.len()))
    } else {
        match &exp[0] {
            Expression::Integer(v) => Ok(Expression::Integer(!v)),
            e => Err(create_error_value!(ErrCode::E1002, e)),
        }
    }
}
fn bitcount(exp: &[Expression], func: fn(x: Int, y: Int) -> bool) -> ResultExpression {
    if exp.len() != 1 {
        Err(create_error_value!(ErrCode::E1007, exp.len()))
    } else {
        match &exp[0] {
            Expression::Integer(v) => {
                // https://practical-scheme.net/gauche/man/gauche-refe/Numbers.html
                // (If n is negative, returns the number of 0’s in the bits of 2’s complement)
                let x = if *v >= 0 { v } else { &!v };

                let mut n = 0;
                for i in 0..64 {
                    if func(*x, i) {
                        n += 1;
                    }
                }

                Ok(Expression::Integer(n))
            }
            e => Err(create_error_value!(ErrCode::E1002, e)),
        }
    }
}
fn twos_exponent(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = match &exp[0] {
        Expression::Integer(v) => v,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if 0 >= *v {
        return Ok(Expression::Boolean(false));
    }
    let m = 1;
    for i in 0..63 {
        if *v == (m << i) {
            return Ok(Expression::Integer(i));
        }
        if *v < (m << i) {
            break;
        }
    }
    Ok(Expression::Boolean(false))
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
    fn max_f() {
        assert_eq!(do_lisp("(max)"), "E1007");
        assert_eq!(do_lisp("(max 1 3.4 #t)"), "E1003");
    }
    #[test]
    fn min_f() {
        assert_eq!(do_lisp("(min)"), "E1007");
        assert_eq!(do_lisp("(min 1 3.4 #t)"), "E1003");
    }
    #[test]
    fn ash() {
        assert_eq!(do_lisp("(ash)"), "E1007");
        assert_eq!(do_lisp("(ash 10)"), "E1007");
        assert_eq!(do_lisp("(ash 10 1 1)"), "E1007");
        assert_eq!(do_lisp("(ash 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(ash 10 1.5)"), "E1002");
    }
    #[test]
    fn logand() {
        assert_eq!(do_lisp("(logand)"), "E1007");
        assert_eq!(do_lisp("(logand 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logand 10 1.5)"), "E1002");
    }
    #[test]
    fn logior() {
        assert_eq!(do_lisp("(logior)"), "E1007");
        assert_eq!(do_lisp("(logior 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logior 10 1.5)"), "E1002");
    }
    #[test]
    fn logxor() {
        assert_eq!(do_lisp("(logxor)"), "E1007");
        assert_eq!(do_lisp("(logxor 10.5 1)"), "E1002");
        assert_eq!(do_lisp("(logxor 10 1.5)"), "E1002");
    }
    #[test]
    fn lognot() {
        assert_eq!(do_lisp("(lognot)"), "E1007");
        assert_eq!(do_lisp("(lognot 10 10)"), "E1007");
        assert_eq!(do_lisp("(lognot 1.5)"), "E1002");
    }
    #[test]
    fn logcount() {
        assert_eq!(do_lisp("(logcount)"), "E1007");
        assert_eq!(do_lisp("(logcount 10 10)"), "E1007");
        assert_eq!(do_lisp("(logcount 1.5)"), "E1002");
    }
    #[test]
    fn integer_length() {
        assert_eq!(do_lisp("(integer-length)"), "E1007");
        assert_eq!(do_lisp("(integer-length 10 10)"), "E1007");
        assert_eq!(do_lisp("(integer-length 1.5)"), "E1002");
    }
    #[test]
    fn modulo() {
        assert_eq!(do_lisp("(modulo 10)"), "E1007");
        assert_eq!(do_lisp("(modulo 10 0)"), "E1013");
        assert_eq!(do_lisp("(modulo 13 5.5)"), "E1002");
    }
    #[test]
    fn quotient() {
        assert_eq!(do_lisp("(quotient 10)"), "E1007");
        assert_eq!(do_lisp("(quotient 10 0)"), "E1013");
        assert_eq!(do_lisp("(quotient 13 5.5)"), "E1002");
    }
    #[test]
    fn twos_exponent() {
        assert_eq!(do_lisp("(twos-exponent)"), "E1007");
        assert_eq!(do_lisp("(twos-exponent #f)"), "E1002");
    }
}
