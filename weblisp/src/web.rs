/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.
   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   ex) curl 'http://127.0.0.1:9000/lisp' --get --data-urlencode 'expr=(define a 100)'

   ex) curl -v -c /tmp/cookie.txt http://localhost:9000/samples/test.scm
       curl -v -b /tmp/cookie.txt http://localhost:9000/samples/test.scm

   hidekuno@gmail.com
*/
extern crate elisp;
use elisp::lisp;

use chrono::Duration;
use chrono::Utc;
use mio::net::TcpStream;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{ChildStdout, Command, Stdio};

#[allow(unused_imports)]
use log::{debug, error, info, warn};

pub const PROTOCOL: &str = "HTTP/1.1";
pub const CRLF: &str = "\r\n";

pub struct Response(pub u32, pub &'static str);
pub const RESPONSE_200: Response = Response(200, "OK");
pub const RESPONSE_400: Response = Response(400, "Bad Request");
pub const RESPONSE_404: Response = Response(404, "Not Found");
pub const RESPONSE_405: Response = Response(405, "Method Not Allowed");
pub const RESPONSE_500: Response = Response(500, "Internal Server Error");

pub struct MimeType(pub &'static str, pub &'static str);
pub const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
pub const MIME_HTML: MimeType = MimeType("html", "text/html");
pub const MIME_PNG: MimeType = MimeType("png", "image/png");
pub const DEFALUT_MIME: &str = "application/octet-stream";
pub const SESSION_ID: &str = "RUST-ELISP-SID";
pub const LISP_EXT: &str = ".scm";
pub type WebResult = (Response, Contents, String, Option<String>);

const CGI_EXT: &str = ".cgi";
const LISP: &str = "/lisp";

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
    fn cause(&self) -> Option<&dyn Error> {
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
        $s.write_all($v.as_bytes())?;
        $s.write_all(CRLF.as_bytes())?;
    };
}
#[macro_export]
macro_rules! http_error {
    ($c: expr) => {
        (
            $c,
            Contents::String(String::from($c.1) + CRLF),
            MIME_PLAIN.1.to_string(),
            None,
        )
    };
}
#[macro_export]
macro_rules! http_value_error {
    ($c: expr, $err: expr) => {
        (
            $c,
            Contents::String(format!(
                "{}\r\n{}[{}:{}]\r\n",
                String::from($c.1),
                $err,
                file!(),
                line!()
            )),
            MIME_PLAIN.1.to_string(),
            None,
        )
    };
}
#[macro_export]
macro_rules! make_path {
    ($f: expr) => {{
        let mut path = match env::current_dir() {
            Ok(f) => f,
            Err(e) => return http_value_error!(RESPONSE_500, e),
        };
        set_path_security(&mut path, $f);

        if !path.as_path().exists() {
            return http_error!(RESPONSE_404);
        }
        path
    }};
}
#[derive(Clone)]
pub enum Method {
    Get,
    Post,
    Head,
}
impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Head => "HEAD",
        }
    }
    pub fn create(other: &str) -> Option<Method> {
        match other {
            "GET" => Some(Method::Get),
            "POST" => Some(Method::Post),
            "HEAD" => Some(Method::Head),
            _ => None,
        }
    }
}
impl PartialEq<Method> for Method {
    fn eq(&self, other: &Method) -> bool {
        self == other
    }
}
impl PartialEq<str> for Method {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<Method> for str {
    fn eq(&self, other: &Method) -> bool {
        self == other.as_str()
    }
}
pub struct Request {
    method: Option<Method>,
    resource: String,
    protocol: String,
    parameter: String,
    headers: Vec<String>,
    body: String,
}
impl Request {
    pub fn get_method(&self) -> Option<Method> {
        self.method.clone()
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
    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
    pub fn get_body(&self) -> &String {
        &self.body
    }
    fn get_parsed_list(&self, vec_str: Vec<&str>, del: char) -> String {
        let mut param = String::from("(list ");

        for rec in vec_str {
            if let Some(idx) = rec.find(del) {
                let v0 = &rec[0..idx];
                let v1 = &rec[(idx + 1)..].trim_start_matches(' ');
                param.push_str(format!("(cons {:#?} {:#?})", v0, v1).as_str());
            }
        }
        param.push(')');
        debug!("{:#?}", param);
        param
    }
    pub fn get_lisp_param(&self) -> String {
        self.get_parsed_list(self.parameter.split('&').collect(), '=')
    }
    pub fn get_lisp_header(&self) -> String {
        // A little sloppy implements
        self.get_parsed_list(self.headers.iter().map(|x| x.as_str()).collect(), ':')
    }
}
pub struct WebFile {
    file: File,
    length: u64,
}
pub enum Contents {
    String(String),
    File(WebFile),
    Cgi(BufReader<ChildStdout>),
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
            Contents::Cgi(v) => {
                if let Err(e) = io::copy(v, stream) {
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
macro_rules! is_head_method {
    ($result: expr) => {
        if let Ok(ref req) = $result {
            if let Some(Method::Head) = req.get_method() {
                true
            } else {
                false
            }
        } else {
            false
        }
    };
}
pub fn entry_proc(
    mut stream: TcpStream,
    env: lisp::Environment,
    buffer: &[u8],
    id: usize,
) -> Result<(), Box<dyn Error>> {
    let r = parse_request(buffer);
    let (status, mut contents, mime, cookie) = match &r {
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
    if contents.len() != 0 {
        http_write!(stream, format!("Content-length: {}", contents.len()));
    }

    let head = is_head_method!(r);
    if !head {
        stream.write_all(CRLF.as_bytes())?;
        contents.http_write(&mut stream);
    }
    stream.flush()?;
    Ok(())
}
fn parse_request(buffer: &[u8]) -> Result<Request, Box<dyn Error>> {
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
    let method = Method::create(requst[0]);
    let iter = urldecode(requst[1])?;
    let mut iter = iter.split('?');

    let url = if let Some(s) = iter.next() {
        s
    } else {
        return Err(Box::new(UriParseError { code: "E2001" }));
    };
    let mut parameter = if let Some(s) = iter.next() { s } else { "" };
    let mut headers = Vec::new();
    let mut body = String::from("");
    let mut header = true;
    for e in lines {
        if e.is_empty() {
            header = false;
        } else if header {
            headers.push(e.into());
        } else if let Some(Method::Post) = method {
            body = match urldecode(e) {
                Ok(n) => n,
                Err(_) => body,
            };
            parameter = body.as_str();
        }
    }
    Ok(Request {
        method,
        resource: String::from(url),
        protocol: String::from(requst[2]),
        parameter: String::from(parameter),
        headers,
        body,
    })
}
fn urldecode(s: &str) -> Result<String, Box<dyn Error>> {
    enum PercentState {
        Init,
        First,
        Second,
    }
    enum ByteMode {
        Ascii,
        Jpn,
    }
    let mut r = String::new();
    let mut en: [u8; 2] = [0; 2];
    let mut ja: [u8; 3] = [0; 3];
    let mut ja_cnt = 0;
    let mut state = PercentState::Init;
    let mut mode = ByteMode::Ascii;

    for b in s.bytes() {
        if b == 0x00 {
            continue;
        }
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
                    0x30..=0x37 => ByteMode::Ascii,
                    _ => ByteMode::Jpn,
                }
            }
            PercentState::Second => {
                en[1] = b;
                state = PercentState::Init;
                match mode {
                    ByteMode::Ascii => {
                        let v = u8::from_str_radix(std::str::from_utf8(&en)?, 16)?;
                        r.push(v as char);
                    }
                    ByteMode::Jpn => {
                        ja[ja_cnt] = u8::from_str_radix(std::str::from_utf8(&en)?, 16)?;
                        ja_cnt += 1;

                        // not full support utf8
                        if ja_cnt == 3 {
                            mode = ByteMode::Ascii;
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
fn dispatch(r: &Request, env: lisp::Environment, id: usize) -> WebResult {
    if None == r.get_method() {
        return http_error!(RESPONSE_405);
    }
    return if r.get_resource() == "/" {
        static_contents("index.html")
    } else if r.get_resource() == LISP {
        crate::lisp::do_repl(r, env)
    } else if r.get_resource().ends_with(CGI_EXT) {
        do_cgi(r)
    } else if r.get_resource().ends_with(LISP_EXT) {
        crate::lisp::do_scm(r, env, id)
    } else {
        static_contents(r.get_resource())
    };
}
pub fn set_path_security(path: &mut PathBuf, filename: &str) {
    for s in filename.split('/') {
        if s.is_empty() {
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
fn static_contents(filename: &str) -> WebResult {
    let path = make_path!(filename);
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    let meta = match file.metadata() {
        Ok(meta) => meta,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    if meta.is_dir() {
        return static_contents(&(filename.to_owned() + "/index.html"));
    }
    (
        RESPONSE_200,
        Contents::File(WebFile {
            file,
            length: meta.len(),
        }),
        get_mime(filename).to_string(),
        None,
    )
}
pub fn get_mime(filename: &str) -> &'static str {
    let ext = match filename.rfind('.') {
        Some(i) => &filename[i + 1..],
        None => "",
    };
    if ext == MIME_HTML.0 {
        MIME_HTML.1
    } else if ext == MIME_PLAIN.0 {
        MIME_PLAIN.1
    } else if ext == MIME_PNG.0 {
        MIME_PNG.1
    } else {
        DEFALUT_MIME
    }
}
fn do_cgi(r: &Request) -> WebResult {
    let path = make_path!(r.get_resource());
    let mut cgi = match Command::new(path)
        .arg(r.get_parameter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(v) => v,
        Err(e) => return http_value_error!(RESPONSE_500, e),
    };
    let stdin = cgi.stdin.as_mut().unwrap();
    stdin.write_all(r.get_body().as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();

    let out = match cgi.stdout {
        Some(out) => out,
        None => return http_error!(RESPONSE_500),
    };
    let mut br = BufReader::new(out);

    // Max 2 lines
    // It's Support for Content-Type, Status,
    // It's Not support Location, etc..
    let (content_type, status) = {
        let mut content_type: Option<String> = None;
        let mut status: Option<String> = None;

        for _ in 1..=3 {
            let mut guess = String::new();
            if br.read_line(&mut guess).is_err() {
                return http_error!(RESPONSE_500);
            }
            if guess == "" {
                break;
            }
            if guess.starts_with("Content-Type: ") {
                content_type = Some(guess["Content-Type: ".len()..].trim().to_owned())
            }
            if guess.starts_with("Status: ") {
                status = Some(guess["Status: ".len()..].trim().to_owned())
            }
        }
        (content_type, status)
    };

    let mime = match content_type {
        Some(n) => n,
        None => DEFALUT_MIME.to_string(),
    };
    let response = match status {
        Some(n) => match n.parse::<i64>() {
            Ok(n) => get_status(n),
            Err(_) => return http_error!(RESPONSE_500),
        },
        None => RESPONSE_200,
    };
    (response, Contents::Cgi(br), mime, None)
}
pub fn get_status(status: i64) -> Response {
    match status as u32 {
        200 => RESPONSE_200,
        400 => RESPONSE_400,
        404 => RESPONSE_404,
        405 => RESPONSE_405,
        500 => RESPONSE_500,
        _ => RESPONSE_500,
    }
}
