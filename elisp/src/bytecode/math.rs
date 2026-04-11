/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn}; // ex.) export RUST_LOG=debug

use rand::Rng;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::{Environment, Expression, Int, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::number::Rat;

const SAMPLE_INT: Int = 10_000_000_000_000;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("sqrt", |exp, _| Ok(Expression::Float(to_f64(exp)?.sqrt())));
    b.regist("sin", |exp, _| Ok(Expression::Float(to_f64(exp)?.sin())));
    b.regist("cos", |exp, _| Ok(Expression::Float(to_f64(exp)?.cos())));
    b.regist("tan", |exp, _| Ok(Expression::Float(to_f64(exp)?.tan())));
    b.regist("asin", |exp, _| Ok(Expression::Float(to_f64(exp)?.asin())));
    b.regist("acos", |exp, _| Ok(Expression::Float(to_f64(exp)?.acos())));
    b.regist("atan", |exp, _| Ok(Expression::Float(to_f64(exp)?.atan())));
    b.regist("exp", |exp, _| Ok(Expression::Float(to_f64(exp)?.exp())));
    b.regist("log", |exp, _| {
        Ok(Expression::Float(to_f64(exp)?.log((1.0_f64).exp())))
    });
    b.regist("truncate", |exp, _| {
        Ok(Expression::Float(to_f64(exp)?.trunc()))
    });
    b.regist("floor", |exp, _| {
        Ok(Expression::Float(to_f64(exp)?.floor()))
    });
    b.regist("ceiling", |exp, _| {
        Ok(Expression::Float(to_f64(exp)?.ceil()))
    });
    b.regist("round", |exp, _| {
        Ok(Expression::Float(to_f64(exp)?.round()))
    });
    b.regist("abs", |exp, _| abs(exp));

    b.regist("rand-integer", |exp, _| rand_integer(exp));
    b.regist("rand-list", |exp, _| rand_list(exp));
    b.regist("expt", |exp, _| expt(exp));
}
fn to_f64(exp: &[Expression]) -> Result<f64, Box<Error>> {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::Float(f) => Ok(*f),
        Expression::Integer(i) => Ok(*i as f64),
        Expression::Rational(r) => Ok(r.div_float()),
        e => Err(create_error_value!(ErrCode::E1003, e)),
    }
}
fn abs(exp: &[Expression]) -> ResultExpression {
    if 1 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(match &exp[0] {
        Expression::Float(v) => Expression::Float(v.abs()),
        Expression::Integer(v) => Expression::Integer(v.abs()),
        Expression::Rational(v) => Expression::Rational(Box::new(v.abs())),
        e => return Err(create_error_value!(ErrCode::E1003, e)),
    })
}
fn rand_integer(exp: &[Expression]) -> ResultExpression {
    if !exp.is_empty() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: Int = rng.gen();
    Ok(Expression::Integer(x.abs() / SAMPLE_INT))
}
fn rand_list(exp: &[Expression]) -> ResultExpression {
    if 1 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::Integer(i) = &exp[0] {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::new();
        for _ in 0..(*i) {
            let x: Int = rng.gen();
            vec.push(Expression::Integer(x.abs() / SAMPLE_INT));
        }
        Ok(Environment::create_list(vec))
    } else {
        Err(create_error!(ErrCode::E1002))
    }
}
fn expt(exp: &[Expression]) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match &exp[0] {
        Expression::Float(x) => match &exp[1] {
            Expression::Float(y) => Ok(Expression::Float(x.powf(*y))),
            Expression::Integer(y) => Ok(Expression::Float(x.powf(*y as f64))),
            e => Err(create_error_value!(ErrCode::E1003, e)),
        },
        Expression::Integer(x) => match &exp[1] {
            Expression::Float(y) => Ok(Expression::Float((*x as f64).powf(*y))),
            Expression::Integer(y) => {
                if *y >= 0 {
                    Ok(Expression::Integer(x.pow(*y as u32)))
                } else {
                    Ok(Expression::Rational(Box::new(Rat::new(
                        1,
                        (*x).pow((*y).unsigned_abs() as u32),
                    ))))
                }
            }
            e => Err(create_error_value!(ErrCode::E1003, e)),
        },
        e => Err(create_error_value!(ErrCode::E1003, e)),
    }
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
    fn sqrt() {
        assert_eq!(do_lisp("(sqrt 9)"), "3");
        assert_eq!(do_lisp("(sqrt 25.0)"), "5");
        assert_eq!(do_lisp("(define a 16) (sqrt a)"), "4");
    }
    #[test]
    fn sin() {
        assert_eq!(
            do_lisp("(sin (/ (* 30 (* 4 (atan 1))) 180))"),
            "0.49999999999999994"
        );
        assert_eq!(
            do_lisp("(sin (/ (* 60 (* 4 (atan 1))) 180))"),
            "0.8660254037844386"
        );
    }
    #[test]
    fn cos() {
        assert_eq!(
            do_lisp("(cos (/ (* 30 (* 4 (atan 1))) 180))"),
            "0.8660254037844387"
        );
        assert_eq!(
            do_lisp("(cos (/ (* 60 (* 4 (atan 1))) 180))"),
            "0.5000000000000001"
        );
    }
    #[test]
    fn tan() {
        assert_eq!(
            do_lisp("(tan (/ (* 45 (* 4 (atan 1))) 180))"),
            "0.9999999999999999"
        );
    }
    #[test]
    fn asin() {
        assert_eq!(
            do_lisp("(round (/ (* (asin (/ (sqrt 3) 2)) 180) (* (atan 1) 4)))"),
            "60"
        );
    }
    #[test]
    fn acos() {
        assert_eq!(
            do_lisp("(round (/ (* (acos 0.5) 180) (* (atan 1) 4)))"),
            "60"
        );
    }
    #[test]
    fn atan() {
        assert_eq!(do_lisp("(round (/ (* (atan 1) 180) (* (atan 1) 4)))"), "45");
        assert_eq!(do_lisp("(* 4 (atan 1))"), "3.141592653589793");
        assert_eq!(do_lisp("(* 4 (atan 1.0))"), "3.141592653589793");
    }
    #[test]
    fn exp() {
        assert_eq!(do_lisp("(exp 1)"), "2.718281828459045");
        assert_eq!(do_lisp("(exp 2)"), "7.38905609893065");
        assert_eq!(do_lisp("(define a 3) (exp a)"), "20.085536923187668");
    }
    #[test]
    fn log() {
        assert_eq!(do_lisp("(/ (log 8) (log 2))"), "3");
        assert_eq!(do_lisp("(/ (log 9.0) (log 3.0))"), "2");
    }
    #[test]
    fn truncate() {
        assert_eq!(do_lisp("(truncate 3.7)"), "3");
        assert_eq!(do_lisp("(truncate 3.1)"), "3");
        assert_eq!(do_lisp("(truncate -3.1)"), "-3");
        assert_eq!(do_lisp("(truncate -3.7)"), "-3");
    }
    #[test]
    fn floor() {
        assert_eq!(do_lisp("(floor 3.7)"), "3");
        assert_eq!(do_lisp("(floor 3.1)"), "3");
        assert_eq!(do_lisp("(floor -3.1)"), "-4");
        assert_eq!(do_lisp("(floor -3.7)"), "-4");
    }
    #[test]
    fn ceiling() {
        assert_eq!(do_lisp("(ceiling 3.7)"), "4");
        assert_eq!(do_lisp("(ceiling 3.1)"), "4");
        assert_eq!(do_lisp("(ceiling -3.1)"), "-3");
        assert_eq!(do_lisp("(ceiling -3.7)"), "-3");
    }
    #[test]
    fn round() {
        assert_eq!(do_lisp("(round 3.7)"), "4");
        assert_eq!(do_lisp("(round 3.1)"), "3");
        assert_eq!(do_lisp("(round -3.1)"), "-3");
        assert_eq!(do_lisp("(round -3.7)"), "-4");
    }
    #[test]
    fn abs() {
        assert_eq!(do_lisp("(abs -20)"), "20");
        assert_eq!(do_lisp("(abs  20)"), "20");
        assert_eq!(do_lisp("(abs -1.5)"), "1.5");
        assert_eq!(do_lisp("(abs  1.5)"), "1.5");
        assert_eq!(do_lisp("(abs -1/3)"), "1/3");
        assert_eq!(do_lisp("(abs  1/3)"), "1/3");
    }
    #[test]
    fn rand_integer() {
        assert_eq!(do_lisp("(* 0 (rand-integer))"), "0");
    }
    #[test]
    fn rand_list() {
        assert_eq!(do_lisp("(length (rand-list 4))"), "4");
    }
    #[test]
    fn expt() {
        assert_eq!(do_lisp("(expt 2 3)"), "8");
        assert_eq!(do_lisp("(expt 2 (+ 1 2))"), "8");
        assert_eq!(do_lisp("(expt 2 -2)"), "1/4");
        assert_eq!(do_lisp("(expt 2 0)"), "1");
        assert_eq!(do_lisp("(expt 2.0 3.0)"), "8");
        assert_eq!(do_lisp("(expt 2.0 3)"), "8");
        assert_eq!(do_lisp("(expt 2 3.0)"), "8");
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
    fn sqrt() {
        assert_eq!(do_lisp("(sqrt)"), "E1007");
        assert_eq!(do_lisp("(sqrt 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(sqrt #t)"), "E1003");
    }
    #[test]
    fn sin() {
        assert_eq!(do_lisp("(sin)"), "E1007");
        assert_eq!(do_lisp("(sin 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(sin #t)"), "E1003");
    }
    #[test]
    fn cos() {
        assert_eq!(do_lisp("(cos)"), "E1007");
        assert_eq!(do_lisp("(cos 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(cos #t)"), "E1003");
    }
    #[test]
    fn tan() {
        assert_eq!(do_lisp("(tan)"), "E1007");
        assert_eq!(do_lisp("(tan 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(tan #t)"), "E1003");
    }
    #[test]
    fn asin() {
        assert_eq!(do_lisp("(asin)"), "E1007");
        assert_eq!(do_lisp("(asin 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(asin #t)"), "E1003");
    }
    #[test]
    fn acos() {
        assert_eq!(do_lisp("(acos)"), "E1007");
        assert_eq!(do_lisp("(acos 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(acos #t)"), "E1003");
    }
    #[test]
    fn atan() {
        assert_eq!(do_lisp("(atan)"), "E1007");
        assert_eq!(do_lisp("(atan 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(atan #t)"), "E1003");
    }
    #[test]
    fn exp() {
        assert_eq!(do_lisp("(exp)"), "E1007");
        assert_eq!(do_lisp("(exp 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(exp #t)"), "E1003");
    }
    #[test]
    fn log() {
        assert_eq!(do_lisp("(log)"), "E1007");
        assert_eq!(do_lisp("(log 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(log #t)"), "E1003");
    }
    #[test]
    fn truncate() {
        assert_eq!(do_lisp("(truncate)"), "E1007");
        assert_eq!(do_lisp("(truncate 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(truncate #t)"), "E1003");
    }
    #[test]
    fn floor() {
        assert_eq!(do_lisp("(floor)"), "E1007");
        assert_eq!(do_lisp("(floor 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(floor #t)"), "E1003");
    }
    #[test]
    fn ceiling() {
        assert_eq!(do_lisp("(ceiling)"), "E1007");
        assert_eq!(do_lisp("(ceiling 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(ceiling #t)"), "E1003");
    }
    #[test]
    fn round() {
        assert_eq!(do_lisp("(round)"), "E1007");
        assert_eq!(do_lisp("(round 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(round #t)"), "E1003");
    }
    #[test]
    fn abs() {
        assert_eq!(do_lisp("(abs)"), "E1007");
        assert_eq!(do_lisp("(abs 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(abs #t)"), "E1003");
    }
    #[test]
    fn rand_integer() {
        assert_eq!(do_lisp("(rand-integer 10)"), "E1007");
    }
    #[test]
    fn rand_list() {
        assert_eq!(do_lisp("(rand-list)"), "E1007");
        assert_eq!(do_lisp("(rand-list 1 2)"), "E1007");
        assert_eq!(do_lisp("(rand-list 10.5)"), "E1002");
    }
    #[test]
    fn expt() {
        assert_eq!(do_lisp("(expt 10)"), "E1007");
        assert_eq!(do_lisp("(expt 10 #f)"), "E1003");
        assert_eq!(do_lisp("(expt 10.5 #f)"), "E1003");
        assert_eq!(do_lisp("(expt #t 10)"), "E1003");
    }
}
