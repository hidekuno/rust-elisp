/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html
   ex) curl 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'
*/
extern crate elisp;
use elisp::lisp;

use chrono::Utc;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;

const PROTOCOL: &'static str = "HTTP/1.1";
const CRLF: &'static str = "\r\n";

const RESPONSE_200: &'static str = "200 OK";
const RESPONSE_404: &'static str = "404 Not Found";
const RESPONSE_405: &'static str = "405 Method Not Allowed";
const RESPONSE_500: &'static str = "500 Internal Server Error";

struct MimeType(&'static str, &'static str);
const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
const MIME_HTML: MimeType = MimeType("html", "text/html");
const MIME_PNG: MimeType = MimeType("png", "image/png");
const DEFALUT_MIME: &'static str = "application/octet-stream";

const LISP: &'static str = "/lisp?expr=";

macro_rules! print_error {
    ($f: expr, $e: expr) => {
        println!("{} fault: {:?}", $f, $e)
    };
}
macro_rules! http_write {
    ($s: expr, $v: expr) => {
        $s.write($v.as_bytes())?;
        $s.write(CRLF.as_bytes())?;
    };
}
macro_rules! error_404 {
    () => {
        (
            RESPONSE_404,
            Contents::String(String::from(&RESPONSE_404[4..]) + CRLF),
            MIME_PLAIN.1,
        )
    };
}
macro_rules! error_405 {
    () => {
        (
            RESPONSE_405,
            Contents::String(String::from(&RESPONSE_405[4..]) + CRLF),
            MIME_PLAIN.1,
        )
    };
}
macro_rules! error_500 {
    () => {
        (
            RESPONSE_500,
            Contents::String(String::from(&RESPONSE_500[4..]) + CRLF),
            MIME_PLAIN.1,
        )
    };
}
pub struct Request {
    method: String,
    resource: String,
    protocol: String,
}
impl Request {
    pub fn get_method(&self) -> &String {
        &self.method
    }
    pub fn get_resource(&self) -> &String {
        &self.resource
    }
    pub fn get_protocol(&self) -> &String {
        &self.protocol
    }
}
enum Contents {
    String(String),
    File(File),
}
impl Contents {
    fn http_write(&mut self, stream: &mut TcpStream) {
        match self {
            Contents::String(v) => match stream.write(v.as_bytes()) {
                Ok(_) => {}
                Err(e) => print_error!("write", e),
            },
            Contents::File(v) => match io::copy(v, stream) {
                Ok(_) => {}
                Err(e) => print_error!("copy", e),
            },
        }
    }
    fn len(&self) -> usize {
        match self {
            Contents::String(v) => v.len(),
            Contents::File(v) => v.metadata().unwrap().len() as usize,
        }
    }
}
pub fn handle_connection(mut stream: TcpStream, env: lisp::Environment) {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    let mut buffer = [0; 1024];
    // read() is Not Good.(because it's not detected EOF)
    // I try read_to_end() and read_exact(), But it's Ng
    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            print_error!("read", e);
            return;
        }
    }
    match core_proc(stream, env, &buffer) {
        Ok(_) => {}
        Err(e) => print_error!("core proc", e),
    }
}
fn core_proc(
    mut stream: TcpStream,
    env: lisp::Environment,
    buffer: &[u8],
) -> Result<(), Box<std::error::Error>> {
    let (status_line, mut contents, mime) = dispatch(&buffer, env);
    println!("{}", status_line);
    http_write!(stream, format!("{} {}", PROTOCOL, status_line));

    let date = Utc::now()
        .format("Date: %a, %d %h %Y %H:%M:%S GMT")
        .to_string();
    http_write!(stream, date);
    http_write!(stream, format!("Content-type: {}", mime));
    http_write!(stream, format!("Content-length: {}", contents.len()));
    let header: [&'static str; 2] = ["Server: Rust eLisp", "Connection: closed"];
    for h in &header {
        http_write!(stream, h);
    }

    stream.write(CRLF.as_bytes())?;
    contents.http_write(&mut stream);
    stream.flush()?;
    Ok(())
}
fn dispatch(buffer: &[u8], mut env: lisp::Environment) -> (&'static str, Contents, &'static str) {
    let r = parse_request(buffer);
    if r.get_method() != "GET" {
        return error_405!();
    }
    return if r.get_resource() == "/" {
        static_contents("index.html")
    } else if r.get_resource().starts_with(LISP) {
        let (_, expr) = r.get_resource().split_at(LISP.len());

        let mut result = match lisp::do_core_logic(&expr.to_string(), &mut env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_code(),
        };
        result.push_str(CRLF);
        (RESPONSE_200, Contents::String(result), MIME_PLAIN.1)
    } else {
        static_contents(r.get_resource())
    };
}
fn parse_request(buffer: &[u8]) -> Request {
    let mut lines = std::str::from_utf8(buffer).unwrap().lines();
    let mut requst: [&str; 128] = [""; 128];

    if let Some(r) = lines.next() {
        println!("{}", r);
        for (i, s) in r.split_whitespace().into_iter().enumerate() {
            requst[i] = s;
        }
    }
    Request {
        method: String::from(requst[0]),
        resource: urldecode(requst[1]),
        protocol: String::from(requst[2]),
    }
}
fn urldecode(s: &str) -> String {
    enum PercentState {
        Init,
        First,
        Second,
    }
    let mut r = String::new();
    let mut e: [u8; 2] = [0; 2];
    let mut state = PercentState::Init;

    for b in s.bytes() {
        let c = b as char;
        match state {
            PercentState::Init => match c {
                '%' => state = PercentState::First,
                _ => r.push(c),
            },
            PercentState::First => {
                // not support multi byte
                match b {
                    0x30...0x37 => {
                        e[0] = b;
                        state = PercentState::Second;
                    }
                    _ => state = PercentState::Init,
                }
            }
            PercentState::Second => {
                e[1] = b;
                state = PercentState::Init;
                match u8::from_str_radix(std::str::from_utf8(&e).unwrap(), 16) {
                    Ok(v) => r.push(v as char),
                    Err(e) => print_error!("u8::from_str_radix hex", e),
                }
            }
        }
    }
    r
}
fn static_contents<'a>(filename: &str) -> (&'a str, Contents, &'a str) {
    let mut path = match env::current_dir() {
        Ok(f) => f,
        Err(_) => return error_500!(),
    };
    for s in filename.split('/') {
        if s == "" {
            continue;
        }
        path.push(s);
    }
    if false == path.as_path().exists() {
        return error_404!();
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return error_500!(),
    };
    let meta = match file.metadata() {
        Ok(meta) => meta,
        Err(_) => return error_500!(),
    };
    if true == meta.is_dir() {
        return static_contents(&(filename.to_owned() + "/index.html"));
    }
    (RESPONSE_200, Contents::File(file), get_mime(filename))
}
fn get_mime(filename: &str) -> &'static str {
    let ext = match filename.rfind('.') {
        Some(i) => &filename[i + 1..],
        None => "",
    };
    return if ext == MIME_HTML.0 {
        MIME_HTML.1
    } else if ext == MIME_PLAIN.0 {
        MIME_PLAIN.1
    } else if ext == MIME_PNG.0 {
        MIME_PNG.1
    } else {
        DEFALUT_MIME
    };
}
