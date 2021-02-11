/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
// cargo run --example load_url
//
extern crate elisp;
extern crate surf;

use crate::elisp::create_error;
use surf::http::StatusCode;
use elisp::lisp;
use std::io;
use lisp::Environment;
use lisp::ErrCode;
use lisp::Expression;
use lisp::Error;
use lisp::repl;
use lisp::eval;
use async_std::task;

fn load_url(url: &String) -> Result<(String,StatusCode), Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(
        async {
            let mut res = surf::get(url).await?;
            let lisp = res.body_string().await?;
            Ok((lisp, res.status()))
        }
    )
}
pub fn build_lisp_function(env: &Environment) {
    env.add_builtin_ext_func("load-url", move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error!(ErrCode::E1007));
        }
        let url = if let Expression::String(s) = eval(&exp[1],env)? {
            s
        } else {
            return Err(create_error!(ErrCode::E1015));
        };
        if url.starts_with("http://") == false && url.starts_with("https://") == false {
            return Err(create_error!(ErrCode::E1021));
        }

        let lisp = match load_url(&url) {
            Err(e) => {
                println!("{:?}", e);
                return Err(create_error!(ErrCode::E9999));
            },
            Ok(s) => {
                if s.1 != 200 {
                    println!("{}", s.1);
                    return Err(create_error!(ErrCode::E9999));
                }
                s.0
            }
        };
        println!("{}",lisp);
        let mut cursor =io::Cursor::new(lisp.into_bytes());
        match repl(&mut cursor,&env, None) {
            Err(e) => println!("{}", e),
            Ok(_) => {}
        }
        Ok(Expression::Nil())
    });
}

const PROGRAM_URL: &'static str =
    "https://raw.githubusercontent.com/hidekuno/rust-elisp/master/elisp/samples/oops.scm";

fn main() {
    let env = Environment::new();
    build_lisp_function(&env);

    let url = format!("(load-url \"{}\")", PROGRAM_URL);

    let r = match lisp::do_core_logic(&url, &env) {
        Ok(r) => r.to_string(),
        Err(e) =>e.get_msg(),
    };
    println!("{}", r);
}
