/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use crate::create_error;
use crate::create_error_value;

use crate::buildin::BuildInTable;
use crate::lisp::{count_parenthesis, eval, parse, repl, tokenize};
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::lisp::{NEWLINE, SPACE, TAB};

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("load-file", load_file);
    b.regist("display", display);
    b.regist("newline", newline);
    b.regist("read", |exp, env| {
        read(exp, env, &mut BufReader::new(std::io::stdin()))
    });
    b.regist("read-char", |exp, env| {
        read_char(exp, env, &mut BufReader::new(std::io::stdin()))
    });
}
fn load_file(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::String(s) = v {
        if !Path::new(s.as_ref()).exists() {
            return Err(create_error!(ErrCode::E1014));
        }
        let file = match File::open(s.as_ref()) {
            Err(e) => return Err(create_error_value!(ErrCode::E1014, e)),
            Ok(file) => file,
        };
        let meta = match file.metadata() {
            Err(e) => return Err(create_error_value!(ErrCode::E9999, e)),
            Ok(meta) => meta,
        };
        if meta.is_dir() {
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
    for e in &exp[1..] {
        let v = eval(e, env)?;
        if let Expression::Char(c) = v {
            print!("{} ", c);
        } else if let Expression::String(s) = v {
            print!("{} ", s);
        } else {
            print!("{} ", v);
        }
        std::io::stdout().flush().unwrap();
    }
    Ok(Expression::Nil())
}
fn newline(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    println!();
    Ok(Expression::Nil())
}
fn read(exp: &[Expression], env: &Environment, stream: &mut dyn BufRead) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }

    let mut buffer = String::new();
    let mut expression: Vec<String> = Vec::new();

    let result = loop {
        buffer.clear();
        let n = match stream.read_line(&mut buffer) {
            Ok(n) => n,
            Err(_) => return Err(create_error!(ErrCode::E9999)),
        };
        if n == 1 {
            continue;
        }
        expression.push(buffer.trim().to_string());
        let lisp = expression.join(" ");
        let (left, right) = count_parenthesis(&lisp);
        if left > right {
            continue;
        }
        let token = tokenize(&lisp);
        break parse(&token, &mut 1, env);
    };
    result
}
fn read_char(exp: &[Expression], env: &Environment, stream: &mut dyn BufRead) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }

    let mut buffer = [0; 4];
    let mut exp_char = String::new();

    // pub trait BufRead: Read {
    //    ...
    // }
    let n = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return Err(create_error!(ErrCode::E9999)),
    };
    if n == 0 {
        exp_char.push_str(NEWLINE.1);
    } else {
        let s = match std::str::from_utf8(&buffer) {
            Ok(s) => s,
            Err(_) => return Err(create_error!(ErrCode::E9999)),
        };
        if let Some(c) = s.chars().next() {
            match c {
                ' ' => exp_char.push_str(SPACE.1),
                '\t' => exp_char.push_str(TAB.1),
                _ => {
                    exp_char.push_str("#\\");
                    exp_char.push(buffer[0] as char);
                }
            }
        }
    }
    parse(&[exp_char], &mut 1, env)
}
#[cfg(test)]
mod tests {
    use crate::io;
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};
    use lisp::Expression;
    use std::env;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::Write;
    use std::path::Path;

    fn read_char_test(data: &str) -> String {
        let mut cur = Cursor::new(data.as_bytes());
        let env = lisp::Environment::new();
        match io::read_char(&[Expression::Nil()], &env, &mut cur) {
            Ok(s) => s.to_string(),
            Err(_) => "error".to_string(),
        }
    }
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
        writeln!(file, "(define  s1 \"(\"))").unwrap();
        writeln!(file, "(define  s2 \")\"))").unwrap();
        writeln!(file, "(define  c1 #\\()").unwrap();
        writeln!(file, "(define  c2 #\\))").unwrap();
        writeln!(file, "(define  (testf a b)\n(+ a b))").unwrap();
        writeln!(file, "(define  (teststr)\"(\")").unwrap();
        writeln!(file, "(define  (testchr)\n#\\()").unwrap();
        // Error for testing.
        writeln!(file, "(load-file)").unwrap();
        writeln!(file, "(quit)").unwrap();

        file.flush().unwrap();

        let env = lisp::Environment::new();
        let f = test_file.as_path().to_str().expect("die");
        do_lisp_env(format!("(load-file \"{}\")", f).as_str(), &env);
        assert_eq!(do_lisp_env("foo", &env), "100");
        assert_eq!(do_lisp_env("hoge", &env), "200");
        assert_eq!(do_lisp_env("fuga", &env), "300");
        assert_eq!(do_lisp_env("(+ a b c)", &env), "600");
        assert_eq!(do_lisp_env("s1", &env), "\"(\"");
        assert_eq!(do_lisp_env("s2", &env), "\")\"");
        assert_eq!(do_lisp_env("c1", &env), "#\\(");
        assert_eq!(do_lisp_env("c2", &env), "#\\)");
        assert_eq!(do_lisp_env("(testf 10 20)", &env), "30");
        assert_eq!(do_lisp_env("(teststr)", &env), "\"(\"");
        assert_eq!(do_lisp_env("(testchr)", &env), "#\\(");
    }
    #[test]
    fn display() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_eq!(do_lisp_env("(display \"abc\")", &env), "nil");
        assert_eq!(do_lisp_env("(display #\\a)", &env), "nil");
        assert_eq!(do_lisp_env("(display a)", &env), "nil");
    }
    #[test]
    fn newline() {
        assert_eq!(do_lisp("(newline)"), "nil");
    }
    #[test]
    fn read() {
        let mut cur = Cursor::new("abcdef".as_bytes());
        let env = lisp::Environment::new();
        if let Ok(s) = io::read(&[Expression::Nil()], &env, &mut cur) {
            assert_eq!(s.to_string(), "abcdef")
        }

        let mut cur = Cursor::new("1".as_bytes());
        if let Ok(s) = io::read(&[Expression::Nil()], &env, &mut cur) {
            assert_eq!(s.to_string(), "1")
        }
        let mut cur = Cursor::new("(2\n )".as_bytes());
        if let Ok(s) = io::read(&[Expression::Nil()], &env, &mut cur) {
            assert_eq!(s.to_string(), "(2)")
        }
    }
    #[test]
    fn read_char() {
        assert_eq!(read_char_test("a"), "#\\a");
        assert_eq!(read_char_test("\n"), "#\\newline");
        assert_eq!(read_char_test("\t"), "#\\tab");
        assert_eq!(read_char_test(" "), "#\\space");
        assert_eq!(read_char_test(""), "#\\newline");
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
        assert_eq!(do_lisp("(load-file \"/etc/shadow\")"), "E1014");
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
    #[test]
    fn read() {
        assert_eq!(do_lisp("(read 123)"), "E1007");
    }
    #[test]
    fn read_char() {
        assert_eq!(do_lisp("(read-char 123)"), "E1007");
    }
}
