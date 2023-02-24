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
pub const RESPONSE_301: Response = Response(301, "Moved Permanently");
pub const RESPONSE_302: Response = Response(302, "Found");
pub const RESPONSE_400: Response = Response(400, "Bad Request");
pub const RESPONSE_404: Response = Response(404, "Not Found");
pub const RESPONSE_405: Response = Response(405, "Method Not Allowed");
pub const RESPONSE_500: Response = Response(500, "Internal Server Error");

pub struct MimeType(pub &'static str, pub &'static str);
pub const MIME_PLAIN: MimeType = MimeType("txt", "text/plain");
pub const MIME_HTML: MimeType = MimeType("html", "text/html");
pub const MIME_XML: MimeType = MimeType("xml", "text/xml");
pub const MIME_CSS: MimeType = MimeType("css", "text/css");
pub const MIME_JS: MimeType = MimeType("js", "text/javascript");
pub const MIME_PNG: MimeType = MimeType("png", "image/png");
pub const MIME_JPG: MimeType = MimeType("jpg", "image/jpeg");
pub const MIME_GIF: MimeType = MimeType("gif", "image/gif");
pub const MIME_PDF: MimeType = MimeType("pdf", "application/pdf");
pub const MIME_JSON: MimeType = MimeType("json", "application/json");
pub const MIME_WASM: MimeType = MimeType("wasm", "application/wasm");
pub const DEFALUT_MIME: &str = "application/octet-stream";

static MIME_TYPES: &[MimeType] = &[
    MIME_PLAIN, MIME_HTML, MIME_XML, MIME_CSS, MIME_JS, MIME_PNG, MIME_JPG, MIME_GIF, MIME_PDF,
    MIME_JSON, MIME_WASM,
];

pub const SESSION_ID: &str = "RUST-ELISP-SID";
pub const LISP_EXT: &str = ".scm";
pub type WebResult = (Response, Contents, String, Option<String>);

const CGI_EXT: &str = ".cgi";
const LISP: &str = "/lisp";

#[derive(Debug)]
pub enum WebError {
    UriParse(u32),
    UTF8(u32, String),
}
impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebError::UriParse(n) => write!(f, "Uri Parse Error {}", n),
            WebError::UTF8(n, s) => write!(f, "UTF8 Parse Error {} {}", n, s),
        }
    }
}
impl Error for WebError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WebError::UriParse(_) => None,
            WebError::UTF8(_, _) => None,
        }
    }
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
    pub file: File,
    pub length: u64,
}
pub enum Contents {
    String(String),
    File(WebFile),
    Cgi(BufReader<ChildStdout>),
}
impl Contents {
    fn http_write(&mut self, stream: &mut dyn std::io::Write) {
        match self {
            Contents::String(v) => {
                if let Err(e) = stream.write(v.as_bytes()) {
                    error!("write fault: {:?}", e);
                }
            }
            Contents::File(v) => {
                if let Err(e) = io::copy(&mut v.file, stream) {
                    error!("copy fault: {:?}", e);
                }
            }
            Contents::Cgi(v) => {
                if let Err(e) = io::copy(v, stream) {
                    error!("copy fault: {:?}", e);
                }
            }
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Contents::String(v) => v.len(),
            Contents::File(v) => v.length as usize,

            // unknown size
            Contents::Cgi(_) => 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
#[macro_export]
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
pub fn entry_proc<T>(
    stream: &mut T,
    env: lisp::Environment,
    buffer: &[u8],
    id: usize,
) -> Result<(), Box<dyn Error>>
where
    T: std::io::Write,
{
    let r = parse_request(buffer);
    let (status, mut contents, mime, extended) = match &r {
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
    if status.0 == 301 || status.0 == 302 {
        if let Some(location) = extended {
            http_write!(stream, format!("Location: {}", location));
        }
        return Ok(());
    }
    if let Some(cookie) = extended {
        http_write!(stream, make_cookie_header(cookie));
    }

    http_write!(stream, format!("Content-type: {}", mime));
    if !contents.is_empty() {
        http_write!(stream, format!("Content-length: {}", contents.len()));
    }

    let head = is_head_method!(r);
    if !head {
        stream.write_all(CRLF.as_bytes())?;
        contents.http_write(stream);
    }
    stream.flush()?;
    Ok(())
}
pub fn parse_request(buffer: &[u8]) -> Result<Request, Box<WebError>> {
    let mut lines = match std::str::from_utf8(buffer) {
        Ok(b) => b.lines(),
        Err(e) => return Err(Box::new(WebError::UTF8(line!(), e.to_string()))),
    };
    let mut requst: [&str; 8] = [""; 8];

    if let Some(r) = lines.next() {
        info!("{}", r);
        for (i, s) in r.split_whitespace().into_iter().enumerate() {
            if i >= 3 {
                return Err(Box::new(WebError::UriParse(line!())));
            }
            requst[i] = s;
        }
    } else {
        return Err(Box::new(WebError::UriParse(line!())));
    }
    let method = Method::create(requst[0]);
    let iter = urldecode(requst[1])?;
    let mut iter = iter.split('?');

    let url = if let Some(s) = iter.next() {
        s
    } else {
        return Err(Box::new(WebError::UriParse(line!())));
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
fn urldecode(s: &str) -> Result<String, Box<WebError>> {
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
                        let v = match u8::from_str_radix(
                            match std::str::from_utf8(&en) {
                                Ok(s) => s,
                                Err(e) => {
                                    return Err(Box::new(WebError::UTF8(line!(), e.to_string())))
                                }
                            },
                            16,
                        ) {
                            Ok(v) => v,
                            Err(e) => return Err(Box::new(WebError::UTF8(line!(), e.to_string()))),
                        };
                        r.push(v as char);
                    }
                    ByteMode::Jpn => {
                        ja[ja_cnt] = match u8::from_str_radix(
                            match std::str::from_utf8(&en) {
                                Ok(s) => s,
                                Err(e) => {
                                    return Err(Box::new(WebError::UTF8(line!(), e.to_string())))
                                }
                            },
                            16,
                        ) {
                            Ok(v) => v,
                            Err(e) => return Err(Box::new(WebError::UTF8(line!(), e.to_string()))),
                        };
                        ja_cnt += 1;

                        // not full support utf8
                        if ja_cnt == 3 {
                            mode = ByteMode::Ascii;
                            ja_cnt = 0;
                            r.push_str(match std::str::from_utf8(&ja) {
                                Ok(s) => s,
                                Err(e) => {
                                    return Err(Box::new(WebError::UTF8(line!(), e.to_string())))
                                }
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(r)
}
pub fn dispatch(r: &Request, env: lisp::Environment, id: usize) -> WebResult {
    if r.get_method().is_none() {
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
    for mime in MIME_TYPES {
        if ext == mime.0 {
            return mime.1;
        }
    }
    DEFALUT_MIME
}
fn do_cgi(r: &Request) -> WebResult {
    let path = make_path!(r.get_resource());
    let mut cgi = match Command::new(path)
        .env("QUERY_STRING", r.get_parameter())
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
    let mut content_type: Option<String> = None;
    let mut status: Option<String> = None;
    let mut location: Option<String> = None;
    for i in 1..=4 {
        let mut line = String::new();
        if br.read_line(&mut line).is_err() {
            return http_error!(RESPONSE_500);
        }
        if line.trim() == "" {
            break;
        }
        if let Some(end) = line.strip_prefix("Content-Type:") {
            content_type = Some(end.trim().to_string())
        } else if let Some(end) = line.strip_prefix("Status:") {
            status = Some(end.trim().to_string())
        } else if let Some(end) = line.strip_prefix("Location:") {
            location = Some(end.trim().to_string())
        } else {
            return http_error!(RESPONSE_500);
        }
        if i == 4 {
            // Content-Type: a,Content-Type: b ...
            return http_error!(RESPONSE_500);
        }
    }
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
    if location.is_none() {
        match response.0 {
            301 | 302 => return http_error!(RESPONSE_500),
            _ => {}
        }
    }
    (response, Contents::Cgi(br), mime, location)
}
pub fn get_status(status: i64) -> Response {
    match status as u32 {
        200 => RESPONSE_200,
        301 => RESPONSE_301,
        302 => RESPONSE_302,
        400 => RESPONSE_400,
        404 => RESPONSE_404,
        405 => RESPONSE_405,
        500 => RESPONSE_500,
        _ => RESPONSE_500,
    }
}
pub fn make_cookie_header(cookie: String) -> String {
    let expire = Utc::now() + Duration::days(365);

    format!(
        "Set-Cookie: {}={};expires={};",
        SESSION_ID,
        cookie,
        expire
            .format("%a, %d %h %Y %H:%M:%S GMT")
            .to_string()
            .into_boxed_str()
    )
}
