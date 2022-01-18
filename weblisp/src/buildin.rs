/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   hidekuno@gmail.com
*/
extern crate elisp;

use chrono::Utc;
use elisp::create_error;
use elisp::create_error_value;
use elisp::lisp;
use elisp::reference_obj;
use lisp::eval;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;
use lisp::Expression;
use lisp::ResultExpression;
use std::vec::Vec;

const REQUEST_COLUMNS: usize = 5;
pub const RESPONSE_COLUMNS: usize = 3;

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
    // get resource
    // ex. (web-get-resource request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-resource", |exp, env| get_value(exp, env, 3));

    //--------------------------------------------------------
    // get protocol
    // ex. (web-get-protocol request)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-protocol", |exp, env| get_value(exp, env, 4));

    //--------------------------------------------------------
    // create response
    // ex. (web-create-response status mime data)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-create-response", create_response);

    //--------------------------------------------------------
    // set session
    // ex. (web-set-session sid data)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-set-session", set_session);

    //--------------------------------------------------------
    // create response
    // ex. (web-create-response sid)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-get-session", get_session);

    //--------------------------------------------------------
    // debug log
    // ex. (web-debug-log exp)
    //--------------------------------------------------------
    env.add_builtin_ext_func("web-debug", log_debug);
}
fn get_value(exp: &[Expression], env: &Environment, idx: usize) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match lisp::eval(&exp[1], env)? {
        Expression::Vector(l) => l,
        e => return Err(create_error_value!(ErrCode::E1022, e)),
    };
    let l = &*(reference_obj!(l));
    if l.len() != REQUEST_COLUMNS {
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
    let l = &*(reference_obj!(l));
    if l.len() != REQUEST_COLUMNS {
        return Err(create_error!(ErrCode::E1021));
    }
    let l = match &l[idx] {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l = &*(reference_obj!(l));
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
fn create_response(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != RESPONSE_COLUMNS + 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let status = match lisp::eval(&exp[1], env)? {
        Expression::Integer(i) => i,
        e => return Err(create_error_value!(ErrCode::E1002, e)),
    };

    let mime = match lisp::eval(&exp[2], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };

    Ok(Environment::create_vector(vec![
        Expression::Integer(status),
        Expression::String(mime),
        lisp::eval(&exp[3], env)?,
    ]))
}
fn set_session(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let key = match lisp::eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let value = lisp::eval(&exp[2], env)?;

    if env.find(&key).is_some() {
        env.update(&key, value);
    } else {
        env.regist_root(key.to_string(), value);
    }
    Ok(Expression::String(key))
}
fn get_session(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let key = match lisp::eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    match env.find(&key) {
        Some(e) => Ok(e),
        None => Ok(Environment::create_list(Vec::new())),
    }
}
fn log_debug(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let value = lisp::eval(&exp[1], env)?;
    println!(
        "SCM-DEBUG [{}]: {}",
        Utc::now().to_string(),
        value.to_string()
    );
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
  \"/test.scm\"
  \"HTTP/1.0\"
)
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
    fn web_get_protocol() {
        let env = init();
        let lisp = format!("(web-get-protocol {})", create_data());
        assert_eq!(do_lisp_env(&lisp, &env), "\"HTTP/1.0\"");
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

    #[test]
    fn web_create_response() {
        let env = init();
        assert_eq!(
            do_lisp_env("(web-create-response 200 \"txt\" 10)", &env),
            "#(200 \"txt\" 10)"
        );
    }
    #[test]
    fn web_set_session() {
        let env = init();
        assert_eq!(
            do_lisp_env("(web-set-session \"RE-1641717077-3\" 10)", &env),
            "\"RE-1641717077-3\""
        );
    }
    #[test]
    fn web_get_session() {
        let env = init();
        do_lisp_env("(web-set-session \"RE-1641717077-3\" 20)", &env);
        assert_eq!(
            do_lisp_env("(web-get-session \"RE-1641717077-3\")", &env),
            "20"
        );
    }
    #[test]
    fn web_debug() {
        let env = init();
        assert_eq!(do_lisp_env("(web-debug 10)", &env), "nil");
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
            do_lisp_env("(web-get-method #(10 10 10 10 10))", &env),
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
            do_lisp_env(
                "(web-get-header \"User-Agent\" #(1 (list 10) 10 10 10))",
                &env
            ),
            "E1005"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-header \"User-Agent\" #(1 (list (cons 10 20)) 10 10 10))",
                &env
            ),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-header \"User-Agent\" #(1 (list (cons \"User-Agent\" 20)) 10 10 10))",
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
            do_lisp_env(
                "(web-get-parameter \"Value1\" #(1 10 (list 10) 10 10))",
                &env
            ),
            "E1005"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-parameter \"Value1\" #(1 10 (list (cons 10 20)) 10 10))",
                &env
            ),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                "(web-get-parameter \"Value1\" #(1 10 (list (cons \"Value1\" 20)) 10 10))",
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
            do_lisp_env("(web-get-resource #(10 10 10 10 10))", &env),
            "E1005"
        );
    }
    #[test]
    fn web_get_protocol() {
        let env = init();
        assert_eq!(do_lisp_env("(web-get-protocol)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-protocol 1 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-protocol 10)", &env), "E1022");
        assert_eq!(do_lisp_env("(web-get-protocol #(10))", &env), "E1021");
        assert_eq!(
            do_lisp_env("(web-get-protocol #(10 10 10 10 10))", &env),
            "E1005"
        );
    }
    #[test]
    fn web_create_response() {
        let env = init();
        assert_eq!(do_lisp_env("(web-create-response)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-create-response 1 2 3 4)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-create-response #\\a 2 3)", &env), "E1002");
        assert_eq!(do_lisp_env("(web-create-response 200 2 3)", &env), "E1015");
        assert_eq!(
            do_lisp_env("(web-create-response 200 \"txt\" a)", &env),
            "E1008"
        );
    }
    #[test]
    fn web_set_session() {
        let env = init();
        assert_eq!(do_lisp_env("(web-set-session 1)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-set-session 1 2 3)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-set-session 10 3)", &env), "E1015");
        assert_eq!(do_lisp_env("(web-set-session \"a\" a)", &env), "E1008");
    }
    #[test]
    fn web_get_session() {
        let env = init();
        assert_eq!(do_lisp_env("(web-get-session)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-session 1 2)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-session 10)", &env), "E1015");
    }
    #[test]
    fn web_debug() {
        let env = init();
        assert_eq!(do_lisp_env("(web-debug)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-debug 10 10)", &env), "E1007");
        assert_eq!(do_lisp_env("(web-get-session a)", &env), "E1008");
    }
}
