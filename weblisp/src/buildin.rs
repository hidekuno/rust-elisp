/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html
   ref) https://docs.oracle.com/cd/E17802_01/products/products/servlet/2.5/docs/servlet-2_5-mr2/javax/servlet/ServletRequest.html

   hidekuno@gmail.com
*/
extern crate elisp;

use elisp::create_error_value;
use elisp::lisp;
use elisp::referlence_list;
use lisp::eval;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Error;
use lisp::Expression;
use lisp::ResultExpression;

pub fn build_lisp_function(env: &Environment) {
    //--------------------------------------------------------
    // get header value
    // ex. (get-header "User-Agent" hdr)
    //--------------------------------------------------------
    env.add_builtin_ext_func("get-header", get_value);

    //--------------------------------------------------------
    // get parameter value
    // ex. (get-parameter "Foo" req)
    //--------------------------------------------------------
    env.add_builtin_ext_func("get-parameter", get_value);
}
fn get_value(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let key = match lisp::eval(&exp[1], env)? {
        Expression::String(s) => s,
        e => return Err(create_error_value!(ErrCode::E1015, e)),
    };
    let l = match lisp::eval(&exp[2], env)? {
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
