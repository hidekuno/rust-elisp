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
use crate::lisp::{Environment, Expression, Int, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::strings::do_radix;

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
    b.regist("char-alphabetic?", |exp, env| {
        char_kind(exp, env, |x| Expression::Boolean(x.is_alphabetic()))
    });
    b.regist("char-numeric?", |exp, env| {
        char_kind(exp, env, |x| Expression::Boolean(x.is_numeric()))
    });
    b.regist("char-whitespace?", |exp, env| {
        char_kind(exp, env, |x| Expression::Boolean(x.is_whitespace()))
    });
    b.regist("char-upper-case?", |exp, env| {
        char_kind(exp, env, |x| Expression::Boolean(x.is_uppercase()))
    });
    b.regist("char-lower-case?", |exp, env| {
        char_kind(exp, env, |x| Expression::Boolean(x.is_lowercase()))
    });

    b.regist("integer->char", integer_char);
    b.regist("char->integer", char_integer);

    b.regist("char-upcase", |exp, env| {
        //convert_char(exp, env, |x| x.to_ascii_uppercase())
        char_kind(exp, env, |x| {
            Expression::Char(x.to_uppercase().collect::<Vec<char>>()[0])
        })
    });
    b.regist("char-downcase", |exp, env| {
        //convert_char(exp, env, |x| x.to_ascii_lowercase())
        char_kind(exp, env, |x| {
            Expression::Char(x.to_lowercase().collect::<Vec<char>>()[0])
        })
    });

    b.regist("digit->integer", |exp, env| {
        do_radix(exp, env, digit_integer)
    });
    b.regist("integer->digit", |exp, env| {
        do_radix(exp, env, integer_digit)
    });
}
fn charcmp(
    exp: &[Expression],
    env: &Environment,
    func: fn(x: char, y: char) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v: [char; 2] = [' '; 2];

    for (i, e) in exp[1..].iter().enumerate() {
        v[i] = match eval(e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!(ErrCode::E1019)),
        }
    }
    Ok(Expression::Boolean(func(v[0], v[1])))
}
fn char_kind(
    exp: &[Expression],
    env: &Environment,
    func: fn(x: char) -> Expression,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let c = match eval(&exp[1], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(ErrCode::E1019)),
    };
    Ok(func(c))
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
    Ok(Expression::Integer(a as Int))
}
fn digit_integer(exp: &Expression, env: &Environment, r: u32) -> ResultExpression {
    let c = match eval(exp, env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(ErrCode::E1019)),
    };
    match c.to_digit(r as u32) {
        Some(i) => Ok(Expression::Integer(i as Int)),
        None => Ok(Expression::Boolean(false)),
    }
}
fn integer_digit(exp: &Expression, env: &Environment, r: u32) -> ResultExpression {
    let i = match eval(exp, env)? {
        Expression::Integer(c) => c,
        _ => return Err(create_error!(ErrCode::E1002)),
    };
    match char::from_digit(i as u32, r as u32) {
        Some(c) => Ok(Expression::Char(c)),
        None => Ok(Expression::Boolean(false)),
    }
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
        assert_eq!(do_lisp("(char-ci=? #\\a #\\B)"), "#f");
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
    fn char_alphabetic() {
        assert_eq!(do_lisp("(char-alphabetic? #\\a)"), "#t");
        assert_eq!(do_lisp("(char-alphabetic? #\\A)"), "#t");
        assert_eq!(do_lisp("(char-alphabetic? #\\0)"), "#f");
        assert_eq!(do_lisp("(char-alphabetic? #\\9)"), "#f");
    }
    #[test]
    fn char_numeric() {
        assert_eq!(do_lisp("(char-numeric? #\\0)"), "#t");
        assert_eq!(do_lisp("(char-numeric? #\\9)"), "#t");
        assert_eq!(do_lisp("(char-numeric? #\\a)"), "#f");
        assert_eq!(do_lisp("(char-numeric? #\\A)"), "#f");
    }
    #[test]
    fn char_whitespace() {
        assert_eq!(do_lisp("(char-whitespace? #\\space)"), "#t");
        assert_eq!(do_lisp("(char-whitespace? #\\tab)"), "#t");
        assert_eq!(do_lisp("(char-whitespace? #\\newline)"), "#t");
        assert_eq!(do_lisp("(char-whitespace? #\\return)"), "#t");

        assert_eq!(do_lisp("(char-whitespace? #\\0)"), "#f");
        assert_eq!(do_lisp("(char-whitespace? #\\9)"), "#f");
        assert_eq!(do_lisp("(char-whitespace? #\\a)"), "#f");
        assert_eq!(do_lisp("(char-whitespace? #\\A)"), "#f");
    }
    #[test]
    fn char_upper_case() {
        assert_eq!(do_lisp("(char-upper-case? #\\A)"), "#t");
        assert_eq!(do_lisp("(char-upper-case? #\\a)"), "#f");
        assert_eq!(do_lisp("(char-upper-case? #\\0)"), "#f");
        assert_eq!(do_lisp("(char-upper-case? #\\9)"), "#f");
    }
    #[test]
    fn char_lower_case() {
        assert_eq!(do_lisp("(char-lower-case? #\\a)"), "#t");
        assert_eq!(do_lisp("(char-lower-case? #\\A)"), "#f");
        assert_eq!(do_lisp("(char-lower-case? #\\0)"), "#f");
        assert_eq!(do_lisp("(char-lower-case? #\\9)"), "#f");
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
    #[test]
    fn char_upcase() {
        assert_eq!(do_lisp("(char-upcase #\\a)"), "#\\A");
        assert_eq!(do_lisp("(char-upcase #\\A)"), "#\\A");
        assert_eq!(do_lisp("(char-upcase #\\0)"), "#\\0");
        assert_eq!(do_lisp("(char-upcase #\\9)"), "#\\9");
    }
    #[test]
    fn char_downcase() {
        assert_eq!(do_lisp("(char-downcase #\\a)"), "#\\a");
        assert_eq!(do_lisp("(char-downcase #\\A)"), "#\\a");
        assert_eq!(do_lisp("(char-downcase #\\0)"), "#\\0");
        assert_eq!(do_lisp("(char-downcase #\\9)"), "#\\9");
    }
    #[test]
    fn digit_integer() {
        assert_eq!(do_lisp("(digit->integer #\\0)"), "0");
        assert_eq!(do_lisp("(digit->integer #\\8)"), "8");
        assert_eq!(do_lisp("(digit->integer #\\9 10)"), "9");
        assert_eq!(do_lisp("(digit->integer #\\7 8)"), "7");
        assert_eq!(do_lisp("(digit->integer #\\a 16)"), "10");
        assert_eq!(do_lisp("(digit->integer #\\f 16)"), "15");

        assert_eq!(do_lisp("(digit->integer #\\a 10)"), "#f");
        assert_eq!(do_lisp("(digit->integer #\\8 8)"), "#f");
        assert_eq!(do_lisp("(digit->integer #\\g 16)"), "#f");
    }
    #[test]
    fn integer_digit() {
        assert_eq!(do_lisp("(integer->digit 0)"), "#\\0");
        assert_eq!(do_lisp("(integer->digit 8)"), "#\\8");
        assert_eq!(do_lisp("(integer->digit 9 10)"), "#\\9");
        assert_eq!(do_lisp("(integer->digit 7 8)"), "#\\7");
        assert_eq!(do_lisp("(integer->digit 13 16)"), "#\\d");
        assert_eq!(do_lisp("(integer->digit 15 16)"), "#\\f");

        assert_eq!(do_lisp("(integer->digit 10 10)"), "#f");
        assert_eq!(do_lisp("(integer->digit 8 8)"), "#f");
        assert_eq!(do_lisp("(integer->digit 16 16)"), "#f");
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
    fn char_alphabetic() {
        assert_eq!(do_lisp("(char-alphabetic?)"), "E1007");
        assert_eq!(do_lisp("(char-alphabetic? #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-alphabetic? a)"), "E1008");
        assert_eq!(do_lisp("(char-alphabetic? 10)"), "E1019");
    }
    #[test]
    fn char_numeric() {
        assert_eq!(do_lisp("(char-numeric?)"), "E1007");
        assert_eq!(do_lisp("(char-numeric? #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-numeric? a)"), "E1008");
        assert_eq!(do_lisp("(char-numeric? 10)"), "E1019");
    }
    #[test]
    fn char_whitespace() {
        assert_eq!(do_lisp("(char-whitespace?)"), "E1007");
        assert_eq!(do_lisp("(char-whitespace? #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-whitespace? a)"), "E1008");
        assert_eq!(do_lisp("(char-whitespace? 10)"), "E1019");
    }
    #[test]
    fn char_upper_case() {
        assert_eq!(do_lisp("(char-upper-case?)"), "E1007");
        assert_eq!(do_lisp("(char-upper-case? #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-upper-case? a)"), "E1008");
        assert_eq!(do_lisp("(char-upper-case? 10)"), "E1019");
    }
    #[test]
    fn char_lower_case() {
        assert_eq!(do_lisp("(char-lower-case?)"), "E1007");
        assert_eq!(do_lisp("(char-lower-case? #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-lower-case? a)"), "E1008");
        assert_eq!(do_lisp("(char-lower-case? 10)"), "E1019");
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
    #[test]
    fn char_upcase() {
        assert_eq!(do_lisp("(char-upcase)"), "E1007");
        assert_eq!(do_lisp("(char-upcase #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-upcase a)"), "E1008");
        assert_eq!(do_lisp("(char-upcase 10)"), "E1019");
    }
    #[test]
    fn char_downcase() {
        assert_eq!(do_lisp("(char-downcase)"), "E1007");
        assert_eq!(do_lisp("(char-downcase #\\0 #\\9)"), "E1007");
        assert_eq!(do_lisp("(char-downcase a)"), "E1008");
        assert_eq!(do_lisp("(char-downcase 10)"), "E1019");
    }
    #[test]
    fn digit_integer() {
        assert_eq!(do_lisp("(digit->integer)"), "E1007");
        assert_eq!(do_lisp("(digit->integer 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(digit->integer #\\8 #t)"), "E1002");
        assert_eq!(do_lisp("(digit->integer #\\8 1)"), "E1021");
        assert_eq!(do_lisp("(digit->integer #\\8 37)"), "E1021");
        assert_eq!(do_lisp("(digit->integer 10 10)"), "E1019");
    }
    #[test]
    fn integer_digit() {
        assert_eq!(do_lisp("(integer->digit)"), "E1007");
        assert_eq!(do_lisp("(integer->digit 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(integer->digit 8 #t)"), "E1002");
        assert_eq!(do_lisp("(integer->digit 8 1)"), "E1021");
        assert_eq!(do_lisp("(integer->digit 8 37)"), "E1021");
        assert_eq!(do_lisp("(integer->digit #\\8 10)"), "E1002");
    }
}
