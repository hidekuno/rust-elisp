extern crate elisp;
use elisp::lisp;

use chrono::Utc;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;

const PROTOCOL: &'static str = "HTTP/1.1";
const CRLF: &'static str = "\r\n";
const CR: u8 = 13;
const LF: u8 = 10;

const RESPONSE_200: &'static str = "200 OK";
const RESPONSE_404: &'static str = "404 NOT FOUND";
const RESPONSE_500: &'static str = "500 Internal Server Error";

const MIME_PLAIN: &'static str = "text/plain";
const MIME_HTML: &'static str = "text/html";

macro_rules! http_output {
    ($s: expr, $v: expr) => {
        $s.write($v.as_bytes()).unwrap();
        $s.write(CRLF.as_bytes()).unwrap();
    };
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
    fn htmlfile(filename: &str) -> String {
        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        return contents;
    }
    let get = b"GET / ";
    let lisp = b"GET /lisp?expr=";

    return if buffer.starts_with(get) {
        (RESPONSE_200, htmlfile("hello.html"), MIME_HTML)
    } else if buffer.starts_with(lisp) {
        let mut expr = String::new();
        for i in &buffer[lisp.len()..buffer.len()] {
            if *i == CR || *i == LF {
                break;
            }
            expr.push(*i as char);
        }
        if let Some(n) = expr.rfind(PROTOCOL) {
            let (expr, _) = expr.split_at(n + 1);
            let mut result = match lisp::do_core_logic(&expr.to_string(), &mut env) {
                Ok(r) => r.value_string(),
                Err(e) => e.get_code(),
            };
            result.push_str("\n");
            (RESPONSE_200, result, MIME_PLAIN)
        } else {
            (
                RESPONSE_500,
                String::from("Internal Server Error\n"),
                MIME_PLAIN,
            )
        }
    } else {
        (RESPONSE_404, htmlfile("404.html"), MIME_HTML)
    };
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
