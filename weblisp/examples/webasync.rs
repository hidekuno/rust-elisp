/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  ref) https://rust-lang.github.io/async-book/09_example/02_handling_connections_concurrently.html

       RUST_LOG=info cargo run --example webasync
       cargo test --example webasync -- --test-threads=1 --nocapture

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
use async_std::net::Shutdown;
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
use web::Request;
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
    write_contents(r.unwrap(), contents, &mut stream).await;

    stream.flush().await.unwrap();

    stream.shutdown(Shutdown::Both).unwrap();
}
async fn write_contents(r: Request, contents: Contents, stream: &mut TcpStream) {
    macro_rules! copy {
        ($r: expr, $s: expr) => {
            let mut buffer = [0; 2048];
            loop {
                let n = $r.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                $s.write(&buffer[..n]).await.unwrap();
            }
        };
    }
    match r.get_method() {
        Some(Method::Get) | Some(Method::Post) => {
            stream.write_all(CRLF.as_bytes()).await.unwrap();
            match contents {
                Contents::String(v) => {
                    stream.write(v.as_bytes()).await.unwrap();
                }
                Contents::File(mut v) => {
                    copy!(v.file, stream);
                }
                Contents::Cgi(mut v) => {
                    copy!(v, stream);
                }
            }
        }
        Some(Method::Head) => {}
        _ => {}
    }
}
async fn handle_connection(mut stream: TcpStream, env: lisp::Environment, id: usize) {
    let mut buffer = [0; 2048];
    let n = stream.read(&mut buffer).await.unwrap();
    entry_async_proc(stream, env, &buffer[..n], id).await;
}

#[async_std::main]
async fn main() {
    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| {
            let m = record.module_path().unwrap_or_default();
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
#[cfg(test)]
mod tests {
    use crate::main;
    use crate::web::PROTOCOL;
    use crate::weblisp::assert_str;
    use crate::weblisp::make_request;
    use crate::weblisp::make_response;
    use crate::weblisp::test_skelton;

    use std::env;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_case_00() {
        use std::path::Path;
        let path = match env::current_dir() {
            Ok(p) => p,
            Err(_) => panic!("current_dir() panic!!"),
        };
        if !path.ends_with("samples") {
            let root = Path::new("samples");
            if let Err(e) = env::set_current_dir(&root) {
                eprintln!("test_case_00 fault: {:?} {:?}", e, path);
            }
        }
        thread::sleep(Duration::from_millis(10));
        thread::spawn(|| main());
    }
    #[test]
    fn test_case_01_index() {
        let r = make_request!("GET", "/");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();
        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/html", iter.next());
        assert_str!("Content-length: 63", iter.next());
        iter.next();
        assert_str!(
            "<html><head><title>test</title></head><body>TEST</body></html>",
            iter.next()
        );
    }
    #[test]
    fn test_case_02_cgi() {
        let r = make_request!("GET", "/examples/index.cgi?hogehoge=hoge");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
    }
    #[test]
    fn test_case_03_lisp() {
        let r = make_request!("GET", "/lisp?expr=%28define%20%E5%B1%B1%20100%29");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("山", iter.next());
    }
    #[test]
    fn test_case_04_get_lispfile() {
        let r = make_request!("GET", "/test.scm");
        let s = vec![
            r.as_str(),
            "HTTP/1.1",
            "User-Agent: rust",
            "Host: 127.0.0.1:9000",
            "",
        ];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();
        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        if let Some(e) = iter.next() {
            assert_str!("Set-Cookie: RUST-ELISP-SID=", Some(&e[0..27].into()))
        }
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 18", iter.next());
        iter.next();
        assert_str!("\"Hello,World rust\"", iter.next());
    }
    #[test]
    fn test_case_91_stop() {
        let t = thread::spawn(|| {
            let r = make_request!("GET", "/lisp?expr=%28let%20loop%20%28%28i%200%29%29%20%28if%20%28%3C%3D%20100000000%20i%29%20i%20%28loop%20%28%2B%20i%201%29%29%29%29");
            let s = vec![r.as_str()];
            test_skelton(&s);
        });
        let r = make_request!("GET", "/lisp?expr=%28force-stop%29");
        let s = vec![r.as_str()];
        test_skelton(&s);

        if let Err(e) = t.join() {
            eprintln!("test_case_91 fault: {:?}", e);
        }
        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("nil", iter.next());
    }
}
