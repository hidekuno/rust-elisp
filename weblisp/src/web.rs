/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
   ex) curl -v 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'
*/
extern crate elisp;
use elisp::lisp;

use chrono::Utc;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::TcpStream;

const PROTOCOL: &'static str = "HTTP/1.1";
const CRLF: &'static str = "\r\n";

const RESPONSE_200: &'static str = "200 OK";
const RESPONSE_404: &'static str = "404 NOT FOUND";
const RESPONSE_500: &'static str = "500 Internal Server Error";

struct MimeType(&'static str, &'static str);
const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
const MIME_HTML: MimeType = MimeType("html", "text/html");
const MIME_PNG: MimeType = MimeType("png", "image/png");

macro_rules! http_output {
    ($s: expr, $v: expr) => {
        $s.write($v.as_bytes())?;
        $s.write(CRLF.as_bytes())?;
    };
}
macro_rules! error_404 {
    () => {
        (
            RESPONSE_404,
            String::from("Not Found\r\n").into_bytes(),
            MIME_PLAIN.1,
        )
    };
}
macro_rules! error_500 {
    () => {
        (
            RESPONSE_500,
            String::from("Internal Server Error\n").into_bytes(),
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
pub fn handle_connection(mut stream: TcpStream, env: lisp::Environment) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    break;
                }
                if n == 0 {
                    match stream.shutdown(Shutdown::Both) {
                        Ok(_) => {}
                        Err(e) => println!("{:?}", e),
                    }
                    return;
                }
            }
            Err(e) => {
                println!("{:?}", e);
                return;
            }
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
    let (status_line, contents, mime) = dispatch(&buffer, env);
    println!("{}", status_line);
    http_output!(stream, format!("{} {}", PROTOCOL, status_line));

    let date = Utc::now()
        .format("Date: %a, %d %h %Y %H:%M:%S GMT")
        .to_string();
    http_output!(stream, date);
    http_output!(stream, format!("Content-type: {}", mime));
    http_output!(stream, format!("Content-length: {}", contents.len()));
    let header: [&'static str; 2] = ["Server: Rust eLisp", "Connection: closed"];
    for h in &header {
        http_output!(stream, h);
    }

    stream.write(CRLF.as_bytes())?;
    stream.write(contents.as_slice())?;
    stream.flush()?;
    Ok(())
}
fn dispatch(buffer: &[u8], mut env: lisp::Environment) -> (&'static str, Vec<u8>, &'static str) {
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
        result.push_str("\n");
        (RESPONSE_200, result.into_bytes(), MIME_PLAIN.1)
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
fn static_contents<'a>(filename: &str) -> (&'a str, Vec<u8>, &'a str) {
    let mut vec = Vec::new();

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
    let mut file = match File::open(path) {
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
    return match file.read_to_end(&mut vec) {
        Ok(_) => (RESPONSE_200, vec, get_mime(filename)),
        Err(_) => error_500!(),
    };
}
fn get_mime(filename: &str) -> &'static str {
    return if filename.ends_with((".".to_owned() + MIME_HTML.0).as_str()) {
        MIME_HTML.1
    } else if filename.ends_with((".".to_owned() + MIME_PLAIN.0).as_str()) {
        MIME_PLAIN.1
    } else if filename.ends_with((".".to_owned() + MIME_PNG.0).as_str()) {
        MIME_PNG.1
    } else {
        MIME_PLAIN.1
    };
}
