/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::{eval, repl};
use crate::lisp::{Expression, ResultExpression};
use crate::lisp::{RsCode, RsError};

#[cfg(feature = "thread")]
use crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::Environment;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("load-file", load_file);
    b.regist("display", display);
    b.regist("newline", newline);
}
fn load_file(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::String(s) = v {
        if false == Path::new(&s).exists() {
            return Err(create_error!(RsCode::E1014));
        }
        let file = match File::open(s) {
            Err(e) => return Err(create_error_value!(RsCode::E1014, e)),
            Ok(file) => file,
        };
        let meta = match file.metadata() {
            Err(e) => return Err(create_error_value!(RsCode::E9999, e)),
            Ok(meta) => meta,
        };
        if true == meta.is_dir() {
            return Err(create_error!(RsCode::E1016));
        }
        let mut stream = BufReader::new(file);
        match repl(&mut stream, env, true) {
            Err(e) => return Err(create_error_value!(RsCode::E9999, e)),
            Ok(_) => return Ok(Expression::Nil()),
        }
    }
    Err(create_error!(RsCode::E1015))
}
fn display(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        let v = eval(e, env)?;
        if let Expression::Char(c) = v {
            print!("{} ", c);
        } else {
            print!("{} ", v.to_string());
        }
    }
    Ok(Expression::Nil())
}
fn newline(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(RsCode::E1007, exp.len()));
    }
    print!("\n");
    Ok(Expression::Nil())
}
