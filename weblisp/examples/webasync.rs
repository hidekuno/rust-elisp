/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://rust-lang.github.io/async-book/09_example/02_handling_connections_concurrently.html

   ex) curl 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'

   ex) curl -v -c /tmp/cookie.txt http://localhost:9000/samples/test.scm
       curl -v -b /tmp/cookie.txt http://localhost:9000/samples/test.scm

   hidekuno@gmail.com
 */
extern crate elisp;
extern crate weblisp;

use elisp::lisp;
use weblisp::web;

use chrono::Duration;
use chrono::Utc;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::io::ReadExt;
use async_std::io::WriteExt;
use futures::StreamExt;
use async_std::task;
use weblisp::config::BIND_ADDRESS;
use weblisp::buildin;
use weblisp::http_value_error;

use web::PROTOCOL;
use web::SESSION_ID;
use web::Contents;
use web::CRLF;
use web::dispatch;
use web::parse_request;
use web::Method;
use web::RESPONSE_400;
use web::MIME_PLAIN;

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
)
{
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
        let expire = Utc::now() + Duration::days(365);
        http_write!(
            stream,
            format!(
                "Set-Cookie: {}={};expires={};",
                SESSION_ID,
                cookie,
                expire
                    .format("%a, %d %h %Y %H:%M:%S GMT")
                    .to_string()
                    .into_boxed_str()
            )
        );
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
            if let Contents::String(v) = contents {
                 stream.write(v.as_bytes()).await.unwrap();
            };
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
    }
}
