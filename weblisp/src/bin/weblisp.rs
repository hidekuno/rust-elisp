/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
   ex) cargo test --bin weblisp -- --test-threads=1
*/
extern crate elisp;
extern crate weblisp;

use concurrency::ThreadPool;
use elisp::lisp;
use weblisp::concurrency;
use weblisp::web;

use std::env;
use std::error::Error;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const MAX_TRANSACTION: usize = 1000;
const MAX_CONCURRENCY: usize = 4;
const BIND_ADDRESS: &'static str = "127.0.0.1:9000";

pub fn run_web_service(count: usize) -> Result<(), Box<Error>> {
    let listenner = TcpListener::bind(BIND_ADDRESS)?;
    listenner.set_nonblocking(false)?;

    let pool = ThreadPool::new(MAX_CONCURRENCY);
    let env = lisp::Environment::new();
    for stream in listenner.incoming().take(count) {
        match stream {
            Ok(stream) => {
                let env = env.clone();
                pool.execute(|| {
                    web::handle_connection(stream, env);
                });
            }
            Err(ref e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    eprintln!("take fault: {:?}", e);
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    Ok(())
}
fn main() {
    let args: Vec<String> = env::args().collect();

    let t = if args.len() < 2 {
        MAX_TRANSACTION
    } else {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        } else {
            eprintln!("bad paramemter: {}", args[1]);
            return;
        }
    };
    if let Err(e) = run_web_service(t) {
        eprintln!("main fault: {:?}", e);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COUNT: usize = 15;

    fn web_test_client(msg: &str, vec: &mut Vec<String>) -> Result<(), Box<Error>> {
        use std::io::prelude::*;
        use std::net::TcpStream;

        let mut stream = TcpStream::connect(BIND_ADDRESS)?;
        stream.write(msg.as_bytes())?;
        stream.flush()?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        let mut v = Vec::new();
        for e in &buffer {
            match &e {
                0x00...0x7F => v.push(e.clone()),
                0xE5 => v.push(e.clone()), //山(0xE5B1B1)
                0xB1 => v.push(e.clone()), //山(0xE5B1B1)
                _ => {}
            }
        }
        for l in std::str::from_utf8(&v)?.lines() {
            vec.push(String::from(l));
        }
        Ok(())
    }

    fn test_skelton(msg: &[&str]) -> Vec<String> {
        let mut vec = Vec::new();
        thread::sleep(Duration::from_millis(10));
        for m in msg {
            if let Err(e) = web_test_client(m, &mut vec) {
                eprintln!("test fault: {:?}", e);
            }
        }
        vec
    }
    macro_rules! assert_str {
        ($a: expr,
         $b: expr) => {
            assert!(Some(&String::from($a)) == $b)
        };
    }

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
                eprintln!("test1 fault: {:?} {:?}", e, path);
            }
        }
        thread::sleep(Duration::from_millis(10));
        thread::spawn(|| {
            if let Err(e) = run_web_service(TEST_COUNT) {
                eprintln!("test2 fault: {:?}", e);
            }
        });
    }
    #[test]
    fn test_case_01_index() {
        let s = vec!["GET / HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();
        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
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
    fn test_case_02_txt() {
        let s = vec!["GET /index.txt HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("TEST", iter.next());
    }
    #[test]
    fn test_case_03_dir() {
        let s = vec!["GET /examples/ HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/html", iter.next());
        assert_str!("Content-length: 80", iter.next());
        iter.next();
        assert_str!(
            "<html><head><title>default</title></head><body>default index page</body></html>",
            iter.next()
        );
    }
    #[test]
    fn test_case_04_subdir() {
        let s = vec!["GET /examples/hoge.html HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/html", iter.next());
        assert_str!("Content-length: 63", iter.next());
        iter.next();
        assert_str!(
            "<html><head><title>hoge</title></head><body>HOGE</body></html>",
            iter.next()
        );
    }
    #[test]
    fn test_case_05_png() {
        let s = vec!["GET /index.png HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: image/png", iter.next());
        assert_str!("Content-length: 5", iter.next());
    }
    #[test]
    fn test_case_06_lisp() {
        let s = vec!["GET /lisp?expr=%28define%20a%20100%29 HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 3", iter.next());
        iter.next();
        assert_str!("a", iter.next());
    }
    #[test]
    fn test_case_07_lisp() {
        let s = vec!["GET /lisp?expr=%28%2B%20a%2080%29 HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("180", iter.next());
    }
    #[test]
    fn test_case_08_octet_stream() {
        let s = vec!["GET /index.dat HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: application/octet-stream", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("TEST", iter.next());
    }
    #[test]
    fn test_case_09_404() {
        let s = vec!["GET /hoge.html HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 404 Not Found", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 11", iter.next());
        iter.next();
        assert_str!("Not Found", iter.next());
    }
    #[test]
    fn test_case_10_405() {
        let s = vec!["POST /index.html HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 405 Method Not Allowed", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 20", iter.next());
        iter.next();
        assert_str!("Method Not Allowed", iter.next());
    }
    #[test]
    fn test_case_11_lisp() {
        let s = vec!["GET /lisp?expr=%28define%20%E5%B1%B1%20100%29 HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("山", iter.next());
    }
    #[test]
    fn test_case_12_lisp() {
        let s = vec!["GET /lisp?expr=%E5%B1%B1 HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 5", iter.next());
        iter.next();
        assert_str!("100", iter.next());
    }
    #[test]
    fn test_case_13_cgi() {
        let s = vec!["GET /examples/index.cgi?hogehoge=hoge HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 200 OK", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
    }
    #[test]
    fn test_case_14_cgi_error() {
        let s = vec!["GET /../samples/examples/index.cgi?hogehoge=hoge HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 404 Not Found", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
    }
    #[test]
    fn test_case_15_cgi_error() {
        let s = vec!["GET /examples/ng.cgi?hogehoge=hoge HTTP/1.1"];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!("HTTP/1.1 500 Internal Server Error", iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&String::from(&e[0..6])))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        iter.next();
        iter.next();
        assert_str!("Internal Server Error", iter.next());
    }
}
