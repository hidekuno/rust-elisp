/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
   ex) curl -v 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define #\a 100)'
*/
extern crate elisp;
use elisp::lisp;

use chrono::Utc;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;

const PROTOCOL: &'static str = "HTTP/1.1";
const CRLF: &'static str = "\r\n";

const RESPONSE_200: &'static str = "200 OK";
const RESPONSE_404: &'static str = "404 NOT FOUND";
const RESPONSE_500: &'static str = "500 Internal Server Error";

struct MimeType(&'static str, &'static str);
const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
const MIME_HTML: MimeType = MimeType("html", "text/html");

macro_rules! http_output {
    ($s: expr, $v: expr) => {
        $s.write($v.as_bytes()).unwrap();
        $s.write(CRLF.as_bytes()).unwrap();
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
    stream.read(&mut buffer).unwrap();

    let (status_line, contents, mime) = dispatch(&buffer, env);
    http_output!(stream, format!("{} {}", PROTOCOL, status_line));

    write_response_header(&mut stream, mime);
    stream.write(CRLF.as_bytes()).unwrap();
    stream.write(contents.as_bytes()).unwrap();
    stream.flush().unwrap();
}
fn dispatch(buffer: &[u8], mut env: lisp::Environment) -> (&'static str, String, &'static str) {
    let r = parse_request(buffer);
    let lisp = "/lisp?expr=";

    return if r.resource == "/" {
        textfile("index.html", MIME_HTML.0)
    } else if r.resource.starts_with(lisp) {
        let (_, expr) = r.resource.split_at(lisp.len());

        let mut result = match lisp::do_core_logic(&expr.to_string(), &mut env) {
            Ok(r) => r.value_string(),
            Err(e) => e.get_code(),
        };
        result.push_str("\n");
        (RESPONSE_200, result, MIME_PLAIN.1)
    } else if r
        .resource
        .ends_with((".".to_owned() + MIME_HTML.0).as_str())
    {
        textfile(r.get_resource(), MIME_HTML.0)
    } else {
        textfile(r.get_resource(), MIME_HTML.0)
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
fn textfile<'a>(filename: &str, mime: &'a str) -> (&'a str, String, &'a str) {
    let mut path = match env::current_dir() {
        Ok(f) => f,
        Err(_) => {
            return (
                RESPONSE_500,
                String::from("Internal Server Error\n"),
                MIME_PLAIN.1,
            )
        }
    };
    for s in filename.split('/') {
        if s == "" {
            continue;
        }
        path.push(s);
    }
    if false == path.as_path().exists() {
        return (RESPONSE_404, String::from("Not Found\r\n"), MIME_PLAIN.1);
    }
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return (
                RESPONSE_500,
                String::from("Internal Server Error\n"),
                MIME_PLAIN.1,
            )
        }
    };
    let meta = match file.metadata() {
        Ok(meta) => meta,
        Err(_) => {
            return (
                RESPONSE_500,
                String::from("Internal Server Error\n"),
                MIME_PLAIN.1,
            )
        }
    };
    if true == meta.is_dir() {
        return textfile(&(filename.to_owned() + "/index.html"), mime);
    }

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    return (RESPONSE_200, contents, mime);
}
fn write_response_header(stream: &mut TcpStream, mime: &str) {
    let date = Utc::now()
        .format("Date: %a, %d %h %Y %H:%M:%S GMT")
        .to_string();
    http_output!(stream, date);
    http_output!(stream, format!("Content-type: {}", mime));
    let header: [&'static str; 2] = ["Server: Rust eLisp", "Connection: keep-alive"];
    for h in &header {
        http_output!(stream, h);
    }
}
