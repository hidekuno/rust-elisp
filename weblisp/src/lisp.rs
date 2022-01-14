/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.
   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   ex) curl 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'

   ex) curl -v -c /tmp/cookie.txt http://localhost:9000/samples/test.scm
       curl -v -b /tmp/cookie.txt http://localhost:9000/samples/test.scm

   hidekuno@gmail.com
*/
extern crate elisp;
use crate::buildin;
use crate::web;
use elisp::lisp;

use chrono::Utc;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::http_error;
use crate::http_value_error;
use crate::make_path;

use web::get_mime;
use web::get_status;
use web::set_path_security;
use web::Contents;
use web::Request;
use web::WebResult;
use web::CRLF;
use web::LISP_EXT;
use web::MIME_PLAIN;
use web::RESPONSE_200;
use web::RESPONSE_400;
use web::RESPONSE_404;
use web::RESPONSE_500;
use web::SESSION_ID;

const LISP_PARAMNAME: &str = "expr=";

pub fn do_repl(r: &Request, env: lisp::Environment) -> WebResult {
    if !r.get_parameter().starts_with(LISP_PARAMNAME) {
        return http_error!(RESPONSE_400);
    }
    let (_, expr) = r.get_parameter().split_at(LISP_PARAMNAME.len());
    let mut result = match lisp::do_core_logic(&expr.to_string(), &env) {
        Ok(r) => r.to_string(),
        Err(e) => {
            if lisp::ErrCode::E9000.as_str() == e.get_code() {
                env.set_force_stop(false);
            }
            e.get_msg()
        }
    };
    result.push_str(CRLF);
    (
        RESPONSE_200,
        Contents::String(result),
        Some(MIME_PLAIN.1),
        None,
    )
}
pub fn do_scm(r: &Request, env: lisp::Environment, id: usize) -> WebResult {
    let path = make_path!(r.get_resource());

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    let path = Path::new(r.get_resource());
    let f = match path.file_name() {
        Some(f) => match f.to_str() {
            Some(f) => f.replace(LISP_EXT, ""),
            None => return http_error!(RESPONSE_500),
        },
        None => return http_error!(RESPONSE_500),
    };

    let mut load_file = String::new();
    if file.read_to_string(&mut load_file).is_err() {
        return http_error!(RESPONSE_500);
    }
    match lisp::do_core_logic(&load_file, &env) {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e.get_msg());
            return http_value_error!(RESPONSE_500, e.get_msg());
        }
    };

    let method = if let Some(method) = r.get_method() {
        method.as_str().to_string()
    } else {
        String::from("")
    };
    let (sid, first) = get_session_id(r, id);

    let lisp = format!(
        "((lambda () ({}::main #({:#?} {} {} {:#?} {:#?}) {:#?})))",
        f,
        method,
        r.get_lisp_header(),
        r.get_lisp_param(),
        r.get_resource(),
        r.get_protocol(),
        sid,
    );

    let result = match lisp::do_core_logic(&lisp, &env) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e.get_msg());
            return http_value_error!(RESPONSE_500, e.get_msg());
        }
    };
    let cookie = if first { Some(sid) } else { None };
    parse_lisp_result(result, env, cookie)
}
fn get_session_id(r: &Request, id: usize) -> (String, bool) {
    for e in r.get_headers() {
        if e.starts_with("Cookie:") && e.contains(SESSION_ID) {
            // Cookie: RUST-ELISP-SID=RE-1641713444-1
            let s = format!("Cookie: {}=", SESSION_ID);
            if e.contains(&s) {
                return (e[s.len()..].to_string(), false);
            }
        }
    }
    let session_id = format!("RE-{}-{}", Utc::now().timestamp(), id);
    (session_id, true)
}
fn parse_lisp_result(
    exp: lisp::Expression,
    env: lisp::Environment,
    cookie: Option<String>,
) -> WebResult {
    let l = match lisp::eval(&exp, &env) {
        Ok(v) => match v {
            lisp::Expression::Vector(l) => l,
            _ => return http_error!(RESPONSE_500),
        },
        Err(e) => return http_value_error!(RESPONSE_500, e.get_msg()),
    };
    let l = &*(elisp::referlence_list!(l));
    if l.len() != buildin::RESPONSE_COLUMNS {
        return http_error!(RESPONSE_500);
    }

    let status = match lisp::eval(&l[0], &env) {
        Ok(v) => match v {
            lisp::Expression::Integer(i) => i,
            _ => return http_error!(RESPONSE_500),
        },
        Err(e) => return http_value_error!(RESPONSE_500, e.get_msg()),
    };

    let mime = match lisp::eval(&l[1], &env) {
        Ok(v) => match v {
            lisp::Expression::String(s) => s,
            _ => return http_error!(RESPONSE_500),
        },
        Err(e) => return http_value_error!(RESPONSE_500, e.get_msg()),
    };
    (
        get_status(status),
        Contents::String(l[2].to_string()),
        Some(get_mime(&(".".to_owned() + &mime))),
        cookie,
    )
}
