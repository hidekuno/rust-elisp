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
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

const PROTOCOL: &'static str = "HTTP/1.1";
const CRLF: &'static str = "\r\n";

const RESPONSE_200: &'static str = "200 OK";
const RESPONSE_400: &'static str = "400 Bad Request";
const RESPONSE_404: &'static str = "404 Not Found";
const RESPONSE_405: &'static str = "405 Method Not Allowed";
const RESPONSE_500: &'static str = "500 Internal Server Error";

struct MimeType(&'static str, &'static str);
const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
const MIME_HTML: MimeType = MimeType("html", "text/html");
const MIME_PNG: MimeType = MimeType("png", "image/png");
const DEFALUT_MIME: &'static str = "application/octet-stream";
const CGI_EXT: &'static str = ".cgi";

const LISP: &'static str = "/lisp";
const LISP_PARAMNAME: &'static str = "expr=";

const READ_TIMEOUT: u64 = 60;

#[derive(Debug)]
struct UriParseError {
    pub code: &'static str,
}
impl fmt::Display for UriParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}
impl Error for UriParseError {
    fn description(&self) -> &str {
        self.code
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}
macro_rules! print_error {
    ($f: expr, $e: expr) => {
        error!("{} fault: {:?}", $f, $e)
    };
}
macro_rules! http_write {
    ($s: expr, $v: expr) => {
        $s.write($v.as_bytes())?;
        $s.write(CRLF.as_bytes())?;
    };
}
macro_rules! http_error {
    ($c: expr) => {
        (
            $c,
            Contents::String(String::from(&$c[4..]) + CRLF),
            Some(MIME_PLAIN.1),
        )
    };
}
macro_rules! http_value_error {
    ($c: expr, $err: expr) => {
        (
            $c,
            Contents::String(format!(
                "{}\r\n{}[{}:{}]\r\n",
                String::from(&$c[4..]),
                $err,
                file!(),
                line!()
            )),
            Some(MIME_PLAIN.1),
        )
    };
}
pub struct Request {
    method: String,
    resource: String,
    protocol: String,
    parameter: String,
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
    pub fn get_parameter(&self) -> &String {
        &self.parameter
    }
}
pub struct WebFile {
    file: File,
    length: u64,
}
enum Contents {
    String(String),
    File(WebFile),
    Cgi(Child),
}
impl Contents {
    fn http_write(&mut self, stream: &mut TcpStream) {
        match self {
            Contents::String(v) => {
                if let Err(e) = stream.write(v.as_bytes()) {
                    print_error!("write", e);
                }
            }
            Contents::File(v) => {
                if let Err(e) = io::copy(&mut v.file, stream) {
                    print_error!("copy", e);
                }
            }
            Contents::Cgi(cgi) => {
                let out = match cgi.stdout.as_mut() {
                    Some(out) => out,
                    None => {
                        error!("as_mut err");
                        return;
                    }
                };
                if let Err(e) = io::copy(out, stream) {
                    print_error!("copy", e);
                }
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Contents::String(v) => v.len(),
            Contents::File(v) => v.length as usize,
            // unknown size
            Contents::Cgi(_) => 0,
        }
    }
}
pub fn handle_connection(mut stream: TcpStream, env: lisp::Environment) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))
        .unwrap();

    let mut buffer = [0; 1024];

    // read() is Not Good.(because it's not detected EOF)
    // I try read_to_end() and read_exact(), But it was NG
    if let Err(e) = stream.read(&mut buffer) {
        print_error!("read", e);
        return;
    }

    if let Err(e) = core_proc(stream, env, &buffer) {
        print_error!("core proc", e);
    }
}
fn core_proc(
    mut stream: TcpStream,
    env: lisp::Environment,
    buffer: &[u8],
) -> Result<(), Box<Error>> {
    let (status_line, mut contents, mime) = dispatch(&buffer, env);
    info!("{}", status_line);
    http_write!(stream, format!("{} {}", PROTOCOL, status_line));

    let header: [&str; 3] = [
        &Utc::now()
            .format("Date: %a, %d %h %Y %H:%M:%S GMT")
            .to_string()
            .into_boxed_str(),
        "Server: Rust eLisp",
        "Connection: closed",
    ];
    for h in &header {
        http_write!(stream, h);
    }

    if let Some(v) = mime {
        http_write!(stream, format!("Content-type: {}", v));
        http_write!(stream, format!("Content-length: {}", contents.len()));
        stream.write(CRLF.as_bytes())?;
    }
    contents.http_write(&mut stream);
    stream.flush()?;
    Ok(())
}
fn dispatch(
    buffer: &[u8],
    env: lisp::Environment,
) -> (&'static str, Contents, Option<&'static str>) {
    let r = match parse_request(buffer) {
        Ok(r) => r,
        Err(e) => return http_value_error!(RESPONSE_400, e),
    };
    if r.get_method() != "GET" {
        return http_error!(RESPONSE_405);
    }
    return if r.get_resource() == "/" {
        static_contents("index.html")
    } else if r.get_resource().starts_with(LISP) {
        let (_, expr) = r.get_parameter().split_at(LISP_PARAMNAME.len());

        let mut result = match lisp::do_core_logic(&expr.to_string(), &env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_msg(),
        };
        result.push_str(CRLF);
        (RESPONSE_200, Contents::String(result), Some(MIME_PLAIN.1))
    } else if r.get_resource().ends_with(CGI_EXT) {
        do_cgi(&r)
    } else {
        static_contents(r.get_resource())
    };
}
fn parse_request(buffer: &[u8]) -> Result<Request, Box<Error>> {
    let mut lines = std::str::from_utf8(buffer)?.lines();
    let mut requst: [&str; 8] = [""; 8];

    if let Some(r) = lines.next() {
        info!("{}", r);
        for (i, s) in r.split_whitespace().into_iter().enumerate() {
            if i >= 3 {
                return Err(Box::new(UriParseError { code: "E2001" }));
            }
            requst[i] = s;
        }
    } else {
        return Err(Box::new(UriParseError { code: "E2001" }));
    }
    let iter = urldecode(requst[1])?;
    let mut iter = iter.split('?').into_iter();

    let url = if let Some(s) = iter.next() {
        s
    } else {
        return Err(Box::new(UriParseError { code: "E2001" }));
    };
    let parameter = if let Some(s) = iter.next() { s } else { "" };

    Ok(Request {
        method: String::from(requst[0]),
        resource: String::from(url),
        protocol: String::from(requst[2]),
        parameter: String::from(parameter),
    })
}
fn urldecode(s: &str) -> Result<String, Box<Error>> {
    enum PercentState {
        Init,
        First,
        Second,
    }
    enum ByteMode {
        ASCII,
        Jpn,
    }
    let mut r = String::new();
    let mut en: [u8; 2] = [0; 2];
    let mut ja: [u8; 3] = [0; 3];
    let mut ja_cnt = 0;
    let mut state = PercentState::Init;
    let mut mode = ByteMode::ASCII;

    for b in s.bytes() {
        let c = b as char;
        match state {
            PercentState::Init => match c {
                '%' => state = PercentState::First,
                _ => r.push(c),
            },
            PercentState::First => {
                en[0] = b;
                state = PercentState::Second;
                mode = match b {
                    0x30...0x37 => ByteMode::ASCII,
                    _ => ByteMode::Jpn,
                }
            }
            PercentState::Second => {
                en[1] = b;
                state = PercentState::Init;
                match mode {
                    ByteMode::ASCII => {
                        let v = u8::from_str_radix(std::str::from_utf8(&en)?, 16)?;
                        r.push(v as char);
                    }
                    ByteMode::Jpn => {
                        ja[ja_cnt] = u8::from_str_radix(std::str::from_utf8(&en)?, 16)?;
                        ja_cnt += 1;

                        // not full support utf8
                        if ja_cnt == 3 {
                            mode = ByteMode::ASCII;
                            ja_cnt = 0;
                            r.push_str(std::str::from_utf8(&ja)?);
                        }
                    }
                }
            }
        }
    }
    Ok(r)
}
fn set_path_security(path: &mut PathBuf, filename: &str) {
    for s in filename.split('/') {
        if s == "" {
            continue;
        }
        if s == "." {
            continue;
        }
        if s == ".." {
            continue;
        }
        path.push(s);
    }
}
fn static_contents<'a>(filename: &str) -> (&'a str, Contents, Option<&'a str>) {
    let mut path = match env::current_dir() {
        Ok(f) => f,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    set_path_security(&mut path, filename);
    if false == path.as_path().exists() {
        return http_error!(RESPONSE_404);
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    let meta = match file.metadata() {
        Ok(meta) => meta,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    if true == meta.is_dir() {
        return static_contents(&(filename.to_owned() + "/index.html"));
    }
    (
        RESPONSE_200,
        Contents::File(WebFile {
            file: file,
            length: meta.len(),
        }),
        Some(get_mime(filename)),
    )
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
fn do_cgi(r: &Request) -> (&'static str, Contents, Option<&'static str>) {
    let mut path = match env::current_dir() {
        Ok(f) => f,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    set_path_security(&mut path, r.get_resource());

    if false == path.as_path().exists() {
        return http_error!(RESPONSE_404);
    }
    let cgi = match Command::new(path)
        .arg(r.get_parameter())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(v) => v,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    (RESPONSE_200, Contents::Cgi(cgi), None)
}
