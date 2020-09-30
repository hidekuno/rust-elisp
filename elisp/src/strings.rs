/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::create_error;
use crate::create_error_value;
use crate::referlence_list;

use std::vec::Vec;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::number::Rat;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("format", format_f);

    b.regist("string=?", |exp, env| strcmp(exp, env, |x, y| x == y));
    b.regist("string<?", |exp, env| strcmp(exp, env, |x, y| x < y));
    b.regist("string>?", |exp, env| strcmp(exp, env, |x, y| x > y));
    b.regist("string<=?", |exp, env| strcmp(exp, env, |x, y| x <= y));
    b.regist("string>=?", |exp, env| strcmp(exp, env, |x, y| x >= y));

    b.regist("string-ci=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() == y.to_lowercase())
    });
    b.regist("string-ci<?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() < y.to_lowercase())
    });
    b.regist("string-ci>?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() > y.to_lowercase())
    });
    b.regist("string-ci<=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() <= y.to_lowercase())
    });
    b.regist("string-ci>=?", |exp, env| {
        strcmp(exp, env, |x, y| x.to_lowercase() >= y.to_lowercase())
    });

    b.regist("string-append", str_append);
    b.regist("string-length", |exp, env| {
        str_length(exp, env, |s| s.chars().count())
    });
    b.regist("string-size", |exp, env| str_length(exp, env, |s| s.len()));
    b.regist("number->string", number_string);
    b.regist("string->number", string_number);
    b.regist("list->string", list_string);
    b.regist("string->list", string_list);

    b.regist("substring", substring);
    b.regist("symbol->string", symbol_string);
    b.regist("string->symbol", string_symbol);
    b.regist("make-string", make_string);
}
fn format_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = if let Expression::String(s) = eval(&exp[1], env)? {
        s
    } else {
        return Err(create_error!(ErrCode::E1015));
    };
    let i = if let Expression::Integer(i) = eval(&exp[2], env)? {
        i
    } else {
        return Err(create_error!(ErrCode::E1002));
    };
    let s = match s.as_str() {
        "~X" => format!("{:X}", i),
        "~x" => format!("{:x}", i),
        n => match n.to_lowercase().as_str() {
            "~d" => format!("{:?}", i),
            "~o" => format!("{:o}", i),
            "~b" => format!("{:b}", i),
            _ => return Err(create_error!(ErrCode::E1018)),
        },
    };
    Ok(Expression::String(s))
}
fn strcmp(
    exp: &[Expression],
    env: &Environment,
    f: fn(x: &String, y: &String) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v = Vec::new();
    for e in &exp[1 as usize..] {
        let s = match eval(e, env)? {
            Expression::String(s) => s,
            _ => return Err(create_error!(ErrCode::E1015)),
        };
        v.push(s);
    }
    Ok(Expression::Boolean(f(&v[0], &v[1])))
}
fn str_append(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 > exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v = String::new();
    for e in &exp[1 as usize..] {
        match eval(e, env)? {
            Expression::String(s) => v.push_str(&s.into_boxed_str()),
            _ => return Err(create_error!(ErrCode::E1015)),
        };
    }
    Ok(Expression::String(v))
}
fn str_length(
    exp: &[Expression],
    env: &Environment,
    f: fn(s: String) -> usize,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Integer(f(s) as i64)),
        _ => return Err(create_error!(ErrCode::E1015)),
    }
}
fn number_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = match eval(&exp[1], env)? {
        Expression::Float(f) => Expression::Float(f),
        Expression::Integer(i) => Expression::Integer(i),
        Expression::Rational(r) => Expression::Rational(r),
        _ => return Err(create_error!(ErrCode::E1003)),
    };
    Ok(Expression::String(v.to_string()))
}
fn string_number(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(ErrCode::E1015)),
    };
    let v = if let Ok(n) = s.parse::<i64>() {
        Expression::Integer(n)
    } else if let Ok(n) = s.parse::<f64>() {
        Expression::Float(n)
    } else {
        match Rat::from(&s) {
            Ok(n) => Expression::Rational(n),
            Err(n) => {
                return if n.code != ErrCode::E1020 {
                    return Err(create_error!(n.code));
                } else {
                    Err(create_error!(ErrCode::E1003))
                }
            }
        }
    };
    Ok(v)
}
fn list_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        _ => return Err(create_error!(ErrCode::E1005)),
    };

    let l = &*(referlence_list!(l));
    let mut v = String::new();

    for e in l.into_iter() {
        v.push(match eval(&e, env)? {
            Expression::Char(c) => c,
            _ => return Err(create_error!(ErrCode::E1019)),
        });
    }
    Ok(Expression::String(v))
}
fn string_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(ErrCode::E1015)),
    };
    let mut l: Vec<Expression> = Vec::new();
    for c in s.as_str().chars() {
        l.push(Expression::Char(c));
    }
    Ok(Environment::create_list(l))
}
fn substring(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 4 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        _ => return Err(create_error!(ErrCode::E1015)),
    };
    let mut param: [usize; 2] = [0; 2];
    for (i, e) in exp[2..].iter().enumerate() {
        let v = match eval(e, env)? {
            Expression::Integer(v) => v,
            _ => return Err(create_error!(ErrCode::E1002)),
        };
        if 0 > v {
            return Err(create_error!(ErrCode::E1021));
        }
        param[i] = v as usize;
    }
    let (start, end) = (param[0], param[1]);
    if s.chars().count() < end {
        return Err(create_error!(ErrCode::E1021));
    }
    if start > end {
        return Err(create_error!(ErrCode::E1021));
    }
    // the trait `std::convert::From<str>` is not implemented for `std::string::String`
    // s.as_str()[start..end].to_string()),
    //     => panicked at 'byte index 1 is not a char boundary; it is inside '山' (bytes 0..3)
    let mut v = String::new();
    for c in &s.chars().collect::<Vec<char>>()[start..end] {
        v.push(*c);
    }
    Ok(Expression::String(v))
}
fn symbol_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Symbol(s) => Ok(Expression::String(s)),
        _ => return Err(create_error!(ErrCode::E1004)),
    }
}
fn string_symbol(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Symbol(s)),
        _ => Err(create_error!(ErrCode::E1015)),
    }
}
fn make_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let n = match eval(&exp[1], env)? {
        Expression::Integer(n) => n,
        _ => return Err(create_error!(ErrCode::E1002)),
    };
    if n < 0 {
        return Err(create_error!(ErrCode::E1021));
    }
    let c = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        _ => return Err(create_error!(ErrCode::E1019)),
    };

    let mut s = String::new();
    for _ in 0..n {
        s.push(c);
    }
    Ok(Expression::String(s))
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn format_f() {
        assert_eq!(do_lisp("(format \"~D\" 10)"), "\"10\"");
        assert_eq!(do_lisp("(format \"~d\" 10)"), "\"10\"");
        assert_eq!(do_lisp("(format \"~X\" 10)"), "\"A\"");
        assert_eq!(do_lisp("(format \"~x\" 10)"), "\"a\"");
        assert_eq!(do_lisp("(format \"~O\" 10)"), "\"12\"");
        assert_eq!(do_lisp("(format \"~o\" 10)"), "\"12\"");
        assert_eq!(do_lisp("(format \"~B\" 10)"), "\"1010\"");
        assert_eq!(do_lisp("(format \"~b\" 10)"), "\"1010\"");

        let env = lisp::Environment::new();
        do_lisp_env("(define a \"~D\")", &env);
        do_lisp_env("(define b 100)", &env);
        assert_eq!(do_lisp_env("(format a b)", &env), "\"100\"");
    }
    #[test]
    fn string_eq() {
        assert_eq!(do_lisp("(string=? \"abc\" \"abc\")"), "#t");
        assert_eq!(do_lisp("(string=? \"abc\" \"ABC\")"), "#f");
    }
    #[test]
    fn string_less() {
        assert_eq!(do_lisp("(string<? \"1234\" \"9\")"), "#t");
        assert_eq!(do_lisp("(string<? \"9\" \"1234\")"), "#f");
    }
    #[test]
    fn string_than() {
        assert_eq!(do_lisp("(string>? \"9\" \"1234\")"), "#t");
        assert_eq!(do_lisp("(string>? \"1234\" \"9\")"), "#f");
    }
    #[test]
    fn string_le() {
        assert_eq!(do_lisp("(string<=? \"1234\" \"9\")"), "#t");
        assert_eq!(do_lisp("(string<=? \"1234\" \"1234\")"), "#t");
        assert_eq!(do_lisp("(string<=? \"9\" \"1234\")"), "#f");
    }
    #[test]
    fn string_ge() {
        assert_eq!(do_lisp("(string>=?  \"9\" \"1234\")"), "#t");
        assert_eq!(do_lisp("(string>=?  \"1234\" \"1234\")"), "#t");
        assert_eq!(do_lisp("(string>=?  \"1234\" \"9\")"), "#f");
    }
    #[test]
    fn string_ci_eq() {
        assert_eq!(do_lisp("(string-ci=? \"Abc\" \"aBc\")"), "#t");
        assert_eq!(do_lisp("(string-ci=? \"abc\" \"ABC\")"), "#t");
        assert_eq!(do_lisp("(string-ci=? \"abcd\" \"ABC\")"), "#f");
    }
    #[test]
    fn string_ci_less() {
        assert_eq!(do_lisp("(string-ci<? \"abc\" \"DEF\")"), "#t");
        assert_eq!(do_lisp("(string-ci<? \"DEF\" \"abc\")"), "#f");
    }
    #[test]
    fn string_ci_than() {
        assert_eq!(do_lisp("(string-ci>? \"DEF\" \"abc\")"), "#t");
        assert_eq!(do_lisp("(string-ci>? \"abc\" \"DEF\")"), "#f");
    }
    #[test]
    fn string_ci_le() {
        assert_eq!(do_lisp("(string-ci<=? \"abc\" \"DEF\")"), "#t");
        assert_eq!(do_lisp("(string-ci<=? \"DEF\" \"abc\")"), "#f");
        assert_eq!(do_lisp("(string-ci<=? \"Abc\" \"aBC\")"), "#t");
    }
    #[test]
    fn string_ci_ge() {
        assert_eq!(do_lisp("(string-ci>=? \"abc\" \"DEF\")"), "#f");
        assert_eq!(do_lisp("(string-ci>=? \"DEF\" \"abc\")"), "#t");
        assert_eq!(do_lisp("(string-ci>=? \"Abc\" \"aBC\")"), "#t");
    }
    #[test]
    fn str_append() {
        assert_eq!(do_lisp("(string-append \"ABC\" \"DEF\")"), "\"ABCDEF\"");
        assert_eq!(
            do_lisp("(string-append \"ABC\" \"DEF\" \"123\")"),
            "\"ABCDEF123\""
        );
    }
    #[test]
    fn string_length() {
        assert_eq!(do_lisp("(string-length \"\")"), "0");
        assert_eq!(do_lisp("(string-length \"1234567890\")"), "10");
        assert_eq!(do_lisp("(string-length \"山\")"), "1");
    }
    #[test]
    fn string_size() {
        assert_eq!(do_lisp("(string-size \"\")"), "0");
        assert_eq!(do_lisp("(string-size \"1234567890\")"), "10");
        assert_eq!(do_lisp("(string-size \"山\")"), "3");
    }
    #[test]
    fn number_string() {
        assert_eq!(do_lisp("(number->string 10)"), "\"10\"");
        assert_eq!(do_lisp("(number->string 10.5)"), "\"10.5\"");
        assert_eq!(do_lisp("(number->string 1/3)"), "\"1/3\"");
    }
    #[test]
    fn string_number() {
        assert_eq!(do_lisp("(string->number \"123\")"), "123");
        assert_eq!(do_lisp("(string->number \"10.5\")"), "10.5");
        assert_eq!(do_lisp("(string->number \"1/3\")"), "1/3");
    }
    #[test]
    fn list_string() {
        assert_eq!(do_lisp("(list->string (list))"), "\"\"");
        assert_eq!(do_lisp("(list->string (list #\\a #\\b #\\c))"), "\"abc\"");
    }
    #[test]
    fn string_list() {
        assert_eq!(do_lisp("(string->list \"\")"), "()");
        assert_eq!(do_lisp("(string->list \"abc\")"), "(#\\a #\\b #\\c)");
        assert_eq!(do_lisp("(string->list \"山田\")"), "(#\\山 #\\田)");
    }
    #[test]
    fn substring() {
        assert_eq!(do_lisp("(substring \"1234567890\" 1 2)"), "\"2\"");
        assert_eq!(do_lisp("(substring \"1234567890\" 1 3)"), "\"23\"");
        assert_eq!(do_lisp("(substring \"1234567890\" 0 10)"), "\"1234567890\"");
        assert_eq!(do_lisp("(substring \"山\" 0 1)"), "\"山\"");
        assert_eq!(do_lisp("(substring \"山1\" 0 2)"), "\"山1\"");
    }
    #[test]
    fn symbol_string() {
        assert_eq!(do_lisp("(symbol->string 'abc)"), "\"abc\"");
    }
    #[test]
    fn string_symbol() {
        assert_eq!(do_lisp("(string->symbol \"abc\")"), "abc");
    }
    #[test]
    fn make_string() {
        assert_eq!(do_lisp("(make-string 4 #\\a)"), "\"aaaa\"");
        assert_eq!(do_lisp("(make-string 4 #\\山)"), "\"山山山山\"");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;

    #[test]
    fn format_f() {
        assert_eq!(do_lisp("(format)"), "E1007");
        assert_eq!(do_lisp("(format \"~B\")"), "E1007");
        assert_eq!(do_lisp("(format \"~B\" 10 12)"), "E1007");
        assert_eq!(do_lisp("(format 10 12)"), "E1015");
        assert_eq!(do_lisp("(format \"~A\" #f)"), "E1002");
        assert_eq!(do_lisp("(format \"~A\" 10)"), "E1018");
    }
    #[test]
    fn string_eq() {
        assert_eq!(do_lisp("(string=?)"), "E1007");
        assert_eq!(do_lisp("(string=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_less() {
        assert_eq!(do_lisp("(string<?)"), "E1007");
        assert_eq!(do_lisp("(string<? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string<? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string<? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string<? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string<? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_than() {
        assert_eq!(do_lisp("(string>?)"), "E1007");
        assert_eq!(do_lisp("(string>? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string>? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string>? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string>? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string>? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_le() {
        assert_eq!(do_lisp("(string<=?)"), "E1007");
        assert_eq!(do_lisp("(string<=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string<=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string<=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string<=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string<=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ge() {
        assert_eq!(do_lisp("(string>=?)"), "E1007");
        assert_eq!(do_lisp("(string>=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string>=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string>=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string>=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string>=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ci_eq() {
        assert_eq!(do_lisp("(string-ci=?)"), "E1007");
        assert_eq!(do_lisp("(string-ci=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string-ci=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string-ci=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-ci=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string-ci=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ci_less() {
        assert_eq!(do_lisp("(string-ci<?)"), "E1007");
        assert_eq!(do_lisp("(string-ci<? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string-ci<? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string-ci<? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-ci<? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string-ci<? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ci_than() {
        assert_eq!(do_lisp("(string-ci>?)"), "E1007");
        assert_eq!(do_lisp("(string-ci>? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string-ci>? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string-ci>? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-ci>? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string-ci>? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ci_le() {
        assert_eq!(do_lisp("(string-ci<=?)"), "E1007");
        assert_eq!(do_lisp("(string-ci<=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string-ci<=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string-ci<=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-ci<=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string-ci<=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ci_ge() {
        assert_eq!(do_lisp("(string-ci>=?)"), "E1007");
        assert_eq!(do_lisp("(string-ci>=? \"abc\")"), "E1007");
        assert_eq!(do_lisp("(string-ci>=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_eq!(do_lisp("(string-ci>=? \"abc\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-ci>=? 10 \"abc\")"), "E1015");
        assert_eq!(do_lisp("(string-ci>=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn str_append() {
        assert_eq!(do_lisp("(string-append)"), "E1007");
        assert_eq!(do_lisp("(string-append \"a\")"), "E1007");
        assert_eq!(do_lisp("(string-append \"a\" 10)"), "E1015");
        assert_eq!(do_lisp("(string-append \"a\" a)"), "E1008");
    }
    #[test]
    fn string_length() {
        assert_eq!(do_lisp("(string-length)"), "E1007");
        assert_eq!(do_lisp("(string-length \"1234\" \"12345\")"), "E1007");
        assert_eq!(do_lisp("(string-length 1000)"), "E1015");
    }
    #[test]
    fn string_size() {
        assert_eq!(do_lisp("(string-size)"), "E1007");
        assert_eq!(do_lisp("(string-size \"1234\" \"12345\")"), "E1007");
        assert_eq!(do_lisp("(string-size 1000)"), "E1015");
    }
    #[test]
    fn number_string() {
        assert_eq!(do_lisp("(number->string)"), "E1007");
        assert_eq!(do_lisp("(number->string 10 20)"), "E1007");
        assert_eq!(do_lisp("(number->string #f)"), "E1003");
        assert_eq!(do_lisp("(number->string a)"), "E1008");
    }
    #[test]
    fn string_number() {
        assert_eq!(do_lisp("(string->number)"), "E1007");
        assert_eq!(do_lisp("(string->number \"123\" \"10.5\")"), "E1007");
        assert_eq!(do_lisp("(string->number 100)"), "E1015");
        assert_eq!(do_lisp("(string->number \"/1\")"), "E1003");
        assert_eq!(do_lisp("(string->number \"1/3/2\")"), "E1003");
        assert_eq!(do_lisp("(string->number \"1/0\")"), "E1013");
        assert_eq!(do_lisp("(string->number a)"), "E1008");
    }
    #[test]
    fn list_string() {
        assert_eq!(do_lisp("(list->string)"), "E1007");
        assert_eq!(
            do_lisp("(list->string (list #\\a #\\b)(list #\\a #\\b))"),
            "E1007"
        );
        assert_eq!(do_lisp("(list->string 10)"), "E1005");
        assert_eq!(do_lisp("(list->string (list #\\a 10))"), "E1019");
        assert_eq!(do_lisp("(list->string a)"), "E1008");
    }
    #[test]
    fn substring() {
        assert_eq!(do_lisp("(substring)"), "E1007");
        assert_eq!(do_lisp("(substring \"1234567890\" 1)"), "E1007");
        assert_eq!(do_lisp("(substring \"1234567890\" 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(substring  1 2 3)"), "E1015");
        assert_eq!(do_lisp("(substring \"1234567890\" #t 2)"), "E1002");
        assert_eq!(do_lisp("(substring \"1234567890\" 0 #t)"), "E1002");
        assert_eq!(do_lisp("(substring \"1234567890\" a 2)"), "E1008");
        assert_eq!(do_lisp("(substring \"1234567890\" 0 a)"), "E1008");

        assert_eq!(do_lisp("(substring \"1234567890\" -1 2)"), "E1021");
        assert_eq!(do_lisp("(substring \"1234567890\" 0 -2)"), "E1021");
        assert_eq!(do_lisp("(substring \"1234567890\" 0 11)"), "E1021");
        assert_eq!(do_lisp("(substring \"1234567890\" 6 5)"), "E1021");

        assert_eq!(do_lisp("(substring \"山\" 0 2)"), "E1021");
    }
    #[test]
    fn symbol_string() {
        assert_eq!(do_lisp("(symbol->string)"), "E1007");
        assert_eq!(do_lisp("(symbol->string 'a 'b)"), "E1007");
        assert_eq!(do_lisp("(symbol->string #t)"), "E1004");
    }
    #[test]
    fn string_symbol() {
        assert_eq!(do_lisp("(string->symbol)"), "E1007");
        assert_eq!(do_lisp("(string->symbol \"abc\"  \"def\")"), "E1007");
        assert_eq!(do_lisp("(string->symbol #t)"), "E1015");
    }

    #[test]
    fn string_list() {
        assert_eq!(do_lisp("(string->list)"), "E1007");
        assert_eq!(do_lisp("(string->list \"a\" \"b\")"), "E1007");
        assert_eq!(do_lisp("(string->list #\\a)"), "E1015");
        assert_eq!(do_lisp("(string->list a)"), "E1008");
    }
    #[test]
    fn make_string() {
        assert_eq!(do_lisp("(make-string)"), "E1007");
        assert_eq!(do_lisp("(make-string a)"), "E1007");
        assert_eq!(do_lisp("(make-string a a a)"), "E1007");

        assert_eq!(do_lisp("(make-string #t #\\a)"), "E1002");
        assert_eq!(do_lisp("(make-string -1 #\\a)"), "E1021");
        assert_eq!(do_lisp("(make-string 4 a)"), "E1008");
        assert_eq!(do_lisp("(make-string 4 #t)"), "E1019");
    }
}
