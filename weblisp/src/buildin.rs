/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   hidekuno@gmail.com
*/
extern crate elisp;

use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use elisp::referlence_list;
use lisp::eval;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;
use lisp::Expression;
use lisp::ResultExpression;

const DATA_COLUMNS: usize = 4;

pub fn build_lisp_function(env: &Environment) {
    //--------------------------------------------------------
    // get method
    // ex. (web-get-method request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-method", |exp, env| get_value(exp, env, 0));

    //--------------------------------------------------------
    // get header value
    // ex. (web-get-header "User-Agent" request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-header", |exp, env| get_key_value(exp, env, 1));

    //--------------------------------------------------------
    // get parameter value
    // ex. (web-get-parameter "Foo" request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-parameter", |exp, env| get_key_value(exp, env, 2));

    //--------------------------------------------------------
    // get method
    // ex. (web-get-resource request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-resource", |exp, env| get_value(exp, env, 3));
}
fn get_value(exp: &[Expression], env: &Environment, idx: usize) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match lisp::eval(&exp[1], env)? {
        Expression::Vector(l) => l,
        e => return Err(create_error_value!(ErrCode::E1022, e)),
    };
    let l = &*(referlence_list!(l));
    if l.len() != DATA_COLUMNS {
        return Err(create_error!(ErrCode::E1021));
    }
    match &l[idx] {
        Expression::String(_) => Ok(l[idx].clone()),
        e => Err(create_error_value!(ErrCode::E1005, e)),
    }
}
fn get_key_value(exp: &[Expression], env: &Environment, idx: usize) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let key = match lisp::eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let l = match lisp::eval(&exp[2], env)? {
        Expression::Vector(l) => l,
        e => return Err(create_error_value!(ErrCode::E1022, e)),
    };
    let l = &*(referlence_list!(l));
    if l.len() != DATA_COLUMNS {
        return Err(create_error!(ErrCode::E1021));
    }
    let l = match &l[idx] {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l = &*(referlence_list!(l));
    for v in l {
        match eval(v, env)? {
            Expression::Pair(car, cdr) => match *car {
                Expression::String(s) => {
                    if s == key {
                        match *cdr {
                            Expression::String(_) => return Ok(*cdr),
                            e => return Err(create_error_value!(ErrCode::E1015, e)),
                        }
                    }
                }
                e => return Err(create_error_value!(ErrCode::E1015, e)),
            },
            e => return Err(create_error_value!(ErrCode::E1005, e)),
        }
    }
    Ok(Expression::Nil())
}
#[cfg(test)]
fn do_lisp_env(program: &str, env: &Environment) -> String {
    match elisp::lisp::do_core_logic(program, env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[cfg(test)]
fn init() -> Environment {
    let env = Environment::new();
    build_lisp_function(&env);
    env
}

#[cfg(test)]
fn create_data<'a>() -> &'a str {
    "
#(\"GET\"
  (list (cons \"Host\" \"www.mukogawa.or.jp\")(cons \"User-Agent\" \"rust\"))
  (list (cons \"Value1\" \"10\")(cons \"Value2\" \"20\"))
  \"/test.scm\")
"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_get_method() {
        let env = init();
        let lisp = format!("(web-get-method {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "\"GET\"");
    }
    #[test]
    fn web_get_resource() {
        let env = init();
        let lisp = format!("(web-get-resource {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "\"/test.scm\"");
    }
    #[test]
    fn web_get_header() {
        let env = init();

        let lisp = format!("(web-get-header \"User-Agent\" {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "\"rust\"");

        let lisp = format!("(web-get-header \"No-Data\" {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "nil");
    }
    #[test]
    fn web_get_parameter() {
        let env = init();

        let lisp = format!("(web-get-parameter \"Value1\" {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "\"10\"");

        let lisp = format!("(web-get-parameter \"No-Data\" {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "nil");
    }
}
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn web_get_method() {
        let env = init();
        assert_eq!(do_lisp_env("(web-get-method)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-method 1 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-method 10)", &env), "E1022");
        assert_eq!(do_lisp_env("(web-get-method #(10))", &env), "E1021");
        assert_eq!(
            do_lisp_env("(web-get-method #(10 10 10 10))", &env),
            "E1005"
        );
    }
    #[test]
    fn web_get_header() {
        let env = init();

        assert_eq!(do_lisp_env("(web-get-header)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-header 1 2 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-header 10 3)", &env), "E1015");
        assert_eq!(
            do_lisp_env("(web-get-header \"User-Agent\" 10)", &env),
            "E1022"
        );
        assert_eq!(
            do_lisp_env("(web-get-header \"User-Agent\" #(1 10))", &env),
            "E1021"
        );
        assert_eq!(
            do_lisp_env("(web-get-header \"User-Agent\" #(1 (list 10) 10 10))", &env),
            "E1005"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-header \"User-Agent\" #(1 (list (cons 10 20)) 10 10))",
                &env
            ),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-header \"User-Agent\" #(1 (list (cons \"User-Agent\" 20)) 10 10))",
                &env
            ),
            "E1015"
        );
    }
    #[test]
    fn web_get_parameter() {
        let env = init();

        assert_eq!(do_lisp_env("(web-get-parameter)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-parameter 1 2 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-parameter 10 3)", &env), "E1015");
        assert_eq!(
            do_lisp_env("(web-get-parameter \"Value1\" 10)", &env),
            "E1022"
        );
        assert_eq!(
            do_lisp_env("(web-get-parameter \"Value1\" #(10))", &env),
            "E1021"
        );
        assert_eq!(
            do_lisp_env("(web-get-parameter \"Value1\" #(1 10 (list 10) 10))", &env),
            "E1005"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-parameter \"Value1\" #(1 10 (list (cons 10 20)) 10))",
                &env
            ),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-parameter \"Value1\" #(1 10 (list (cons \"Value1\" 20)) 10))",
                &env
            ),
            "E1015"
        );
    }
    #[test]
    fn web_get_resource() {
        let env = init();
        assert_eq!(do_lisp_env("(web-get-resource)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-resource 1 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-resource 10)", &env), "E1022");
        assert_eq!(do_lisp_env("(web-get-resource #(10))", &env), "E1021");
        assert_eq!(
            do_lisp_env("(web-get-resource #(10 10 10 10))", &env),
            "E1005"
        );
    }
}
