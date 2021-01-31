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
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};

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
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::String(s) = v {
        if false == Path::new(&s).exists() {
            return Err(create_error!(ErrCode::E1014));
        }
        let file = match File::open(s) {
            Err(e) => return Err(create_error_value!(ErrCode::E1014, e)),
            Ok(file) => file,
        };
        let meta = match file.metadata() {
            Err(e) => return Err(create_error_value!(ErrCode::E9999, e)),
            Ok(meta) => meta,
        };
        if true == meta.is_dir() {
            return Err(create_error!(ErrCode::E1016));
        }
        let mut stream = BufReader::new(file);
        match repl(&mut stream, env, None) {
            Err(e) => return Err(create_error_value!(ErrCode::E9999, e)),
            Ok(_) => return Ok(Expression::Nil()),
        }
    }
    Err(create_error!(ErrCode::E1015))
}
fn display(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
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
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    print!("\n");
    Ok(Expression::Nil())
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    #[allow(unused_must_use)]
    fn load_file() {
        let test_dir = Path::new(&env::var("HOME").unwrap()).join("tmp");
        let test_file = test_dir.join("test.scm");

        std::fs::create_dir(test_dir);
        std::fs::remove_file(&test_file);

        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "(define foo 100)").unwrap();
        writeln!(file, "(define hoge 200)").unwrap();
        writeln!(file, "(define fuga (+ foo hoge))").unwrap();
        writeln!(file, "(define a 100)(define b 200)(define c 300)").unwrap();
        writeln!(file, "(define d 100)").unwrap();
        file.flush().unwrap();

        let env = lisp::Environment::new();
        let f = test_file.as_path().to_str().expect("die");
        do_lisp_env(format!("(load-file \"{}\")", f).as_str(), &env);
        assert_eq!(do_lisp_env("foo", &env), "100");
        assert_eq!(do_lisp_env("hoge", &env), "200");
        assert_eq!(do_lisp_env("fuga", &env), "300");
        assert_eq!(do_lisp_env("(+ a b c)", &env), "600");
    }
    #[test]
    fn display() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_eq!(do_lisp_env("(display a)", &env), "nil");
    }
    #[test]
    fn newline() {
        assert_eq!(do_lisp("(newline)"), "nil");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;
    #[test]
    fn load_file() {
        assert_eq!(do_lisp("(load-file)"), "E1007");
        assert_eq!(do_lisp("(load-file 1 2)"), "E1007");
        assert_eq!(do_lisp("(load-file hoge)"), "E1008");
        assert_eq!(do_lisp("(load-file #t)"), "E1015");
        assert_eq!(do_lisp("(load-file \"/etc/test.scm\")"), "E1014");
        assert_eq!(do_lisp("(load-file \"/tmp\")"), "E1016");
        assert_eq!(do_lisp("(load-file \"/bin/cp\")"), "E9999");
    }
    #[test]
    fn display() {
        assert_eq!(do_lisp("(display)"), "E1007");
        assert_eq!(do_lisp("(display a)"), "E1008");
    }
    #[test]
    fn newline() {
        assert_eq!(do_lisp("(newline 123)"), "E1007");
    }
}
