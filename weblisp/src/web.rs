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
const RESPONSE_404: &'static str = "404 NOT FOUND";
const RESPONSE_500: &'static str = "500 Internal Server Error";

struct MimeType(&'static str, &'static str);
const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
const MIME_HTML: MimeType = MimeType("html", "text/html");
const MIME_PNG: MimeType = MimeType("png", "image/png");
const DEFALUT_MIME: &'static str = "application/octet-stream";

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
            Contents::String(String::from("Not Found") + CRLF),
            MIME_PLAIN.1,
        )
    };
}
macro_rules! error_500 {
    () => {
        (
            RESPONSE_500,
            Contents::String(String::from("Internal Server Error") + CRLF),
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
    pub fn get_metod(&self) -> &String {
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
                Err(e) => println!("{:?}", e),
            },
            Contents::File(v) => match io::copy(v, stream) {
                Ok(_) => {}
                Err(e) => println!("{:?}", e),
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
            println!("recv fault: {:?}", e);
            return;
        }
    }
    match core_proc(stream, env, &buffer) {
        Ok(_) => {}
        Err(ref e) => println!("{:?}", e),
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
    let lisp = "/lisp?expr=";

    return if r.resource == "/" {
        static_contents("index.html")
    } else if r.resource.starts_with(lisp) {
        let (_, expr) = r.resource.split_at(lisp.len());

        let mut result = match lisp::do_core_logic(&expr.to_string(), &mut env) {
            Ok(r) => r.value_string(),
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
    let mut r = s.to_string();

    for i in 0x20 as u8..0x30 {
        r = r.replace(
            format!("%{:X}", i).as_str(),
            std::str::from_utf8(&[i]).unwrap(),
        );
    }
    for i in 0x5B as u8..0x60 {
        r = r.replace(
            format!("%{:X}", i).as_str(),
            std::str::from_utf8(&[i]).unwrap(),
        );
    }
    for i in 0x7B as u8..0x7E {
        r = r.replace(
            format!("%{:X}", i).as_str(),
            std::str::from_utf8(&[i]).unwrap(),
        );
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
    return if filename.ends_with(format!(".{}", MIME_HTML.0).as_str()) {
        MIME_HTML.1
    } else if filename.ends_with(format!(".{}", MIME_PLAIN.0).as_str()) {
        MIME_PLAIN.1
    } else if filename.ends_with(format!(".{}", MIME_PNG.0).as_str()) {
        MIME_PNG.1
    } else {
        DEFALUT_MIME
    };
}
