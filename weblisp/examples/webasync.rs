/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  ref) https://rust-lang.github.io/async-book/09_example/02_handling_connections_concurrently.html

       RUST_LOG=info cargo run --example webasync

  ex) curl 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'

  ex) curl -v -c /tmp/cookie.txt http://localhost:9000/samples/test.scm
      curl -v -b /tmp/cookie.txt http://localhost:9000/samples/test.scm

  hidekuno@gmail.com
*/
extern crate elisp;
extern crate env_logger;
extern crate weblisp;

use elisp::lisp;
use weblisp::web;

use std::io::Read;
use std::io::Write;

use chrono::Local;
use chrono::Utc;
use env_logger::Builder;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use async_std::io::ReadExt;
use async_std::io::WriteExt;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::task;
use futures::StreamExt;
use weblisp::buildin;
use weblisp::config::BIND_ADDRESS;
use weblisp::http_value_error;

use web::dispatch;
use web::make_cookie_header;
use web::parse_request;
use web::Contents;
use web::Method;
use web::CRLF;
use web::MIME_PLAIN;
use web::PROTOCOL;
use web::RESPONSE_400;

const MAX_ID: usize = 1000;

macro_rules! http_write {
    ($s: expr, $v: expr) => {
        $s.write_all($v.as_bytes()).await.unwrap();
        $s.write_all(CRLF.as_bytes()).await.unwrap();
    };
}
pub async fn entry_async_proc(
    mut stream: TcpStream,
    env: lisp::Environment,
    buffer: &[u8],
    id: usize,
) {
    let r = parse_request(buffer);
    let (status, contents, mime, cookie) = match &r {
        Ok(r) => dispatch(r, env, id),
        Err(e) => http_value_error!(RESPONSE_400, e),
    };

    info!("{} {}", status.0, status.1);
    http_write!(stream, format!("{} {} {}", PROTOCOL, status.0, status.1));

    let res_header: [&str; 3] = [
        &Utc::now()
            .format("Date: %a, %d %h %Y %H:%M:%S GMT")
            .to_string()
            .into_boxed_str(),
        "Server: Rust eLisp",
        "Connection: closed",
    ];
    for h in &res_header {
        http_write!(stream, h);
    }
    if let Some(cookie) = cookie {
        http_write!(stream, make_cookie_header(cookie));
    }
    http_write!(stream, format!("Content-type: {}", mime));
    if !contents.is_empty() {
        http_write!(stream, format!("Content-length: {}", contents.len()));
    }
    let r = r.unwrap();
    match r.get_method() {
        Some(Method::Head) => {}
        None => {}
        _ => {
            stream.write_all(CRLF.as_bytes()).await.unwrap();
            match contents {
                Contents::String(v) => {
                    stream.write(v.as_bytes()).await.unwrap();
                }
                Contents::File(mut v) => {
                    let mut buffer = [0; 2048];
                    loop {
                        let n = v.file.read(&mut buffer).unwrap();
                        if n == 0 {
                            break;
                        }
                        stream.write(&buffer[..n]).await.unwrap();
                    }
                }
                _ => {}
            }
        }
    }
    stream.flush().await.unwrap();
}
async fn handle_connection(mut stream: TcpStream, env: lisp::Environment, id: usize) {
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).await.unwrap();
    entry_async_proc(stream, env, &buffer, id).await;
}

#[async_std::main]
async fn main() {
    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| {
            let m = if let Some(m) = record.module_path() {
                m
            } else {
                ""
            };
            writeln!(
                buf,
                "{} {:<6} {:<20} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                m,
                record.args()
            )
        })
        .init();
    info!("async server is starting...");

    let mut id = 1;
    let env = lisp::Environment::new();
    buildin::build_lisp_function(&env);

    let listener = TcpListener::bind(BIND_ADDRESS).await.unwrap();
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream.unwrap();
        let env = env.clone();
        task::spawn(async move {
            handle_connection(stream, env, id).await;
        });
        id += 1;
        if MAX_ID < id {
            id = 1;
        }
    }
}
