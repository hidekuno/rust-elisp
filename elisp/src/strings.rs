/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::create_error;
use crate::create_error_value;
use crate::reference_obj;

use std::vec::Vec;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, Int, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::number::Number;
use crate::number::Rat;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("format", format_f);

    b.regist("string", string);
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
    b.regist("number->string", |exp, env| {
        do_radix(exp, env, number_string)
    });
    b.regist("string->number", |exp, env| {
        do_radix(exp, env, string_number)
    });
    b.regist("list->string", list_string);
    b.regist("string->list", string_list);

    b.regist("vector->string", vector_string);
    b.regist("string->vector", string_vector);

    b.regist("substring", substring);
    b.regist("symbol->string", symbol_string);
    b.regist("string->symbol", string_symbol);
    b.regist("make-string", make_string);

    b.regist("string-split", string_split);
    b.regist("string-join", string_join);

    b.regist("string-scan", |exp, env| {
        string_scan(exp, env, StringScan::Left)
    });
    b.regist("string-scan-right", |exp, env| {
        string_scan(exp, env, StringScan::Right)
    });

    b.regist("string-reverse", string_reverse);
    b.regist("string-upcase", |exp, env| {
        string_case(exp, env, |s| s.to_uppercase())
    });
    b.regist("string-downcase", |exp, env| {
        string_case(exp, env, |s| s.to_lowercase())
    });
    b.regist("string-index", |exp, env| {
        string_index(exp, env, |s, c| s.find(c))
    });
    b.regist("string-index-right", |exp, env| {
        string_index(exp, env, |s, c| s.rfind(c))
    });
    b.regist("string-delete", string_delete);
    b.regist("string-trim", |exp, env| {
        string_trim(
            exp,
            env,
            |s| s.trim_start(),
            |s, pred| s.trim_start_matches(pred),
        )
    });
    b.regist("string-trim-right", |exp, env| {
        string_trim(
            exp,
            env,
            |s| s.trim_end(),
            |s, pred| s.trim_end_matches(pred),
        )
    });
    b.regist("string-trim-both", |exp, env| {
        string_trim(
            exp,
            env,
            |s| s.trim_start().trim_end(),
            |s, pred| s.trim_start_matches(pred).trim_end_matches(pred),
        )
    });
    b.regist("string-take", |exp, env| {
        string_range(exp, env, |s, v| s.chars().take(v).collect::<String>())
    });
    b.regist("string-take-right", |exp, env| {
        string_range(exp, env, |s, v| {
            s.chars().skip(s.chars().count() - v).collect::<String>()
        })
    });
    b.regist("string-drop", |exp, env| {
        string_range(exp, env, |s, v| s.chars().skip(v).collect::<String>())
    });
    b.regist("string-drop-right", |exp, env| {
        string_range(exp, env, |s, v| {
            s.chars().take(s.chars().count() - v).collect::<String>()
        })
    });

    // The below function is deprecated.
    // (string-take-u8 "1山2" 2)
    // thread 'main' panicked at 'byte index 2 is not a char boundary; it is inside '山' (bytes 1..4) of `1山2`',
    // library/core/src/str/mod.rs:127:5
    // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    b.regist("string-take-u8", |exp, env| {
        string_range_u8(exp, env, |s, v| &s[..v])
    });
    b.regist("string-take-right-u8", |exp, env| {
        string_range_u8(exp, env, |s, v| &s[(s.len() - v)..])
    });
    b.regist("string-drop-u8", |exp, env| {
        string_range_u8(exp, env, |s, v| &s[v..])
    });
    b.regist("string-drop-right-u8", |exp, env| {
        string_range_u8(exp, env, |s, v| &s[..(s.len() - v)])
    });
}
// i64::from_str_radix() is exists, but there is NO to_str_radix.
pub fn to_str_radix(n: Int, r: u32) -> Option<String> {
    let mut num = n;
    let mut s = String::new();

    let tbl: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    if !(2..=36).contains(&r) {
        return None;
    }
    loop {
        let n = num % r as Int;
        s.push(tbl[n as usize]);
        num /= r as Int;
        if 0 == num {
            break;
        }
    }
    Some(s.chars().rev().collect::<String>())
}
pub fn do_radix(
    exp: &[Expression],
    env: &Environment,
    func: fn(exp: &Expression, env: &Environment, r: u32) -> ResultExpression,
) -> ResultExpression {
    if 2 > exp.len() || 3 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let r = if exp.len() == 3 {
        match eval(&exp[2], env)? {
            Expression::Integer(i) => i,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        }
    } else {
        10
    };
    // radix must be between 2 and 36 about scheme
    // rust, 0 and 36
    if !(2..=36).contains(&r) {
        Err(create_error!(ErrCode::E1021))
    } else {
        func(&exp[1], env, r as u32)
    }
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
    Ok(Environment::create_string(s))
}
fn string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let c = match eval(&exp[1], env)? {
        Expression::Char(c) => c,
        e => return Err(create_error_value!(ErrCode::E1019, e)),
    };
    Ok(Environment::create_string(c.to_string()))
}
fn strcmp(
    exp: &[Expression],
    env: &Environment,
    func: fn(x: &String, y: &String) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v = Vec::new();
    for e in &exp[1..] {
        let s = match eval(e, env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        v.push(s.to_string());
    }
    Ok(Expression::Boolean(func(&v[0], &v[1])))
}
fn str_append(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 > exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut v = String::new();
    for e in &exp[1..] {
        match eval(e, env)? {
            Expression::String(s) => {
                let s = s.to_string();
                v.push_str(&s.into_boxed_str());
            }
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
    }
    Ok(Environment::create_string(v))
}
fn str_length(
    exp: &[Expression],
    env: &Environment,
    func: fn(s: String) -> usize,
) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Integer(func(s.to_string()) as Int)),
        e => Err(create_error_value!(ErrCode::E1015, e)),
    }
}
fn number_string(exp: &Expression, env: &Environment, r: u32) -> ResultExpression {
    let v = Expression::to_number(&eval(exp, env)?)?;
    match v {
        Number::Integer(n) => {
            if r == 10 {
                Ok(Environment::create_string(v.to_string()))
            } else if let Some(s) = to_str_radix(n, r) {
                Ok(Environment::create_string(s))
            } else {
                Err(create_error!(ErrCode::E1021))
            }
        }
        _ => Ok(Environment::create_string(v.to_string())),
    }
}
fn string_number(exp: &Expression, env: &Environment, r: u32) -> ResultExpression {
    let s = match eval(exp, env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    if let Ok(n) = Int::from_str_radix(&s, r) {
        return Ok(Expression::Integer(n));
    }

    let v = if let Ok(n) = s.parse::<f64>() {
        Expression::Float(n)
    } else {
        match Rat::from_radix(&s, r) {
            Ok(n) => Expression::Rational(n),
            Err(_) => Expression::Boolean(false),
        }
    };
    Ok(v)
}
fn list_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_string(exp, env, ErrCode::E1005)
}
fn vector_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    seq_string(exp, env, ErrCode::E1022)
}
fn seq_string(exp: &[Expression], env: &Environment, err: ErrCode) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        Expression::Vector(l) => l,
        e => return Err(create_error_value!(err, e)),
    };

    let l = &*(reference_obj!(l));
    let mut v = String::new();

    for e in l.iter() {
        v.push(match eval(e, env)? {
            Expression::Char(c) => c,
            e => return Err(create_error_value!(ErrCode::E1019, e)),
        });
    }
    Ok(Environment::create_string(v))
}
fn string_list(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = string_seq(exp, env)?;
    Ok(Environment::create_list(l))
}
fn string_vector(exp: &[Expression], env: &Environment) -> ResultExpression {
    let l = string_seq(exp, env)?;
    Ok(Environment::create_vector(l))
}
fn string_seq(exp: &[Expression], env: &Environment) -> Result<Vec<Expression>, Error> {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let mut l: Vec<Expression> = Vec::new();
    for c in s.as_str().chars() {
        l.push(Expression::Char(c));
    }
    Ok(l)
}
fn substring(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 4 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let v = inner_substring(&exp[2..], env, s.chars().collect::<String>())?;

    Ok(Environment::create_string(v))
}
fn symbol_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::Symbol(s) => s,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    Ok(Environment::create_string(s))
}
fn string_symbol(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 2 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::String(s) => Ok(Expression::Symbol(s.to_string())),
        e => Err(create_error_value!(ErrCode::E1015, e)),
    }
}
fn make_string(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let n = match eval(&exp[1], env)? {
        Expression::Integer(n) => n,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if n < 0 {
        return Err(create_error!(ErrCode::E1021));
    }
    let c = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        e => return Err(create_error_value!(ErrCode::E1019, e)),
    };

    let mut s = String::new();
    for _ in 0..n {
        s.push(c);
    }
    Ok(Environment::create_string(s))
}
fn string_split(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let c = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        e => return Err(create_error_value!(ErrCode::E1019, e)),
    };
    let v = s
        .split(c)
        .map(|s| Environment::create_string(String::from(s)))
        .collect::<Vec<_>>();

    Ok(Environment::create_list(v))
}
fn string_join(exp: &[Expression], env: &Environment) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l = &*(reference_obj!(l));

    let s = match eval(&exp[2], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };

    let mut v: Vec<String> = Vec::new();
    for e in l {
        let s = match e {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        v.push(s.to_string());
    }
    Ok(Environment::create_string(v.join(&s)))
}
enum StringScan {
    Left,
    Right,
}
fn string_scan(exp: &[Expression], env: &Environment, direct: StringScan) -> ResultExpression {
    fn resolv_scan(x: Option<usize>) -> Expression {
        match x {
            Some(i) => Expression::Integer(i as Int),
            None => Expression::Boolean(false),
        }
    }
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let p = match eval(&exp[1], env)? {
        Expression::String(p) => p,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    match eval(&exp[2], env)? {
        Expression::Char(c) => Ok(match direct {
            StringScan::Left => resolv_scan(p.find(c)),
            StringScan::Right => resolv_scan(p.rfind(c)),
        }),
        Expression::String(s) => Ok(match direct {
            StringScan::Left => resolv_scan(p.find(s.as_ref())),
            StringScan::Right => resolv_scan(p.rfind(s.as_ref())),
        }),
        e => Err(create_error_value!(ErrCode::E1009, e)),
    }
}
fn string_reverse(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 || 4 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let s = inner_substring(&exp[2..], env, s.chars().collect::<String>())?;
    Ok(Environment::create_string(
        s.chars().rev().collect::<String>(),
    ))
}
fn string_case(
    exp: &[Expression],
    env: &Environment,
    case: fn(&String) -> String,
) -> ResultExpression {
    if exp.len() < 2 || 4 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let s = inner_substring(&exp[2..], env, s.chars().collect::<String>())?;
    Ok(Environment::create_string(case(&s)))
}
fn string_index(
    exp: &[Expression],
    env: &Environment,
    find: fn(&String, char) -> Option<usize>,
) -> ResultExpression {
    if exp.len() < 2 || 5 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let pred = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        e => return Err(create_error_value!(ErrCode::E1019, e)),
    };

    let (start, end) = get_start_end(&exp[3..], env, &s)?;

    Ok(match find(&s, pred) {
        Some(i) => {
            if (start <= i) && (i < end) {
                Expression::Integer(i as Int)
            } else {
                Expression::Boolean(false)
            }
        }
        None => Expression::Boolean(false),
    })
}
fn string_delete(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 || 5 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let pred = match eval(&exp[2], env)? {
        Expression::Char(c) => c,
        e => return Err(create_error_value!(ErrCode::E1019, e)),
    };
    let s = inner_substring(&exp[3..], env, s.chars().collect::<String>())?;
    Ok(Environment::create_string(
        s.chars().filter(|c| *c != pred).collect::<String>(),
    ))
}
fn string_trim(
    exp: &[Expression],
    env: &Environment,
    trim: for<'a> fn(&'a String) -> &'a str,
    trim_match: for<'a> fn(&'a String, char) -> &'a str,
) -> ResultExpression {
    if exp.len() < 2 || 3 < exp.len() {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    if exp.len() == 2 {
        Ok(Environment::create_string(trim(&s).to_string()))
    } else {
        let pred = match eval(&exp[2], env)? {
            Expression::Char(c) => c,
            e => return Err(create_error_value!(ErrCode::E1019, e)),
        };
        Ok(Environment::create_string(trim_match(&s, pred).to_string()))
    }
}
fn string_range(
    exp: &[Expression],
    env: &Environment,
    range: fn(&String, usize) -> String,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let v = match eval(&exp[2], env)? {
        Expression::Integer(v) => v,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if 0 > v || s.chars().count() < v as usize {
        return Err(create_error!(ErrCode::E1021));
    }
    Ok(Environment::create_string(range(&s, v as usize)))
}
fn string_range_u8(
    exp: &[Expression],
    env: &Environment,
    range: for<'a> fn(&'a String, usize) -> &'a str,
) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let s = match eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let v = match eval(&exp[2], env)? {
        Expression::Integer(v) => v,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };
    if 0 > v || s.len() < v as usize {
        return Err(create_error!(ErrCode::E1021));
    }
    Ok(Environment::create_string(
        range(&s, v as usize).to_string(),
    ))
}
fn inner_substring(exp: &[Expression], env: &Environment, s: String) -> Result<String, Error> {
    let (start, end) = get_start_end(exp, env, &s)?;

    // the trait `std::convert::From<str>` is not implemented for `std::string::String`
    // s.as_str()[start..end].to_string()),
    //     => panicked at 'byte index 1 is not a char boundary; it is inside '山' (bytes 0..3)
    let mut v = String::new();
    for c in &s.chars().collect::<Vec<char>>()[start..end] {
        v.push(*c);
    }
    Ok(v)
}
fn get_start_end(exp: &[Expression], env: &Environment, s: &str) -> Result<(usize, usize), Error> {
    let mut param: [usize; 2] = [0, s.chars().count()];

    for (i, e) in exp.iter().enumerate() {
        let v = match eval(e, env)? {
            Expression::Integer(v) => v,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
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
    Ok((start, end))
}
#[test]
fn test_to_str_radix() {
    assert_eq!(to_str_radix(32, 37), None);
}
#[test]
fn test_number_string() {
    use crate::lisp::Environment;

    let env = Environment::new();
    let _ = number_string(&Expression::Integer(10), &env, 37)
        .map_err(|e| assert_eq!(e.get_code(), "E1021"));
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
    fn string() {
        assert_eq!(do_lisp("(string #\\a)"), "\"a\"");
        assert_eq!(do_lisp("(string #\\A)"), "\"A\"");
        assert_eq!(do_lisp("(string #\\0)"), "\"0\"");
        assert_eq!(do_lisp("(string #\\9)"), "\"9\"");
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
        assert_eq!(
            do_lisp("(number->string 3735927486 2)"),
            "\"11011110101011011011101010111110\""
        );
        assert_eq!(
            do_lisp("(number->string 3735927486 3)"),
            "\"100122100210210001200\""
        );
        assert_eq!(
            do_lisp("(number->string 3735927486 4)"),
            "\"3132223123222332\""
        );
        assert_eq!(
            do_lisp("(number->string 3735927486 5)"),
            "\"30122344134421\""
        );
        assert_eq!(
            do_lisp("(number->string 3735927486 6)"),
            "\"1414413520330\""
        );
        assert_eq!(do_lisp("(number->string 3735927486 7)"), "\"161402600604\"");
        assert_eq!(do_lisp("(number->string 3735927486 8)"), "\"33653335276\"");
        assert_eq!(do_lisp("(number->string 3735927486 9)"), "\"10570723050\"");
        assert_eq!(do_lisp("(number->string 3735927486 10)"), "\"3735927486\"");
        assert_eq!(do_lisp("(number->string 3735927486 11)"), "\"1647919685\"");
        assert_eq!(do_lisp("(number->string 3735927486 12)"), "\"8831a30a6\"");
        assert_eq!(do_lisp("(number->string 3735927486 13)"), "\"476cc28a5\"");
        assert_eq!(do_lisp("(number->string 3735927486 14)"), "\"276253874\"");
        assert_eq!(do_lisp("(number->string 3735927486 15)"), "\"16ceb1726\"");
        assert_eq!(do_lisp("(number->string 3735927486 16)"), "\"deadbabe\"");
        assert_eq!(do_lisp("(number->string 3735927486 17)"), "\"91d36cc6\"");
        assert_eq!(do_lisp("(number->string 3735927486 18)"), "\"61f27270\"");
        assert_eq!(do_lisp("(number->string 3735927486 19)"), "\"437f24b8\"");
        assert_eq!(do_lisp("(number->string 3735927486 20)"), "\"2i79aie6\"");
        assert_eq!(do_lisp("(number->string 3735927486 21)"), "\"21bff6ii\"");
        assert_eq!(do_lisp("(number->string 3735927486 22)"), "\"1akk149g\"");
        assert_eq!(do_lisp("(number->string 3735927486 23)"), "\"125a42hj\"");
        assert_eq!(do_lisp("(number->string 3735927486 24)"), "\"jd49956\"");
        assert_eq!(do_lisp("(number->string 3735927486 25)"), "\"f7do8ob\"");
        assert_eq!(do_lisp("(number->string 3735927486 26)"), "\"c2b8boi\"");
        assert_eq!(do_lisp("(number->string 3735927486 27)"), "\"9h9ll1i\"");
        assert_eq!(do_lisp("(number->string 3735927486 28)"), "\"7l225hi\"");
        assert_eq!(do_lisp("(number->string 3735927486 29)"), "\"6842o9l\"");
        assert_eq!(do_lisp("(number->string 3735927486 30)"), "\"53m7kg6\"");
        assert_eq!(do_lisp("(number->string 3735927486 31)"), "\"46f9hir\"");
        assert_eq!(do_lisp("(number->string 3735927486 32)"), "\"3farelu\"");
        assert_eq!(do_lisp("(number->string 3735927486 33)"), "\"2tf7mor\"");
        assert_eq!(do_lisp("(number->string 3735927486 34)"), "\"2e7m366\"");
        assert_eq!(do_lisp("(number->string 3735927486 35)"), "\"214kbpb\"");
        assert_eq!(do_lisp("(number->string 3735927486 36)"), "\"1ps9w3i\"");
    }
    #[test]
    fn string_number() {
        assert_eq!(do_lisp("(string->number \"123\")"), "123");
        assert_eq!(do_lisp("(string->number \"10.5\")"), "10.5");
        assert_eq!(do_lisp("(string->number \"1/3\")"), "1/3");
        assert_eq!(do_lisp("(string->number \"10000\" 2)"), "16");
        assert_eq!(do_lisp("(string->number \"012\" 8)"), "10");
        assert_eq!(do_lisp("(string->number \"123\" 10)"), "123");
        assert_eq!(do_lisp("(string->number \"ab\" 16)"), "171");
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
    fn vector_string() {
        assert_eq!(do_lisp("(vector->string (vector))"), "\"\"");
        assert_eq!(
            do_lisp("(vector->string (vector #\\a #\\b #\\c))"),
            "\"abc\""
        );
    }
    #[test]
    fn string_vector() {
        assert_eq!(do_lisp("(string->vector \"\")"), "#()");
        assert_eq!(do_lisp("(string->vector \"abc\")"), "#(#\\a #\\b #\\c)");
        assert_eq!(do_lisp("(string->vector \"山田\")"), "#(#\\山 #\\田)");
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
    #[test]
    fn string_split() {
        assert_eq!(
            do_lisp("(string-split  \"abc:def:g\"  #\\:)"),
            "(\"abc\" \"def\" \"g\")"
        );
        assert_eq!(do_lisp("(string-split  \"abcdef\"  #\\,)"), "(\"abcdef\")");
        assert_eq!(
            do_lisp("(string-split  \",abcdef\"  #\\,)"),
            "(\"\" \"abcdef\")"
        );
        assert_eq!(
            do_lisp("(string-split  \"abcdef,\"  #\\,)"),
            "(\"abcdef\" \"\")"
        );
        assert_eq!(do_lisp("(string-split  \"\"  #\\,)"), "(\"\")");
    }
    #[test]
    fn string_join() {
        assert_eq!(
            do_lisp("(string-join '(\"a\" \"b\" \"c\" \"d\" \"e\") \":\")"),
            "\"a:b:c:d:e\""
        );
        assert_eq!(
            do_lisp("(string-join '(\"a\" \"b\" \"c\" \"d\" \"e\") \"::\")"),
            "\"a::b::c::d::e\""
        );
        assert_eq!(do_lisp("(string-join '(\"a\") \"::\")"), "\"a\"");
        assert_eq!(do_lisp("(string-join '(\"\") \"::\")"), "\"\"");
    }
    #[test]
    fn string_scan() {
        assert_eq!(do_lisp("(string-scan \"abracadabra\" \"ada\")"), "5");
        assert_eq!(do_lisp("(string-scan \"abracadabra\" #\\c)"), "4");
        assert_eq!(do_lisp("(string-scan \"abracadabra\" \"aba\")"), "#f");
        assert_eq!(do_lisp("(string-scan \"abracadabra\" #\\z)"), "#f");
        assert_eq!(do_lisp("(string-scan \"1122\" #\\2)"), "2");
    }
    #[test]
    fn string_scan_right() {
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\" \"ada\")"), "5");
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\" #\\c)"), "4");
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\" \"aba\")"), "#f");
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\" #\\z)"), "#f");
        assert_eq!(do_lisp("(string-scan-right \"1122\" #\\2)"), "3");
    }
    #[test]
    fn string_reverse() {
        assert_eq!(do_lisp("(string-reverse \"1234567890\")"), "\"0987654321\"");
        assert_eq!(do_lisp("(string-reverse \"1234567890\" 3)"), "\"0987654\"");
        assert_eq!(do_lisp("(string-reverse \"1234567890\" 3 8)"), "\"87654\"");
        assert_eq!(do_lisp("(string-reverse  \"山川\")"), "\"川山\"");
    }
    #[test]
    fn string_upcase() {
        assert_eq!(do_lisp("(string-upcase \"ab1012cd\")"), "\"AB1012CD\"");
        assert_eq!(do_lisp("(string-upcase \"abcd\" 1)"), "\"BCD\"");
        assert_eq!(do_lisp("(string-upcase \"abcd\" 1 2)"), "\"B\"");
    }
    #[test]
    fn string_downcase() {
        assert_eq!(do_lisp("(string-downcase \"AB1012CD\")"), "\"ab1012cd\"");
        assert_eq!(do_lisp("(string-downcase \"ABCD\" 1)"), "\"bcd\"");
        assert_eq!(do_lisp("(string-downcase \"ABCD\" 1 2)"), "\"b\"");
    }
    #[test]
    fn string_index() {
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\a)"), "0");
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\e)"), "4");
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\z)"), "#f");
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\c 2)"), "2");
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\d 2 8)"), "3");
        assert_eq!(do_lisp("(string-index \"abcdefghijlklmn\" #\\k 2 8)"), "#f");
        assert_eq!(
            do_lisp("(string-index \"abcdefghijlklcn\" #\\n 0 14)"),
            "#f"
        );
        assert_eq!(
            do_lisp("(string-index \"abcdefghijlklcn\" #\\n 0 15)"),
            "14"
        );
    }
    #[test]
    fn string_index_right() {
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijlklmn\" #\\a)"),
            "0"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijlklmn\" #\\z)"),
            "#f"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijlklcn\" #\\c 2)"),
            "13"
        );

        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijlklcn\" #\\n 2 14)"),
            "#f"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijlklcn\" #\\n 2 15)"),
            "14"
        );
    }
    #[test]
    fn string_delete() {
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\a)"),
            "\"bcdefghijlklcn\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\a 3)"),
            "\"defghijlklcn\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\n 13)"),
            "\"c\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\n 14)"),
            "\"\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\a 3 4)"),
            "\"d\""
        );
        assert_eq!(
            do_lisp("(string-delete \"aaaaaaaaaaaaaaaaaaaa\" #\\a 3 4)"),
            "\"\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\a 3 9)"),
            "\"defghi\""
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijlklcn\" #\\h 3 9)"),
            "\"defgi\""
        );
    }
    #[test]
    fn string_trim() {
        assert_eq!(do_lisp("(string-trim  \"  ad  \")"), "\"ad  \"");
        assert_eq!(do_lisp("(string-trim \"ada\" #\\a)"), "\"da\"");
    }
    #[test]
    fn string_trim_right() {
        assert_eq!(do_lisp("(string-trim-right  \"  ad  \")"), "\"  ad\"");
        assert_eq!(do_lisp("(string-trim-right \"ada\" #\\a)"), "\"ad\"");
    }
    #[test]
    fn string_trim_both() {
        assert_eq!(do_lisp("(string-trim-both  \"  ad  \")"), "\"ad\"");
        assert_eq!(do_lisp("(string-trim-both \"ada\" #\\a)"), "\"d\"");
    }
    #[test]
    fn string_take() {
        assert_eq!(do_lisp("(string-take \"1234567890\" 0)"), "\"\"");
        assert_eq!(do_lisp("(string-take \"1山2\" 2)"), "\"1山\"");
        assert_eq!(do_lisp("(string-take \"1234567890\" 10)"), "\"1234567890\"");
    }
    #[test]
    fn string_take_right() {
        assert_eq!(do_lisp("(string-take-right \"1234567890\" 0)"), "\"\"");
        assert_eq!(do_lisp("(string-take-right \"1山2\" 2)"), "\"山2\"");
        assert_eq!(
            do_lisp("(string-take-right \"1234567890\" 10)"),
            "\"1234567890\""
        );
    }
    #[test]
    fn string_drop() {
        assert_eq!(do_lisp("(string-drop \"1234567890\" 0)"), "\"1234567890\"");
        assert_eq!(do_lisp("(string-drop \"1山2\" 1)"), "\"山2\"");
        assert_eq!(do_lisp("(string-drop \"1234567890\" 10)"), "\"\"");
    }
    #[test]
    fn string_drop_right() {
        assert_eq!(
            do_lisp("(string-drop-right \"1234567890\" 0)"),
            "\"1234567890\""
        );
        assert_eq!(do_lisp("(string-drop-right \"1山2\" 1)"), "\"1山\"");
        assert_eq!(do_lisp("(string-drop-right \"1234567890\" 10)"), "\"\"");
    }
    #[test]
    fn string_take_u8() {
        assert_eq!(do_lisp("(string-take-u8 \"1234567890\" 0)"), "\"\"");
        assert_eq!(do_lisp("(string-take-u8 \"1234567890\" 3)"), "\"123\"");
        assert_eq!(
            do_lisp("(string-take-u8 \"1234567890\" 10)"),
            "\"1234567890\""
        );
    }
    #[test]
    fn string_take_right_u8() {
        assert_eq!(do_lisp("(string-take-right-u8 \"1234567890\" 0)"), "\"\"");
        assert_eq!(
            do_lisp("(string-take-right-u8 \"1234567890\" 3)"),
            "\"890\""
        );
        assert_eq!(
            do_lisp("(string-take-right-u8 \"1234567890\" 10)"),
            "\"1234567890\""
        );
    }
    #[test]
    fn string_drop_u8() {
        assert_eq!(
            do_lisp("(string-drop-u8 \"1234567890\" 0)"),
            "\"1234567890\""
        );
        assert_eq!(do_lisp("(string-drop-u8 \"1234567890\" 3)"), "\"4567890\"");
        assert_eq!(do_lisp("(string-drop-u8 \"1234567890\" 10)"), "\"\"");
    }
    #[test]
    fn string_drop_right_u8() {
        assert_eq!(
            do_lisp("(string-drop-right-u8 \"1234567890\" 0)"),
            "\"1234567890\""
        );
        assert_eq!(
            do_lisp("(string-drop-right-u8 \"1234567890\" 3)"),
            "\"1234567\""
        );
        assert_eq!(do_lisp("(string-drop-right-u8 \"1234567890\" 10)"), "\"\"");
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
    fn string() {
        assert_eq!(do_lisp("(string)"), "E1007");
        assert_eq!(do_lisp("(string 1 2)"), "E1007");
        assert_eq!(do_lisp("(string 10)"), "E1019");
        assert_eq!(do_lisp("(string a)"), "E1008");
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
        assert_eq!(do_lisp("(number->string 10 20 10)"), "E1007");
        assert_eq!(do_lisp("(number->string #f)"), "E1003");
        assert_eq!(do_lisp("(number->string #f 10)"), "E1003");
        assert_eq!(do_lisp("(number->string 100 1)"), "E1021");
        assert_eq!(do_lisp("(number->string 100 37)"), "E1021");
        assert_eq!(do_lisp("(number->string a)"), "E1008");
        assert_eq!(do_lisp("(number->string 10 a)"), "E1008");
    }
    #[test]
    fn string_number() {
        assert_eq!(do_lisp("(string->number)"), "E1007");
        assert_eq!(do_lisp("(string->number \"123\" \"10.5\" 10)"), "E1007");
        assert_eq!(do_lisp("(string->number 100)"), "E1015");
        assert_eq!(do_lisp("(string->number 100 10)"), "E1015");
        assert_eq!(do_lisp("(string->number 100 #f)"), "E1002");
        assert_eq!(do_lisp("(string->number 100 1)"), "E1021");
        assert_eq!(do_lisp("(string->number 100 37)"), "E1021");
        assert_eq!(do_lisp("(string->number a)"), "E1008");
        assert_eq!(do_lisp("(string->number 10 a)"), "E1008");

        assert_eq!(do_lisp("(string->number \"ab\" 2)"), "#f");
        assert_eq!(do_lisp("(string->number \"ab\" 8)"), "#f");
        assert_eq!(do_lisp("(string->number \"ab\" 10)"), "#f");
        assert_eq!(do_lisp("(string->number \"/1\")"), "#f");
        assert_eq!(do_lisp("(string->number \"1/3/2\")"), "#f");
        assert_eq!(do_lisp("(string->number \"1/0\")"), "#f");
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
    fn vector_string() {
        assert_eq!(do_lisp("(vector->string)"), "E1007");
        assert_eq!(
            do_lisp("(vector->string (list #\\a #\\b)(list #\\a #\\b))"),
            "E1007"
        );
        assert_eq!(do_lisp("(vector->string 10)"), "E1022");
        assert_eq!(do_lisp("(vector->string (vector #\\a 10))"), "E1019");
        assert_eq!(do_lisp("(vector->string a)"), "E1008");
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
    fn string_vector() {
        assert_eq!(do_lisp("(string->vector)"), "E1007");
        assert_eq!(do_lisp("(string->vector \"a\" \"b\")"), "E1007");
        assert_eq!(do_lisp("(string->vector #\\a)"), "E1015");
        assert_eq!(do_lisp("(string->vector a)"), "E1008");
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
    #[test]
    fn string_split() {
        assert_eq!(do_lisp("(string-split)"), "E1007");
        assert_eq!(do_lisp("(string-split 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(string-split #\\a #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-split \"\" \"\")"), "E1019");
        assert_eq!(do_lisp("(string-split a #\\a)"), "E1008");
        assert_eq!(do_lisp("(string-split \"\" a)"), "E1008");
    }
    #[test]
    fn string_join() {
        assert_eq!(do_lisp("(string-join)"), "E1007");
        assert_eq!(do_lisp("(string-join 1 2 3)"), "E1007");
        assert_eq!(do_lisp("(string-join  #\\a (list \"\" \"\"))"), "E1005");
        assert_eq!(
            do_lisp("(string-join (list 1 \"a\" \"b\")  \",\")"),
            "E1015"
        );
        assert_eq!(do_lisp("(string-join (list \"a\" \"b\" 1) \",\")"), "E1015");
        assert_eq!(do_lisp("(string-join a #\\a)"), "E1008");
        assert_eq!(
            do_lisp("(string-join (list \"a\" \"b\"  \"c\") a)"),
            "E1008"
        );
        assert_eq!(do_lisp("(string-join (list \"a\" \"b\") 10)"), "E1015");
    }
    #[test]
    fn string_scan() {
        assert_eq!(do_lisp("(string-scan)"), "E1007");
        assert_eq!(do_lisp("(string-scan \"abracadabra\")"), "E1007");
        assert_eq!(
            do_lisp("(string-scan \"abracadabra\" \"aba\" \"aba\")"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-scan 10  #\\z)"), "E1015");
        assert_eq!(do_lisp("(string-scan \"abracadabra\" 10)"), "E1009");
        assert_eq!(do_lisp("(string-scan a #\\2)"), "E1008");
        assert_eq!(do_lisp("(string-scan \"1122\" a)"), "E1008");
    }
    #[test]
    fn string_scan_right() {
        assert_eq!(do_lisp("(string-scan-right)"), "E1007");
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\")"), "E1007");
        assert_eq!(
            do_lisp("(string-scan-right \"abracadabra\" \"aba\" \"aba\")"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-scan-right 10  #\\z)"), "E1015");
        assert_eq!(do_lisp("(string-scan-right \"abracadabra\" 10)"), "E1009");
        assert_eq!(do_lisp("(string-scan-right a #\\2)"), "E1008");
        assert_eq!(do_lisp("(string-scan-right \"1122\" a)"), "E1008");
    }
    #[test]
    fn string_reverse() {
        assert_eq!(do_lisp("(string-reverse)"), "E1007");
        assert_eq!(
            do_lisp("(string-reverse \"abcdefghijklmn\" 2 3 4)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-reverse 10)"), "E1015");
        assert_eq!(
            do_lisp("(string-reverse \"abcdefghijklmn\" #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-reverse \"abcdefghijklmn\" 1 #\\a)"),
            "E1002"
        );
        assert_eq!(do_lisp("(string-reverse \"abcdefghijklmn\" 1 20)"), "E1021");
        assert_eq!(do_lisp("(string-reverse \"abcdefghijklmn\" 6 5)"), "E1021");
    }
    #[test]
    fn string_upcase() {
        assert_eq!(do_lisp("(string-upcase)"), "E1007");
        assert_eq!(do_lisp("(string-upcase \"abcdefghijklmn\" 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-upcase 10)"), "E1015");
        assert_eq!(
            do_lisp("(string-upcase \"abcdefghijklmn\" #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-upcase \"abcdefghijklmn\" 1 #\\a)"),
            "E1002"
        );
        assert_eq!(do_lisp("(string-upcase \"abcdefghijklmn\" 1 20)"), "E1021");
        assert_eq!(do_lisp("(string-upcase \"abcdefghijklmn\" 6 5)"), "E1021");
    }
    #[test]
    fn string_downcase() {
        assert_eq!(do_lisp("(string-downcase)"), "E1007");
        assert_eq!(
            do_lisp("(string-downcase \"ABCDEFGHIJKLMN\" 2 3 4)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-downcase 10)"), "E1015");
        assert_eq!(
            do_lisp("(string-downcase \"ABCDEFGHIJKLMN\" #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-downcase \"ABCDEFGHIJKLMN\" 1 #\\a)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-downcase \"ABCDEFGHIJKLMN\" 1 20)"),
            "E1021"
        );
        assert_eq!(do_lisp("(string-downcase \"ABCDEFGHIJKLMN\" 6 5)"), "E1021");
    }
    #[test]
    fn string_index() {
        assert_eq!(do_lisp("(string-index)"), "E1007");
        assert_eq!(
            do_lisp("(string-index \"abcdefghijklmn\" 2 3 4 5)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-index 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-index \"abcdefghijklmn\" 10)"), "E1019");
        assert_eq!(
            do_lisp("(string-index \"abcdefghijklmn\" #\\a #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-index \"abcdefghijklmn\" #\\a  1 #\\a)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-index \"abcdefghijklmn\" #\\a 1 20)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-index \"abcdefghijklmn\" #\\a 6 5)"),
            "E1021"
        );
    }
    #[test]
    fn string_index_right() {
        assert_eq!(do_lisp("(string-index-right)"), "E1007");
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijklmn\" 2 3 4 5)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-index-right 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-index-right \"abc\" 10)"), "E1019");
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijklmn\" #\\a #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijklmn\" #\\a  1 #\\a)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijklmn\" #\\a 1 20)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-index-right \"abcdefghijklmn\" #\\a 6 5)"),
            "E1021"
        );
    }
    #[test]
    fn string_delete() {
        assert_eq!(do_lisp("(string-delete)"), "E1007");
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijklmn\" 2 3 4 5)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-delete 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-delete \"abc\" 10)"), "E1019");
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijklmn\" #\\a #\\a 1)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijklmn\" #\\a  1 #\\a)"),
            "E1002"
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijklmn\" #\\a 1 20)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-delete \"abcdefghijklmn\" #\\a 6 5)"),
            "E1021"
        );
    }
    #[test]
    fn string_trim() {
        assert_eq!(do_lisp("(string-trim)"), "E1007");
        assert_eq!(do_lisp("(string-trim \"abcdefghijklmn\" 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-trim 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-trim \"abcdefghijklmn\" 2)"), "E1019");
    }
    #[test]
    fn string_trim_right() {
        assert_eq!(do_lisp("(string-trim-right)"), "E1007");
        assert_eq!(
            do_lisp("(string-trim-right \"abcdefghijklmn\" 2 3 4)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-trim-right 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-trim-right \"abcdefghijklmn\" 2)"), "E1019");
    }
    #[test]
    fn string_trim_both() {
        assert_eq!(do_lisp("(string-trim-both)"), "E1007");
        assert_eq!(
            do_lisp("(string-trim-both \"abcdefghijklmn\" 2 3 4)"),
            "E1007"
        );
        assert_eq!(do_lisp("(string-trim-both 10 #\\a)"), "E1015");
        assert_eq!(do_lisp("(string-trim-both \"abcdefghijklmn\" 2)"), "E1019");
    }
    #[test]
    fn string_take() {
        assert_eq!(do_lisp("(string-take)"), "E1007");
        assert_eq!(do_lisp("(string-take 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-take 0 1)"), "E1015");
        assert_eq!(do_lisp("(string-take \"123456\" #\\a)"), "E1002");
        assert_eq!(do_lisp("(string-take \"abcdefghijklmn\" -1)"), "E1021");
        assert_eq!(do_lisp("(string-take \"abcdefghijklmn\" 15)"), "E1021");
    }
    #[test]
    fn string_take_right() {
        assert_eq!(do_lisp("(string-take-right)"), "E1007");
        assert_eq!(do_lisp("(string-take-right 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-take-right \"123456\" #\\a)"), "E1002");
        assert_eq!(
            do_lisp("(string-take-right \"abcdefghijklmn\" -1)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-take-right \"abcdefghijklmn\" 15)"),
            "E1021"
        );
    }
    #[test]
    fn string_drop() {
        assert_eq!(do_lisp("(string-drop)"), "E1007");
        assert_eq!(do_lisp("(string-drop 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-drop \"123456\" #\\a)"), "E1002");
        assert_eq!(do_lisp("(string-drop \"abcdefghijklmn\" -1)"), "E1021");
        assert_eq!(do_lisp("(string-drop \"abcdefghijklmn\" 15)"), "E1021");
    }
    #[test]
    fn string_drop_right() {
        assert_eq!(do_lisp("(string-drop-right)"), "E1007");
        assert_eq!(do_lisp("(string-drop-right 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-drop-right \"123456\" #\\a)"), "E1002");
        assert_eq!(
            do_lisp("(string-drop-right \"abcdefghijklmn\" -1)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-drop-right \"abcdefghijklmn\" 15)"),
            "E1021"
        );
    }
    #[test]
    fn string_take_u8() {
        assert_eq!(do_lisp("(string-take-u8)"), "E1007");
        assert_eq!(do_lisp("(string-take-u8 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-take-u8 0 1)"), "E1015");
        assert_eq!(do_lisp("(string-take-u8 \"123456\" #\\a)"), "E1002");
        assert_eq!(do_lisp("(string-take-u8 \"abcdefghijklmn\" -1)"), "E1021");
        assert_eq!(do_lisp("(string-take-u8 \"abcdefghijklmn\" 15)"), "E1021");
    }
    #[test]
    fn string_take_right_u8() {
        assert_eq!(do_lisp("(string-take-right-u8)"), "E1007");
        assert_eq!(do_lisp("(string-take-right-u8 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-take-right-u8 \"123456\" #\\a)"), "E1002");
        assert_eq!(
            do_lisp("(string-take-right-u8 \"abcdefghijklmn\" -1)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-take-right-u8 \"abcdefghijklmn\" 15)"),
            "E1021"
        );
    }
    #[test]
    fn string_drop_u8() {
        assert_eq!(do_lisp("(string-drop-u8)"), "E1007");
        assert_eq!(do_lisp("(string-drop-u8 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-drop-u8 \"123456\" #\\a)"), "E1002");
        assert_eq!(do_lisp("(string-drop-u8 \"abcdefghijklmn\" -1)"), "E1021");
        assert_eq!(do_lisp("(string-drop-u8 \"abcdefghijklmn\" 15)"), "E1021");
    }
    #[test]
    fn string_drop_right_u8() {
        assert_eq!(do_lisp("(string-drop-right-u8)"), "E1007");
        assert_eq!(do_lisp("(string-drop-right-u8 2 3 4)"), "E1007");
        assert_eq!(do_lisp("(string-drop-right-u8 \"123456\" #\\a)"), "E1002");
        assert_eq!(
            do_lisp("(string-drop-right-u8 \"abcdefghijklmn\" -1)"),
            "E1021"
        );
        assert_eq!(
            do_lisp("(string-drop-right-u8 \"abcdefghijklmn\" 15)"),
            "E1021"
        );
    }
}
