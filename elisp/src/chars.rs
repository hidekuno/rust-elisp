/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::char;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("char=?", |exp, env| charcmp(exp, env, |x, y| x == y));
    b.regist("char<?", |exp, env| charcmp(exp, env, |x, y| x < y));
    b.regist("char>?", |exp, env| charcmp(exp, env, |x, y| x > y));
    b.regist("char<=?", |exp, env| charcmp(exp, env, |x, y| x <= y));
    b.regist("char>=?", |exp, env| charcmp(exp, env, |x, y| x >= y));

    b.regist("char-ci=?", |exp, env| {
        charcmp(exp, env, |x, y| x.to_lowercase().eq(y.to_lowercase()))
    });
    b.regist("char-ci<?", |exp, env| {
        charcmp(exp, env, |x, y| x.to_lowercase().lt(y.to_lowercase()))
    });
    b.regist("char-ci>?", |exp, env| {
        charcmp(exp, env, |x, y| x.to_lowercase().gt(y.to_lowercase()))
    });
    b.regist("char-ci<=?", |exp, env| {
        charcmp(exp, env, |x, y| x.to_lowercase().le(y.to_lowercase()))
    });
    b.regist("char-ci>=?", |exp, env| {
        charcmp(exp, env, |x, y| x.to_lowercase().ge(y.to_lowercase()))
    });

    b.regist("integer->char", integer_char);
    b.regist("char->integer", char_integer);
}
fn charcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: char, y: char) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v: [char; 2] = [' '; 2];

    for (i, e) in exp[1 as usize..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!(ErrCode::E1019)),
        }
    }
    Ok(Expression::Boolean(f(v[0], v[1])))
}
fn integer_char(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let i = match eval(&exp[1], env)? {
        Expression::Integer(i) => i,
        _ => return Err(create_error!(ErrCode::E1002)),
    };
    let i = i as u32;
    if let Some(c) = char::from_u32(i) {
        Ok(Expression::Char(c))
    } else {
        Err(create_error!(ErrCode::E1019))
    }
}
fn char_integer(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let c = match eval(&exp[1], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(ErrCode::E1019)),
    };
    let a = c as u32;
    Ok(Expression::Integer(a as i64))
}
#[cfg(test)]
mod tests {
    use crate::do_lisp;

    #[test]
    fn char_eq() {
        assert_eq!(do_lisp("(char=? #\\a #\\a)"), "#t");
        assert_eq!(do_lisp("(char=? #\\a #\\b)"), "#f");
    }
    #[test]
    fn char_less() {
        assert_eq!(do_lisp("(char<? #\\a #\\b)"), "#t");
        assert_eq!(do_lisp("(char<? #\\b #\\a)"), "#f");
    }
    #[test]
    fn char_than() {
        assert_eq!(do_lisp("(char>? #\\b #\\a)"), "#t");
        assert_eq!(do_lisp("(char>? #\\a #\\b)"), "#f");
    }
    #[test]
    fn char_le() {
        assert_eq!(do_lisp("(char<=? #\\a #\\b)"), "#t");
        assert_eq!(do_lisp("(char<=? #\\a #\\a)"), "#t");
        assert_eq!(do_lisp("(char<=? #\\b #\\a)"), "#f");
    }
    #[test]
    fn char_ge() {
        assert_eq!(do_lisp("(char>=? #\\b #\\a)"), "#t");
        assert_eq!(do_lisp("(char>=? #\\a #\\a)"), "#t");
        assert_eq!(do_lisp("(char>=? #\\a #\\b)"), "#f");
    }
    #[test]
    fn char_ci_eq() {
        assert_eq!(do_lisp("(char-ci=? #\\a #\\A)"), "#t");
        assert_eq!(do_lisp("(char-ci=? #\\A #\\a)"), "#t");
        assert_eq!(do_lisp("(char=? #\\a #\\B)"), "#f");
    }
    #[test]
    fn char_ci_less() {
        assert_eq!(do_lisp("(char-ci<? #\\a #\\C)"), "#t");
        assert_eq!(do_lisp("(char-ci<? #\\C #\\a)"), "#f");
    }
    #[test]
    fn char_ci_than() {
        assert_eq!(do_lisp("(char-ci>? #\\C #\\a)"), "#t");
        assert_eq!(do_lisp("(char-ci>? #\\a #\\C)"), "#f");
    }
    #[test]
    fn char_ci_le() {
        assert_eq!(do_lisp("(char-ci<=? #\\a #\\C)"), "#t");
        assert_eq!(do_lisp("(char-ci<=? #\\C #\\C)"), "#t");
        assert_eq!(do_lisp("(char-ci<=? #\\C #\\a)"), "#f");
    }
    #[test]
    fn char_ci_ge() {
        assert_eq!(do_lisp("(char-ci>=? #\\C #\\a)"), "#t");
        assert_eq!(do_lisp("(char-ci>=? #\\C #\\C)"), "#t");
        assert_eq!(do_lisp("(char-ci>=? #\\a #\\C)"), "#f");
    }
    #[test]
    fn integer_char() {
        assert_eq!(do_lisp("(integer->char 65)"), "#\\A");
        assert_eq!(do_lisp("(integer->char 23665)"), "#\\山");
    }
    #[test]
    fn char_integer() {
        assert_eq!(do_lisp("(char->integer #\\A)"), "65");
        assert_eq!(do_lisp("(char->integer #\\山)"), "23665");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;

    #[test]
    fn char_eq() {
        assert_eq!(do_lisp("(char=?)"), "E1007");
        assert_eq!(do_lisp("(char=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_less() {
        assert_eq!(do_lisp("(char<?)"), "E1007");
        assert_eq!(do_lisp("(char<? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char<? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char<? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char<? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char<? #\\a a)"), "E1008");
    }
    #[test]
    fn char_than() {
        assert_eq!(do_lisp("(char>?)"), "E1007");
        assert_eq!(do_lisp("(char>? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char>? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char>? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char>? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char>? #\\a a)"), "E1008");
    }
    #[test]
    fn char_le() {
        assert_eq!(do_lisp("(char<=?)"), "E1007");
        assert_eq!(do_lisp("(char<=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char<=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char<=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char<=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char<=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ge() {
        assert_eq!(do_lisp("(char>=?)"), "E1007");
        assert_eq!(do_lisp("(char>=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char>=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char>=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char>=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char>=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ci_eq() {
        assert_eq!(do_lisp("(char-ci=?)"), "E1007");
        assert_eq!(do_lisp("(char-ci=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char-ci=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char-ci=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char-ci=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char-ci=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ci_less() {
        assert_eq!(do_lisp("(char-ci<?)"), "E1007");
        assert_eq!(do_lisp("(char-ci<? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char-ci<? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char-ci<? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char-ci<? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char-ci<? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ci_than() {
        assert_eq!(do_lisp("(char-ci>?)"), "E1007");
        assert_eq!(do_lisp("(char-ci>? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char-ci>? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char-ci>? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char-ci>? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char-ci>? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ci_le() {
        assert_eq!(do_lisp("(char-ci<=?)"), "E1007");
        assert_eq!(do_lisp("(char-ci<=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char-ci<=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char-ci<=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char-ci<=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char-ci<=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ci_ge() {
        assert_eq!(do_lisp("(char-ci>=?)"), "E1007");
        assert_eq!(do_lisp("(char-ci>=? #\\a)"), "E1007");
        assert_eq!(do_lisp("(char-ci>=? #\\a #\\b #\\c)"), "E1007");
        assert_eq!(do_lisp("(char-ci>=? #\\a 10)"), "E1019");
        assert_eq!(do_lisp("(char-ci>=? 10 #\\a)"), "E1019");
        assert_eq!(do_lisp("(char-ci>=? #\\a a)"), "E1008");
    }
    #[test]
    fn integer_char() {
        assert_eq!(do_lisp("(integer->char)"), "E1007");
        assert_eq!(do_lisp("(integer->char 23 665)"), "E1007");
        assert_eq!(do_lisp("(integer->char #\\a)"), "E1002");
        assert_eq!(do_lisp("(integer->char -999)"), "E1019");
        assert_eq!(do_lisp("(integer->char a)"), "E1008");
    }
    #[test]
    fn char_integer() {
        assert_eq!(do_lisp("(char->integer)"), "E1007");
        assert_eq!(do_lisp("(char->integer #\\a #\\b)"), "E1007");
        assert_eq!(do_lisp("(char->integer 999)"), "E1019");
        assert_eq!(do_lisp("(char->integer a)"), "E1008");
    }
}
