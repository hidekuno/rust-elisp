/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rand::Rng;
use std::vec::Vec;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::number::Rat;

const SAMPLE_INT: i64 = 10_000_000_000_000;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("sqrt", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sqrt()))
    });
    b.regist("sin", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.sin()))
    });
    b.regist("cos", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.cos()))
    });
    b.regist("tan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.tan()))
    });
    b.regist("asin", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.asin()))
    });
    b.regist("acos", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.acos()))
    });
    b.regist("atan", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.atan()))
    });
    b.regist("exp", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.exp()))
    });
    b.regist("log", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.log((1.0 as f64).exp())))
    });
    b.regist("truncate", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.trunc()))
    });
    b.regist("floor", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.floor()))
    });
    b.regist("ceiling", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.ceil()))
    });
    b.regist("round", |exp, env| {
        Ok(Expression::Float(to_f64(exp, env)?.round()))
    });
    b.regist("abs", abs);

    b.regist("rand-integer", rand_integer);
    b.regist("rand-list", rand_list);
    b.regist("expt", expt);
}
fn to_f64(exp: &[Expression], env: &Environment) -> Result<f64, Error> {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(f) => Ok(f),
        Expression::Integer(i) => Ok(i as f64),
        Expression::Rational(r) => Ok(r.div_float()),
        _ => Err(create_error!(ErrCode::E1003)),
    }
}
fn abs(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(match eval(&exp[1], env)? {
        Expression::Float(v) => Expression::Float(v.abs()),
        Expression::Integer(v) => Expression::Integer(v.abs()),
        Expression::Rational(v) => Expression::Rational(v.abs()),
        _ => return Err(create_error!(ErrCode::E1003)),
    })
}
fn rand_integer(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if 1 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    Ok(Expression::Integer(x.abs() / SAMPLE_INT))
}
fn rand_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::Integer(i) = eval(&exp[1], env)? {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::new();
        for _ in 0..i {
            let x: i64 = rng.gen();
            vec.push(Expression::Integer(x.abs() / SAMPLE_INT));
        }
        Ok(Expression::List(vec))
    } else {
        Err(create_error!(ErrCode::E1002))
    }
}
fn expt(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Float(x) => match eval(&exp[2], env)? {
            Expression::Float(y) => Ok(Expression::Float(x.powf(y))),
            Expression::Integer(y) => Ok(Expression::Float(x.powf(y as f64))),
            _ => Err(create_error!(ErrCode::E1003)),
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
            _ => Err(create_error!(ErrCode::E1003)),
        },
        _ => Err(create_error!(ErrCode::E1003)),
    }
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn sqrt() {
        assert_eq!(do_lisp("(sqrt 9)"), "3");
        assert_eq!(do_lisp("(sqrt 25.0)"), "5");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 16)", &env);
        assert_eq!(do_lisp_env("(sqrt a)", &env), "4");
    }
    #[test]
    fn sin() {
        assert_eq!(
            do_lisp("(sin (/(* 30 (* 4 (atan 1))) 180))"),
            "0.49999999999999994"
        );
        assert_eq!(
            do_lisp("(sin (/(* 30.025 (* 4 (atan 1))) 180))"),
            "0.5003778272590873"
        );
        assert_eq!(
            do_lisp("(sin (/(* 60 (* 4 (atan 1))) 180))"),
            "0.8660254037844386"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 30 (* 4 (atan 1))) 180))", &env);
        assert_eq!(do_lisp_env("(sin a)", &env), "0.49999999999999994");
    }
    #[test]
    fn cos() {
        assert_eq!(
            do_lisp("(cos (/(* 30 (* 4 (atan 1))) 180))"),
            "0.8660254037844387"
        );
        assert_eq!(
            do_lisp("(cos (/(* 60 (* 4 (atan 1))) 180))"),
            "0.5000000000000001"
        );
        assert_eq!(
            do_lisp("(cos (/(* 59.725 (* 4 (atan 1))) 180))"),
            "0.5041508484218754"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 60 (* 4 (atan 1))) 180))", &env);
        assert_eq!(do_lisp_env("(cos a)", &env), "0.5000000000000001");
    }
    #[test]
    fn tan() {
        assert_eq!(
            do_lisp("(tan (/(* 45 (* 4 (atan 1))) 180))"),
            "0.9999999999999999"
        );
        assert_eq!(
            do_lisp("(tan (/(* 45.5 (* 4 (atan 1))) 180))"),
            "1.0176073929721252"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 45 (* 4 (atan 1))) 180))", &env);
        assert_eq!(do_lisp_env("(tan a)", &env), "0.9999999999999999");
    }
    #[test]
    fn asin() {
        assert_eq!(
            do_lisp("(round (/ (* (asin (/(sqrt 3) 2)) 180)(*(atan 1)4)))"),
            "60"
        );
        assert_eq!(
            do_lisp("(sin (asin (/(* pi 30)180)))"),
            do_lisp("(/(* pi 30)180)")
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* pi 30)180))", &env);
        assert_eq!(do_lisp_env("(sin (asin a))", &env), do_lisp_env("a", &env));
    }
    #[test]
    fn acos() {
        assert_eq!(
            do_lisp("(round (/ (* (acos (/ 1 2)) 180)(*(atan 1)4)))"),
            "60"
        );
        assert_eq!(
            do_lisp("(cos (acos (/(* pi 30)180)))"),
            do_lisp("(/(* pi 30)180)")
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* pi 30)180))", &env);
        assert_eq!(do_lisp_env("(cos (acos a))", &env), do_lisp_env("a", &env));
    }
    #[test]
    fn atan() {
        assert_eq!(do_lisp("(round (/(* (atan 1) 180)(*(atan 1)4)))"), "45");
        assert_eq!(do_lisp("(* 4 (atan 1))"), "3.141592653589793");
        assert_eq!(do_lisp("(* 4 (atan 1.0))"), "3.141592653589793");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 1)", &env);
        assert_eq!(do_lisp_env("(* 4 (atan a))", &env), "3.141592653589793");
    }
    #[test]
    fn exp() {
        assert_eq!(do_lisp("(exp 1)"), "2.718281828459045");
        assert_eq!(do_lisp("(exp 1.025)"), "2.7870954605658507");
        assert_eq!(do_lisp("(exp 2)"), "7.38905609893065");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 3)", &env);
        assert_eq!(do_lisp_env("(exp a)", &env), "20.085536923187668");
    }
    #[test]
    fn log() {
        assert_eq!(do_lisp("(/(log 8)(log 2))"), "3");
        assert_eq!(do_lisp("(/(log 9.0)(log 3.0))"), "2");
        assert_eq!(do_lisp("(exp (/(log 8) 3))"), "2");
        assert_eq!(do_lisp("(round (exp (* (log 2) 3)))"), "8");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 9)", &env);
        do_lisp_env("(define b 3)", &env);
        assert_eq!(do_lisp_env("(/(log a)(log b))", &env), "2");
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

        let env = lisp::Environment::new();
        do_lisp_env("(define a -20)", &env);
        do_lisp_env("(define b -1.5)", &env);
        assert_eq!(do_lisp_env("(+ (abs a)(abs b))", &env), "21.5");
    }
    #[test]
    fn rand_integer() {
        assert_eq!(do_lisp("(integer? (rand-integer))"), "#t");
        assert_eq!(do_lisp("(* 0 (rand-integer))"), "0");
    }
    #[test]
    fn rand_list() {
        assert_eq!(do_lisp("(length (rand-list 4))"), "4");
        assert_eq!(
            do_lisp("(map (lambda (n) (integer? n)) (rand-list 4))"),
            "(#t #t #t #t)"
        );
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
    use crate::do_lisp;

    #[test]
    fn sqrt() {
        assert_eq!(do_lisp("(sqrt)"), "E1007");
        assert_eq!(do_lisp("(sqrt 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(sqrt #t)"), "E1003");
        assert_eq!(do_lisp("(sqrt a)"), "E1008");
    }
    #[test]
    fn sin() {
        assert_eq!(do_lisp("(sin)"), "E1007");
        assert_eq!(do_lisp("(sin 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(sin #t)"), "E1003");
        assert_eq!(do_lisp("(sin a)"), "E1008");
    }
    #[test]
    fn cos() {
        assert_eq!(do_lisp("(cos)"), "E1007");
        assert_eq!(do_lisp("(cos 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(cos #t)"), "E1003");
        assert_eq!(do_lisp("(cos a)"), "E1008");
    }
    #[test]
    fn tan() {
        assert_eq!(do_lisp("(tan)"), "E1007");
        assert_eq!(do_lisp("(tan 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(tan #t)"), "E1003");
        assert_eq!(do_lisp("(tan a)"), "E1008");
    }
    #[test]
    fn asin() {
        assert_eq!(do_lisp("(asin)"), "E1007");
        assert_eq!(do_lisp("(asin 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(asin #t)"), "E1003");
        assert_eq!(do_lisp("(asin a)"), "E1008");
    }
    #[test]
    fn acos() {
        assert_eq!(do_lisp("(acos)"), "E1007");
        assert_eq!(do_lisp("(acos 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(acos #t)"), "E1003");
        assert_eq!(do_lisp("(acos a)"), "E1008");
    }
    #[test]
    fn atan() {
        assert_eq!(do_lisp("(atan)"), "E1007");
        assert_eq!(do_lisp("(atan 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(atan #t)"), "E1003");
        assert_eq!(do_lisp("(atan a)"), "E1008");
    }
    #[test]
    fn exp() {
        assert_eq!(do_lisp("(exp)"), "E1007");
        assert_eq!(do_lisp("(exp 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(exp #t)"), "E1003");
        assert_eq!(do_lisp("(exp a)"), "E1008");
    }
    #[test]
    fn log() {
        assert_eq!(do_lisp("(log)"), "E1007");
        assert_eq!(do_lisp("(log 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(log #t)"), "E1003");
        assert_eq!(do_lisp("(log a)"), "E1008");
    }
    #[test]
    fn truncate() {
        assert_eq!(do_lisp("(truncate)"), "E1007");
        assert_eq!(do_lisp("(truncate 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(truncate #t)"), "E1003");
        assert_eq!(do_lisp("(truncate a)"), "E1008");
    }
    #[test]
    fn floor() {
        assert_eq!(do_lisp("(floor)"), "E1007");
        assert_eq!(do_lisp("(floor 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(floor #t)"), "E1003");
        assert_eq!(do_lisp("(floor a)"), "E1008");
    }
    #[test]
    fn ceiling() {
        assert_eq!(do_lisp("(ceiling)"), "E1007");
        assert_eq!(do_lisp("(ceiling 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(ceiling #t)"), "E1003");
        assert_eq!(do_lisp("(ceiling a)"), "E1008");
    }
    #[test]
    fn round() {
        assert_eq!(do_lisp("(round)"), "E1007");
        assert_eq!(do_lisp("(round 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(round #t)"), "E1003");
        assert_eq!(do_lisp("(round a)"), "E1008");
    }
    #[test]
    fn abs() {
        assert_eq!(do_lisp("(abs)"), "E1007");
        assert_eq!(do_lisp("(abs 10 2.5)"), "E1007");
        assert_eq!(do_lisp("(abs #t)"), "E1003");
        assert_eq!(do_lisp("(abs a)"), "E1008");
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
        assert_eq!(do_lisp("(expt a 2)"), "E1008");
        assert_eq!(do_lisp("(expt 10 #f)"), "E1003");
        assert_eq!(do_lisp("(expt 10.5 #f)"), "E1003");
        assert_eq!(do_lisp("(expt #t 10)"), "E1003");
    }
}
