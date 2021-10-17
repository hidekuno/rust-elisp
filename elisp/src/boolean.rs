/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  hidekuno@gmail.com
*/
use crate::buildin::BuildInTable;
use crate::create_error;
use crate::create_error_value;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("not", |exp, env| do_bool(exp, env, |b| !b));
    b.regist("boolean", |exp, env| do_bool(exp, env, |b| b));
    b.regist("boolean=?", boolean_eq);
}
fn do_bool(exp: &[Expression], env: &Environment, func: fn(x: bool) -> bool) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(Expression::Boolean(match eval(&exp[1], env)? {
        Expression::Boolean(b) => func(b),
        _ => func(true),
    }))
}
fn boolean_eq(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let a = match eval(&exp[1], env)? {
        Expression::Boolean(b) => b,
        _ => return Err(create_error!(ErrCode::E1001)),
    };
    for e in &exp[2..] {
        let b = match eval(e, env)? {
            Expression::Boolean(b) => b,
            _ => return Err(create_error!(ErrCode::E1001)),
        };
        if a != b {
            return Ok(Expression::Boolean(false));
        }
    }
    Ok(Expression::Boolean(true))
}
#[cfg(test)]
mod tests {
    use crate::do_lisp;
    #[test]
    fn not() {
        assert_eq!(do_lisp("(not (= 2 1))"), "#t");
        assert_eq!(do_lisp("(not (= 1 1))"), "#f");
        assert_eq!(do_lisp("(not 10)"), "#f");
        assert_eq!(do_lisp("(not \"abc\")"), "#f");
    }
    #[test]
    fn boolean() {
        assert_eq!(do_lisp("(boolean (= 1 1))"), "#t");
        assert_eq!(do_lisp("(boolean (= 2 1))"), "#f");
        assert_eq!(do_lisp("(boolean 10)"), "#t");
        assert_eq!(do_lisp("(boolean \"abc\")"), "#t");
    }
    #[test]
    fn boolean_eq() {
        assert_eq!(do_lisp("(boolean=? #t #t)"), "#t");
        assert_eq!(do_lisp("(boolean=? #f #f)"), "#t");
        assert_eq!(do_lisp("(boolean=? #t #f)"), "#f");
        assert_eq!(do_lisp("(boolean=? #t #t #t)"), "#t");
        assert_eq!(do_lisp("(boolean=? #f #f #f)"), "#t");
        assert_eq!(do_lisp("(boolean=? #f #f #t)"), "#f");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;

    #[test]
    fn not() {
        assert_eq!(do_lisp("(not)"), "E1007");
        assert_eq!(do_lisp("(not 1 2)"), "E1007");
        assert_eq!(do_lisp("(not a)"), "E1008");
    }
    #[test]
    fn boolean() {
        assert_eq!(do_lisp("(boolean)"), "E1007");
        assert_eq!(do_lisp("(boolean 1 2)"), "E1007");
        assert_eq!(do_lisp("(boolean a)"), "E1008");
    }
    #[test]
    fn boolean_eq() {
        assert_eq!(do_lisp("(boolean=?)"), "E1007");
        assert_eq!(do_lisp("(boolean=? #t)"), "E1007");
        assert_eq!(do_lisp("(boolean=? 10 #f)"), "E1001");
        assert_eq!(do_lisp("(boolean=? #t 10)"), "E1001");
    }
}
