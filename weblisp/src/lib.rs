/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.
   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   ex) cargo test --lib -- --test-threads=1
   ex) cargo test --lib -- --test-threads=1 --nocapture

   hidekuno@gmail.com
*/
pub mod buildin;
pub mod concurrency;
pub mod config;
pub mod epoll;
pub mod lisp;
pub mod server;
pub mod web;

#[cfg(test)]
mod tests {
    use std::env;
    use std::error::Error;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    use crate::config;
    use crate::server::run_web_limit_service;
    use crate::web::CRLF;
    use crate::web::PROTOCOL;
    use config::parse_arg;
    use config::Config;
    use config::BIND_ADDRESS;

    const TEST_COUNT: usize = 23;

    macro_rules! make_request {
        ($method: expr, $resource: expr) => {
            format!("{} {} {}", $method, $resource, PROTOCOL)
        };
    }
    macro_rules! make_response {
        ($status: expr, $message: expr) => {
            format!("{} {} {}", PROTOCOL, $status, $message)
        };
    }
    macro_rules! assert_str {
        ($a: expr,
         $b: expr) => {
            assert!(Some(&String::from($a)) == $b)
        };
    }
    fn make_config(count: usize) -> Config {
        parse_arg(&["--limit".to_string(), "-c".to_string(), count.to_string()]).unwrap()
    }
    fn web_test_client(msg: &[&str], vec: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
        let requst = msg.join(CRLF);

        let mut stream = TcpStream::connect(BIND_ADDRESS)?;
        stream.write_all(requst.as_bytes())?;
        stream.flush()?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        let mut v = Vec::new();
        for e in buffer.into_boxed_slice().iter() {
            match e {
                0x00..=0x7F => v.push(*e),
                0xE5 => v.push(*e), //山(0xE5B1B1)
                0xB1 => v.push(*e), //山(0xE5B1B1)
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
        if let Err(e) = web_test_client(msg, &mut vec) {
            eprintln!("test fault: {:?}", e);
        }
        vec
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
                eprintln!("test_case_00 fault: {:?} {:?}", e, path);
            }
        }
        thread::sleep(Duration::from_millis(10));
        thread::spawn(|| {
            if let Err(e) = run_web_limit_service(make_config(TEST_COUNT)) {
                eprintln!("test_case_00 fault: {:?}", e);
            }
        });
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
    fn test_case_02_txt() {
        let r = make_request!("GET", "/index.txt");
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
        assert_str!("TEST", iter.next());
    }
    #[test]
    fn test_case_03_dir() {
        let r = make_request!("GET", "/examples/");
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
        assert_str!("Content-length: 80", iter.next());
        iter.next();
        assert_str!(
            "<html><head><title>default</title></head><body>default index page</body></html>",
            iter.next()
        );
    }
    #[test]
    fn test_case_04_subdir() {
        let r = make_request!("GET", "/examples/hoge.html");
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
            "<html><head><title>hoge</title></head><body>HOGE</body></html>",
            iter.next()
        );
    }
    #[test]
    fn test_case_05_png() {
        let r = make_request!("GET", "/index.png");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: image/png", iter.next());
        assert_str!("Content-length: 5", iter.next());
    }
    #[test]
    fn test_case_06_lisp() {
        let r = make_request!("GET", "/lisp?expr=%28define%20a%20100%29");
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
        assert_str!("Content-length: 3", iter.next());
        iter.next();
        assert_str!("a", iter.next());
    }
    #[test]
    fn test_case_07_lisp() {
        let r = make_request!("GET", "/lisp?expr=%28%2B%20a%2080%29");
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
        assert_str!("180", iter.next());
    }
    #[test]
    fn test_case_08_octet_stream() {
        let r = make_request!("GET", "/index.dat");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
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
        let r = make_request!("GET", "/hoge.html");
        let s = vec![r.as_str()];
        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("404", "Not Found").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
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
        let r = make_request!("PUT", "/index.html");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(
            make_response!("405", "Method Not Allowed").as_str(),
            iter.next()
        );

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
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
    fn test_case_12_lisp() {
        let r = make_request!("GET", "/lisp?expr=%E5%B1%B1");
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
        assert_str!("100", iter.next());
    }
    #[test]
    fn test_case_13_cgi() {
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
    fn test_case_14_cgi_error() {
        let r = make_request!("GET", "/../samples/examples/index.cgi?hogehoge=hoge");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("404", "Not Found").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
    }
    #[test]
    fn test_case_15_cgi_error() {
        let r = make_request!("GET", "/examples/ng.cgi?hogehoge=hoge");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(
            make_response!("500", "Internal Server Error").as_str(),
            iter.next()
        );

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        iter.next();
        iter.next();
        assert_str!("Internal Server Error", iter.next());
    }
    #[test]
    fn test_case_15_1_cgi_error() {
        let r = make_request!("GET", "/examples/ng2.cgi");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(
            make_response!("500", "Internal Server Error").as_str(),
            iter.next()
        );

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        iter.next();
        iter.next();
        assert_str!("Internal Server Error", iter.next());
    }
    #[test]
    fn test_case_16_lisp_error() {
        let r = make_request!("GET", "/lisp");
        let s = vec![r.as_str()];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("400", "Bad Request").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        iter.next();
        iter.next();
        assert_str!("Bad Request", iter.next());
    }
    #[test]
    fn test_case_17_post_cgi() {
        let r = make_request!("POST", "/examples/index.cgi");
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
    fn test_case_18_post_cgi() {
        let r = make_request!("POST", "/examples/index.cgi");
        let s = vec![r.as_str(), "User-Agent: rust", "Host: 127.0.0.1:9000"];

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
    fn test_case_19_post_cgi() {
        let r = make_request!("POST", "/examples/post.cgi");
        let s = vec![
            r.as_str(),
            "User-Agent: rust",
            "",
            "expr=%28define%20a%20100%29",
        ];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        iter.next();
        assert_str!("expr=(define a 100)", iter.next());
        assert_str!("expr=(define a 100)", iter.next());
    }
    #[test]
    fn test_case_20_post_lisp() {
        let r = make_request!("POST", "/lisp");

        let s = vec![
            r.as_str(),
            " HTTP/1.1",
            "User-Agent: rust",
            "Host: 127.0.0.1:9000",
            "",
            "expr=%28define%20b%20200%29",
        ];

        let iter = test_skelton(&s);
        let mut iter = iter.iter();

        assert_str!(make_response!("200", "OK").as_str(), iter.next());

        if let Some(e) = iter.next() {
            assert_str!("Date: ", Some(&e[0..6].into()))
        }
        assert_str!("Server: Rust eLisp", iter.next());
        assert_str!("Connection: closed", iter.next());
        assert_str!("Content-type: text/plain", iter.next());
        assert_str!("Content-length: 3", iter.next());
        iter.next();
        assert_str!("b", iter.next());
    }
    #[test]
    fn test_case_21_post_lisp() {
        let r = make_request!("POST", "/lisp");
        let s = vec![
            r.as_str(),
            "User-Agent: rust",
            "Host: 127.0.0.1:9000",
            "",
            "expr=b",
        ];

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
        assert_str!("200", iter.next());
    }
    #[test]
    fn test_case_22_get_lispfile() {
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
    fn test_case_90() {
        thread::sleep(Duration::from_millis(30));
        thread::spawn(|| {
            if let Err(e) = run_web_limit_service(make_config(1024)) {
                eprintln!("test_case_90 fault: {:?}", e);
            }
        });
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
