/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::env;
use std::time::Instant;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, Int, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::number::Number;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
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
    b.regist("vector?", |exp, env| {
        is_type(exp, env, Expression::is_vector)
    });
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
    b.regist("boolean?", |exp, env| {
        is_type(exp, env, Expression::is_boolean)
    });
    b.regist("symbol?", |exp, env| {
        is_type(exp, env, Expression::is_symbol)
    });
    b.regist("time", time_f);
    b.regist("eq?", eqv);
    b.regist("eqv?", eqv);
    b.regist("identity", identity);
    b.regist("get-environment-variable", get_env);
    b.regist("native-endian", native_endian);
    b.regist("undefined?", |exp, env| {
        is_type(exp, env, Expression::is_undefined)
    });
    b.regist("undefined", undefined);
}
pub fn identity(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    eval(&exp[1], env)
}
fn odd_even(exp: &[Expression], env: &Environment, func: fn(Int) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Integer(i) => Ok(Expression::Boolean(func(i))),
        e => Err(create_error_value!(ErrCode::E1002, e)),
    }
}
fn is_sign(exp: &[Expression], env: &Environment, func: fn(&Number) -> bool) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = Expression::to_number(&eval(&exp[1], env)?)?;

    Ok(Expression::Boolean(func(&v)))
}
fn is_type(
    exp: &[Expression],
    env: &Environment,
    func: fn(e: &Expression) -> bool,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    Ok(Expression::Boolean(func(&v)))
}
fn get_env(exp: &[Expression], env: &Environment) -> ResultExpression {
    //srfi-98
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => match env::var(s) {
            Ok(v) => Ok(Expression::String(v)),
            Err(_) => Ok(Expression::Boolean(false)),
        },
        e => Err(create_error_value!(ErrCode::E1015, e)),
    }
}
fn time_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }

    let start = Instant::now();
    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{:03}(s)", end.as_secs(), end.subsec_millis());
    result
}
pub fn eqv(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let (a, b) = (eval(&exp[1], env)?, eval(&exp[2], env)?);

    if let (Expression::Integer(x), Expression::Rational(y)) = (&a, &b) {
        Ok(Expression::Boolean(
            Number::Integer(*x) == Number::Rational(*y),
        ))
    } else if let (Expression::Rational(x), Expression::Integer(y)) = (&a, &b) {
        Ok(Expression::Boolean(
            Number::Rational(*x) == Number::Integer(*y),
        ))
    } else {
        Ok(Expression::Boolean(Expression::eq(&a, &b)))
    }
}
fn native_endian(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if cfg!(target_endian = "big") {
        return Ok(Expression::Symbol("big-endian".to_string()));
    }
    if cfg!(target_endian = "little") {
        return Ok(Expression::Symbol("little-endian".to_string()));
    }
    Err(create_error!(ErrCode::E9999))
}
pub fn undefined(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(Expression::Nil())
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};
    use std::env;

    #[test]
    fn even() {
        assert_eq!(do_lisp("(even? 2)"), "#t");
        assert_eq!(do_lisp("(even? 4)"), "#t");
        assert_eq!(do_lisp("(even? 0)"), "#t");
        assert_eq!(do_lisp("(even? 1)"), "#f");
        assert_eq!(do_lisp("(even? 5)"), "#f");
    }
    #[test]
    fn odd() {
        assert_eq!(do_lisp("(odd? 2)"), "#f");
        assert_eq!(do_lisp("(odd? 4)"), "#f");
        assert_eq!(do_lisp("(odd? 0)"), "#f");
        assert_eq!(do_lisp("(odd? 1)"), "#t");
        assert_eq!(do_lisp("(odd? 5)"), "#t");
    }
    #[test]
    fn zero() {
        assert_eq!(do_lisp("(zero? 0)"), "#t");
        assert_eq!(do_lisp("(zero? 0.0)"), "#t");
        assert_eq!(do_lisp("(zero? 0/3)"), "#t");
        assert_eq!(do_lisp("(zero? 2)"), "#f");
        assert_eq!(do_lisp("(zero? -3)"), "#f");
        assert_eq!(do_lisp("(zero? 2.5)"), "#f");
        assert_eq!(do_lisp("(zero? 1/3)"), "#f");
    }
    #[test]
    fn positive() {
        assert_eq!(do_lisp("(positive? 0)"), "#f");
        assert_eq!(do_lisp("(positive? 0.0)"), "#f");
        assert_eq!(do_lisp("(positive? 0/3)"), "#f");
        assert_eq!(do_lisp("(positive? 2)"), "#t");
        assert_eq!(do_lisp("(positive? -3)"), "#f");
        assert_eq!(do_lisp("(positive? 2.5)"), "#t");
        assert_eq!(do_lisp("(positive? -1.5)"), "#f");
        assert_eq!(do_lisp("(positive? 1/3)"), "#t");
        assert_eq!(do_lisp("(positive? -1/3)"), "#f");
    }
    #[test]
    fn negative() {
        assert_eq!(do_lisp("(negative? 0)"), "#f");
        assert_eq!(do_lisp("(negative? 0.0)"), "#f");
        assert_eq!(do_lisp("(negative? 0/3)"), "#f");
        assert_eq!(do_lisp("(negative? 2)"), "#f");
        assert_eq!(do_lisp("(negative? -3)"), "#t");
        assert_eq!(do_lisp("(negative? 2.5)"), "#f");
        assert_eq!(do_lisp("(negative? -1.5)"), "#t");
        assert_eq!(do_lisp("(negative? 1/3)"), "#f");
        assert_eq!(do_lisp("(negative? -1/3)"), "#t");
    }
    #[test]
    fn list_f() {
        assert_eq!(do_lisp("(list? (list 1 2 3))"), "#t");
        assert_eq!(do_lisp("(list? 90)"), "#f");
    }
    #[test]
    fn vector_f() {
        assert_eq!(do_lisp("(vector? #(1 2 3))"), "#t");
        assert_eq!(do_lisp("(vector? 90)"), "#f");
    }
    #[test]
    fn pair_f() {
        assert_eq!(do_lisp("(pair? (cons 1 2))"), "#t");
        assert_eq!(do_lisp("(pair? 110)"), "#f");
    }
    #[test]
    fn char_f() {
        assert_eq!(do_lisp("(char? #\\a)"), "#t");
        assert_eq!(do_lisp("(char? 100)"), "#f");
    }
    #[test]
    fn string_f() {
        assert_eq!(do_lisp("(string? \"a\")"), "#t");
        assert_eq!(do_lisp("(string? 100)"), "#f");
    }
    #[test]
    fn integer_f() {
        assert_eq!(do_lisp("(integer? 10)"), "#t");
        assert_eq!(do_lisp("(integer? \"a\")"), "#f");
    }
    #[test]
    fn number_f() {
        assert_eq!(do_lisp("(number? 10)"), "#t");
        assert_eq!(do_lisp("(number? 10.5)"), "#t");
        assert_eq!(do_lisp("(number? 1/3)"), "#t");
        assert_eq!(do_lisp("(number? \"a\")"), "#f");
    }
    #[test]
    fn symbol_f() {
        assert_eq!(do_lisp("(symbol? 'a)"), "#t");
        assert_eq!(do_lisp("(symbol? \"a\")"), "#f");
        assert_eq!(do_lisp("(symbol? 10)"), "#f");
    }
    #[test]
    fn boolean_f() {
        assert_eq!(do_lisp("(boolean? #t)"), "#t");
        assert_eq!(do_lisp("(boolean? #f)"), "#t");
        assert_eq!(do_lisp("(boolean? 10)"), "#f");
    }
    #[test]
    fn procedure_f() {
        assert_eq!(do_lisp("(procedure? (lambda (n)n))"), "#t");
        assert_eq!(do_lisp("(procedure? +)"), "#t");
        assert_eq!(do_lisp("(procedure? 10)"), "#f");
    }
    #[test]
    fn eqv() {
        assert_eq!(do_lisp("(eqv? 1.1 1.1)"), "#t");
        assert_eq!(do_lisp("(eq? 1.1 1.1)"), "#t");
        assert_eq!(do_lisp("(eqv? 1.1 1.2)"), "#f");
        assert_eq!(do_lisp("(eqv? 10 (+ 2 8))"), "#t");
        assert_eq!(do_lisp("(eqv? 1 2)"), "#f");
        assert_eq!(do_lisp("(eqv? 5/3 5/3)"), "#t");
        assert_eq!(do_lisp("(eqv? 5/3 4/3)"), "#f");
        assert_eq!(do_lisp("(eqv? (+ 1 2) 9/3)"), "#t");
        assert_eq!(do_lisp("(eqv? 8/2 (+ 1 3))"), "#t");
        assert_eq!(do_lisp("(eqv? 1 1.0)"), "#f");
        assert_eq!(do_lisp("(eqv? 1/1 1.0)"), "#f");
        assert_eq!(do_lisp("(eqv? 1.0 1)"), "#f");

        assert_eq!(do_lisp("(eq? 'a 'a)"), "#t");
        assert_eq!(do_lisp("(eq? 'a 'b)"), "#f");
        assert_eq!(do_lisp("(eq? 'a 10)"), "#f");
        assert_eq!(do_lisp("(eq? #f #f)"), "#t");
        assert_eq!(do_lisp("(eq? #t #f)"), "#f");
        assert_eq!(do_lisp("(eq? #t 10)"), "#f");
        assert_eq!(do_lisp("(eq? #\\a #\\a)"), "#t");
        assert_eq!(do_lisp("(eq? #\\a #\\b)"), "#f");
        assert_eq!(do_lisp("(eq? #\\space #\\space)"), "#t");
        assert_eq!(do_lisp("(eq? \"abc\" \"abc\")"), "#t");
        assert_eq!(do_lisp("(eq? \"abc\" \"abc1\")"), "#f");
    }
    #[test]
    fn identity() {
        assert_eq!(do_lisp("(identity (+ 1 2 3))"), "6");
        assert_eq!(do_lisp("(identity ((lambda (a b) (+ a b)) 1 2))"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_eq!(do_lisp_env("(identity a)", &env), "100");
    }
    #[test]
    fn time_f() {
        let env = lisp::Environment::new();
        assert_eq!(do_lisp_env("(time (+ 10 20))", &env), "30");
    }
    #[test]
    fn get_env() {
        assert_eq!(
            do_lisp("(get-environment-variable \"HOME\")"),
            format!("\"{}\"", env::var("HOME").unwrap())
        );
    }
    #[test]
    fn native_endian() {
        let x: u32 = 1;
        if x == x.to_le() {
            assert_eq!(do_lisp("(native-endian)"), "little-endian");
        }
        if x == x.to_be() {
            assert_eq!(do_lisp("(native-endian)"), "big-endian");
        }
    }
    #[test]
    fn undefined_bool() {
        assert_eq!(do_lisp("(undefined? (if #t (display 1)(display 2)))"), "#t");
        assert_eq!(do_lisp("(undefined? 90)"), "#f");
    }
    #[test]
    fn undefined() {
        assert_eq!(do_lisp("(undefined)"), "nil");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn even() {
        assert_eq!(do_lisp("(even?)"), "E1007");
        assert_eq!(do_lisp("(even? 1 2)"), "E1007");
        assert_eq!(do_lisp("(even? 1/3)"), "E1002");
        assert_eq!(do_lisp("(even? 10.5)"), "E1002");
        assert_eq!(do_lisp("(even? a)"), "E1008");
    }
    #[test]
    fn odd() {
        assert_eq!(do_lisp("(odd?)"), "E1007");
        assert_eq!(do_lisp("(odd? 1 2)"), "E1007");
        assert_eq!(do_lisp("(odd? 1/3)"), "E1002");
        assert_eq!(do_lisp("(odd? 10.5)"), "E1002");
        assert_eq!(do_lisp("(odd? a)"), "E1008");
    }
    #[test]
    fn zero() {
        assert_eq!(do_lisp("(zero?)"), "E1007");
        assert_eq!(do_lisp("(zero? 1 2)"), "E1007");
        assert_eq!(do_lisp("(zero? #f)"), "E1003");
        assert_eq!(do_lisp("(zero? a)"), "E1008");
    }
    #[test]
    fn positive() {
        assert_eq!(do_lisp("(positive?)"), "E1007");
        assert_eq!(do_lisp("(positive? 1 2)"), "E1007");
        assert_eq!(do_lisp("(positive? #f)"), "E1003");
        assert_eq!(do_lisp("(positive? a)"), "E1008");
    }
    #[test]
    fn negative() {
        assert_eq!(do_lisp("(negative?)"), "E1007");
        assert_eq!(do_lisp("(negative? 1 2)"), "E1007");
        assert_eq!(do_lisp("(negative? #f)"), "E1003");
        assert_eq!(do_lisp("(negative? a)"), "E1008");
    }
    #[test]
    fn list_f() {
        assert_eq!(do_lisp("(list?)"), "E1007");
        assert_eq!(do_lisp("(list? (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(list? a)"), "E1008");
    }
    #[test]
    fn vector_f() {
        assert_eq!(do_lisp("(vector?)"), "E1007");
        assert_eq!(do_lisp("(vector? (vector 1)(vector 2))"), "E1007");
        assert_eq!(do_lisp("(vector? a)"), "E1008");
    }
    #[test]
    fn pair_f() {
        assert_eq!(do_lisp("(pair?)"), "E1007");
        assert_eq!(do_lisp("(pair? (cons 1 2)(cons 3 4))"), "E1007");
        assert_eq!(do_lisp("(pair? a)"), "E1008");
    }
    #[test]
    fn char_f() {
        assert_eq!(do_lisp("(char?)"), "E1007");
        assert_eq!(do_lisp("(char? #\\a #\\b)"), "E1007");
        assert_eq!(do_lisp("(char? a)"), "E1008");
    }
    #[test]
    fn string_f() {
        assert_eq!(do_lisp("(string?)"), "E1007");
        assert_eq!(do_lisp("(string? \"a\" \"b\")"), "E1007");
        assert_eq!(do_lisp("(string? a)"), "E1008");
    }
    #[test]
    fn integer_f() {
        assert_eq!(do_lisp("(integer?)"), "E1007");
        assert_eq!(do_lisp("(integer? 10 20)"), "E1007");
        assert_eq!(do_lisp("(integer? a)"), "E1008");
    }
    #[test]
    fn number_f() {
        assert_eq!(do_lisp("(number?)"), "E1007");
        assert_eq!(do_lisp("(number? 10 20)"), "E1007");
        assert_eq!(do_lisp("(number? a)"), "E1008");
    }
    #[test]
    fn symbol_f() {
        assert_eq!(do_lisp("(symbol?)"), "E1007");
        assert_eq!(do_lisp("(symbol? 10 20)"), "E1007");
        assert_eq!(do_lisp("(symbol? a)"), "E1008");
    }
    #[test]
    fn boolean_f() {
        assert_eq!(do_lisp("(boolean?)"), "E1007");
        assert_eq!(do_lisp("(boolean? 10 20)"), "E1007");
        assert_eq!(do_lisp("(boolean? a)"), "E1008");
    }
    #[test]
    fn procedure_f() {
        assert_eq!(do_lisp("(procedure?)"), "E1007");
        assert_eq!(
            do_lisp("(procedure? (lambda (n) n)(lambda (n) n))"),
            "E1007"
        );
        assert_eq!(do_lisp("(procedure? a)"), "E1008");
    }
    #[test]
    fn eqv() {
        assert_eq!(do_lisp("(eqv?)"), "E1007");
        assert_eq!(do_lisp("(eqv? 10 10 10)"), "E1007");
        assert_eq!(do_lisp("(eq? 10 10 10)"), "E1007");
        assert_eq!(do_lisp("(eq? 10 a)"), "E1008");
        assert_eq!(do_lisp("(eq? a 10)"), "E1008");
    }
    #[test]
    fn identity() {
        assert_eq!(do_lisp("(identity)"), "E1007");
        assert_eq!(do_lisp("(identity 10 20)"), "E1007");
        assert_eq!(do_lisp("(identity a)"), "E1008");
    }
    #[test]
    fn time_f() {
        let env = lisp::Environment::new();
        assert_eq!(do_lisp_env("(time)", &env), "E1007");
        assert_eq!(do_lisp_env("(time 10 10)", &env), "E1007");
        assert_eq!(do_lisp_env("(time c)", &env), "E1008");
    }
    #[test]
    fn get_env() {
        assert_eq!(do_lisp("(get-environment-variable)"), "E1007");
        assert_eq!(
            do_lisp("(get-environment-variable  \"HOME\"  \"HOME\")"),
            "E1007"
        );
        assert_eq!(do_lisp("(get-environment-variable a)"), "E1008");
        assert_eq!(do_lisp("(get-environment-variable #t)"), "E1015");
    }
    #[test]
    fn native_endian() {
        assert_eq!(do_lisp("(native-endian 1)"), "E1007");
    }
    #[test]
    fn undefined_bool() {
        assert_eq!(do_lisp("(undefined?)"), "E1007");
        assert_eq!(do_lisp("(undefined? (list 1)(list 2))"), "E1007");
        assert_eq!(do_lisp("(undefined? a)"), "E1008");
    }
    #[test]
    fn undefined() {
        assert_eq!(do_lisp("(undefined 1)"), "E1007");
    }
}
