/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::string::ToString;
use std::vec::Vec;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[cfg(feature = "signal")]
use super::unix::signal::{catch_sig_intr_status, init_sig_intr};

use crate::number::Number;
use crate::number::Rat;
use crate::syntax::Continuation;

#[cfg(feature = "thread")]
use crate::env_thread::{ExtFunctionRc, FunctionRc, ListRc};
#[cfg(feature = "thread")]
pub type Environment = crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::{ExtFunctionRc, FunctionRc, ListRc};
#[cfg(not(feature = "thread"))]
pub type Environment = crate::env_single::Environment;

use crate::get_ptr;
use crate::mut_list;
use crate::referlence_list;
//========================================================================
#[derive(Clone, Debug)]
pub enum ErrCode {
    E0001,
    E0002,
    E0003,
    E0004,
    E1001,
    E1002,
    E1003,
    E1004,
    E1005,
    E1006,
    E1007,
    E1008,
    E1009,
    E1010,
    E1011,
    E1012,
    E1013,
    E1014,
    E1015,
    E1016,
    E1017,
    E1018,
    E1019,
    E1020,
    E1021,
    E9000,
    E9999,
    CONT,
}
impl ErrCode {
    pub fn as_str(&self) -> &'static str {
        return match self {
            ErrCode::E0001 => "E0001",
            ErrCode::E0002 => "E0002",
            ErrCode::E0003 => "E0003",
            ErrCode::E0004 => "E0004",
            ErrCode::E1001 => "E1001",
            ErrCode::E1002 => "E1002",
            ErrCode::E1003 => "E1003",
            ErrCode::E1004 => "E1004",
            ErrCode::E1005 => "E1005",
            ErrCode::E1006 => "E1006",
            ErrCode::E1007 => "E1007",
            ErrCode::E1008 => "E1008",
            ErrCode::E1009 => "E1009",
            ErrCode::E1010 => "E1010",
            ErrCode::E1011 => "E1011",
            ErrCode::E1012 => "E1012",
            ErrCode::E1013 => "E1013",
            ErrCode::E1014 => "E1014",
            ErrCode::E1015 => "E1015",
            ErrCode::E1016 => "E1016",
            ErrCode::E1017 => "E1017",
            ErrCode::E1018 => "E1018",
            ErrCode::E1019 => "E1019",
            ErrCode::E1020 => "E1020",
            ErrCode::E1021 => "E1021",
            ErrCode::E9000 => "E9000",
            ErrCode::E9999 => "E9999",
            ErrCode::CONT => "CONT",
        };
    }
}
impl PartialEq<ErrCode> for ErrCode {
    fn eq(&self, other: &ErrCode) -> bool {
        self.as_str() == other.as_str()
    }
}
lazy_static! {
    static ref ERRMSG_TBL: HashMap<&'static str, &'static str> = {
        let mut e: HashMap<&'static str, &'static str> = HashMap::new();
        e.insert(ErrCode::E0001.as_str(), "Unexpected EOF while reading");
        e.insert(ErrCode::E0002.as_str(), "Unexpected ')' while reading");
        e.insert(ErrCode::E0003.as_str(), "Extra close parenthesis `)'");
        e.insert(ErrCode::E0004.as_str(), "Charactor syntax error");
        e.insert(ErrCode::E1001.as_str(), "Not Boolean");
        e.insert(ErrCode::E1002.as_str(), "Not Integer");
        e.insert(ErrCode::E1003.as_str(), "Not Number");
        e.insert(ErrCode::E1004.as_str(), "Not Symbol");
        e.insert(ErrCode::E1005.as_str(), "Not List");
        e.insert(ErrCode::E1006.as_str(), "Not Function");
        e.insert(ErrCode::E1007.as_str(), "Not Enough Parameter Counts");
        e.insert(ErrCode::E1008.as_str(), "Undefine variable");
        e.insert(ErrCode::E1009.as_str(), "Not Enough Data Type");
        e.insert(ErrCode::E1010.as_str(), "Not Promise");
        e.insert(ErrCode::E1011.as_str(), "Not Enough List Length");
        e.insert(ErrCode::E1012.as_str(), "Not Cond Gramar");
        e.insert(ErrCode::E1013.as_str(), "Calculate A Division By Zero");
        e.insert(ErrCode::E1014.as_str(), "Not Found Program File");
        e.insert(ErrCode::E1015.as_str(), "Not String");
        e.insert(ErrCode::E1016.as_str(), "Not Program File");
        e.insert(ErrCode::E1017.as_str(), "Not Case Gramar");
        e.insert(ErrCode::E1018.as_str(), "Not Format Gramar");
        e.insert(ErrCode::E1019.as_str(), "Not Char");
        e.insert(ErrCode::E1020.as_str(), "Not Rat");
        e.insert(ErrCode::E1021.as_str(), "Out Of Range");
        e.insert(ErrCode::E9000.as_str(), "Forced stop");
        e.insert(ErrCode::E9999.as_str(), "System Panic");
        e.insert(ErrCode::CONT.as_str(), "Appear Continuation");
        e
    };
}
pub struct Error {
    pub code: ErrCode,
    pub line: u32,
    pub file: &'static str,
    pub value: Option<String>,
    pub exp: Option<Expression>,
}
impl Error {
    pub fn get_code(&self) -> String {
        String::from(self.code.as_str())
    }
    pub fn get_msg(&self) -> String {
        if let Some(s) = &self.value {
            format!(
                "{}: {} ({}:{})",
                ERRMSG_TBL.get(self.code.as_str()).unwrap(),
                s,
                self.file,
                self.line
            )
        } else {
            format!(
                "{} ({}:{})",
                ERRMSG_TBL.get(self.code.as_str()).unwrap(),
                self.file,
                self.line
            )
        }
    }
}
#[macro_export]
macro_rules! create_error {
    ($e: expr) => {
        Error {
            code: $e,
            line: line!(),
            file: file!(),
            value: None,
            exp: None,
        }
    };
}
#[macro_export]
macro_rules! create_error_value {
    ($e: expr, $v: expr) => {
        Error {
            code: $e,
            line: line!(),
            file: file!(),
            value: Some($v.to_string()),
            exp: None,
        }
    };
}
#[macro_export]
macro_rules! create_continuation {
    ($e: expr, $n:expr) => {
        Error {
            code: ErrCode::CONT,
            line: line!(),
            file: file!(),
            value: Some($n.to_string()),
            exp: Some($e),
        }
    };
}
#[macro_export]
macro_rules! print_error {
    ($e: expr) => {
        println!("{}", $e.get_msg())
    };
}
//========================================================================
pub type ResultExpression = Result<Expression, Error>;
pub type BasicBuiltIn = fn(&[Expression], &Environment) -> ResultExpression;
//========================================================================
#[derive(Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    List(ListRc),
    Pair(Box<Expression>, Box<Expression>),
    Symbol(String),
    String(String),
    Function(FunctionRc),
    BuildInFunction(String, BasicBuiltIn),
    BuildInFunctionExt(ExtFunctionRc),
    TailLoop(),
    Nil(),
    TailRecursion(FunctionRc),
    Promise(Box<Expression>, Environment),
    Rational(Rat),
    Continuation(Box<Continuation>),
}
impl Expression {
    pub fn is_list(exp: &Expression) -> bool {
        match exp {
            Expression::List(_) => true,
            _ => false,
        }
    }
    pub fn is_pair(exp: &Expression) -> bool {
        match exp {
            Expression::Pair(_, _) => true,
            _ => false,
        }
    }
    pub fn is_char(exp: &Expression) -> bool {
        match exp {
            Expression::Char(_) => true,
            _ => false,
        }
    }
    pub fn is_string(exp: &Expression) -> bool {
        match exp {
            Expression::String(_) => true,
            _ => false,
        }
    }
    pub fn is_procedure(exp: &Expression) -> bool {
        match exp {
            Expression::Function(_) => true,
            Expression::BuildInFunction(_, _) => true,
            Expression::BuildInFunctionExt(_) => true,
            _ => false,
        }
    }
    pub fn is_integer(exp: &Expression) -> bool {
        match exp {
            Expression::Integer(_) => true,
            _ => false,
        }
    }
    pub fn is_number(exp: &Expression) -> bool {
        match exp {
            Expression::Integer(_) => true,
            Expression::Float(_) => true,
            Expression::Rational(_) => true,
            _ => false,
        }
    }
    pub fn is_symbol(exp: &Expression) -> bool {
        match exp {
            Expression::Symbol(_) => true,
            _ => false,
        }
    }
    pub fn is_boolean(exp: &Expression) -> bool {
        match exp {
            Expression::Boolean(_) => true,
            _ => false,
        }
    }
    pub fn eq(x: &Expression, y: &Expression) -> bool {
        if let (Expression::Integer(a), Expression::Integer(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::Float(a), Expression::Float(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::Rational(a), Expression::Rational(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::String(a), Expression::String(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::Char(a), Expression::Char(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::Boolean(a), Expression::Boolean(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        if let (Expression::Symbol(a), Expression::Symbol(b)) = (x, y) {
            if a == b {
                return true;
            }
        }
        false
    }
    pub fn to_number(x: &Expression) -> Result<Number, Error> {
        match x {
            Expression::Float(v) => Ok(Number::Float(*v)),
            Expression::Integer(v) => Ok(Number::Integer(*v)),
            Expression::Rational(v) => Ok(Number::Rational(*v)),
            _ => return Err(create_error!(ErrCode::E1003)),
        }
    }
    fn list_string(exp: &[Expression]) -> String {
        let mut s = String::from("(");

        let mut c = 1;
        let mut el = false;
        for e in exp {
            if let Expression::List(l) = e {
                let l = &*(referlence_list!(l));
                s.push_str(Expression::list_string(&l[..]).as_str());
                el = true;
            } else {
                if el {
                    s.push_str(" ");
                }
                s.push_str(e.to_string().as_str());
                if c != exp.len() {
                    s.push_str(" ");
                }
                el = false;
            }
            c += 1;
        }
        s.push_str(")");
        return s;
    }
}
impl ToString for Expression {
    fn to_string(&self) -> String {
        return match self {
            Expression::Integer(v) => v.to_string(),
            Expression::Float(v) => v.to_string(),
            Expression::Char(v) => {
                if v.is_control() || v.is_whitespace() {
                    let c: u8 = *v as u8;
                    if c == SPACE.0 {
                        return SPACE.1.to_string();
                    }
                    if c == TAB.0 {
                        return TAB.1.to_string();
                    }
                    if c == NEWLINE.0 {
                        return NEWLINE.1.to_string();
                    }
                    if c == CARRIAGERETRUN.0 {
                        return CARRIAGERETRUN.1.to_string();
                    }
                    return "#\\non-printable-char".to_string();
                } else {
                    return format!("#\\{}", v);
                }
            }
            Expression::Boolean(v) => (if *v { TRUE } else { FALSE }).to_string(),
            Expression::Symbol(v) => v.to_string(),
            Expression::String(v) => format!("\"{}\"", v),
            Expression::List(v) => {
                let l = &*(referlence_list!(v));
                return Expression::list_string(&l[..]);
            }
            Expression::Pair(car, cdr) => format!("({} . {})", car.to_string(), cdr.to_string()),
            Expression::Function(_) => "Function".into(),
            Expression::BuildInFunction(s, _) => format!("<{}> BuildIn Function", s),
            Expression::BuildInFunctionExt(_) => "BuildIn Function Ext".into(),
            Expression::Nil() => "nil".into(),
            Expression::TailLoop() => "tail loop".into(),
            Expression::TailRecursion(_) => "Tail Recursion".into(),
            Expression::Promise(_, _) => "Promise".into(),
            Expression::Rational(v) => v.to_string(),
            Expression::Continuation(_) => "Continuation".into(),
        };
    }
}
#[derive(Clone)]
pub struct Function {
    param: Vec<String>,
    body: Vec<Expression>,
    name: String,
    closure_env: Environment,
    tail_recurcieve: bool,
}
impl Function {
    pub fn new(sexp: &[Expression], name: String, closure_env: Environment) -> Self {
        let mut param: Vec<String> = Vec::new();

        if let Expression::List(l) = &sexp[1] {
            let l = &*(referlence_list!(l));
            for n in l {
                if let Expression::Symbol(s) = n {
                    param.push(s.to_string());
                }
            }
        }
        let mut vec: Vec<Expression> = Vec::new();
        vec.extend_from_slice(&sexp[2..]);
        Function {
            param: param,
            body: vec,
            name: name,
            closure_env: closure_env,
            tail_recurcieve: false,
        }
    }
    pub fn set_param(&self, exp: &[Expression], env: &Environment) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        // param eval
        let mut vec: Vec<Expression> = Vec::new();
        // env set
        for e in &exp[1 as usize..] {
            vec.push(eval(e, env)?);
        }
        for (i, e) in vec.into_iter().enumerate() {
            env.update(&self.param[i], e);
        }
        Ok(Expression::TailLoop())
    }
    pub fn execute(&self, exp: &[Expression], env: &Environment) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        // param eval
        let mut vec: Vec<Expression> = Vec::new();
        for e in &exp[1 as usize..] {
            vec.push(eval(e, env)?);
        }
        // @@@ env.create();
        let env = Environment::with_parent(&self.closure_env);
        for (i, s) in self.param.iter().enumerate() {
            env.regist(s.to_string(), vec[i].clone());
        }
        // execute!
        let mut ret = Expression::Nil();
        for e in &self.body {
            ret = loop {
                match eval(e, &env) {
                    Ok(n) => match n {
                        Expression::TailLoop() => {
                            if self.tail_recurcieve {
                                continue;
                            }
                        }
                        v => break v,
                    },
                    Err(e) => match &e.code {
                        ErrCode::CONT => {
                            let s = if let Some(ref s) = e.value {
                                s.clone()
                            } else {
                                return Err(e);
                            };
                            if self.param.len() == 1 && self.param[0] == s {
                                if let Expression::Continuation(_) = &exp[1] {
                                    break e.exp.unwrap();
                                }
                            }
                            return Err(e);
                        }
                        _ => {
                            return Err(e);
                        }
                    },
                };
            }
        }
        Ok(ret)
    }
    pub fn get_tail_recurcieve(&self) -> bool {
        return self.tail_recurcieve;
    }
    pub fn set_tail_recurcieve(&mut self) {
        if let Some(l) = self.parse_tail_recurcieve(self.body.as_slice()) {
            self.tail_recurcieve = true;

            let mut l = mut_list!(l);
            l[0] = Environment::create_tail_recursion(self.clone());
        }
    }
    fn parse_tail_recurcieve(&self, exp: &[Expression]) -> Option<ListRc> {
        let mut n = 0;
        let mut vec: Option<ListRc> = None;

        for (i, e) in exp.iter().enumerate() {
            if let Expression::List(l) = e {
                let l = &*(referlence_list!(l));
                if 1 >= l.len() {
                    continue;
                }
                if let Expression::BuildInFunction(s, _) = &l[0] {
                    match s.as_str() {
                        "if" | "cond" => {
                            return self.parse_tail_recurcieve(&l[1..]);
                        }
                        "begin" => {
                            return self.parse_tail_recurcieve(&l[1..]);
                        }
                        _ => {}
                    }
                } else if let Expression::Symbol(s) = &l[0] {
                    if *s == self.name {
                        // check tail
                        if (exp.len() - 1) == i {
                            debug!("set tail_recurcieve {}", s);
                            if let Expression::List(ref v) = exp[i] {
                                // check calling times
                                vec = if n == 0 { Some(v.clone()) } else { None };
                            }
                        }
                        n = n + 1;
                    } else if *s == "else" {
                        return self.parse_tail_recurcieve(&l[1..]);
                    }
                }
            }
        }
        return vec;
    }
}
//========================================================================
const PROMPT: &str = "rust.elisp> ";
const QUIT: &str = "(quit)";
const TAIL_OFF: &str = "(tail-recursion-off)";
const TAIL_ON: &str = "(tail-recursion-on)";
const FORCE_STOP: &str = "(force-stop)";

pub struct ControlChar(pub u8, pub &'static str);
pub const SPACE: ControlChar = ControlChar(0x20, "#\\space");
pub const TAB: ControlChar = ControlChar(0x09, "#\\tab");
pub const NEWLINE: ControlChar = ControlChar(0x0A, "#\\newline");
pub const CARRIAGERETRUN: ControlChar = ControlChar(0x0D, "#\\return");

const TRUE: &'static str = "#t";
const FALSE: &'static str = "#f";

const BACKSLASH: u8 = 0x5c;
//========================================================================
pub fn do_interactive() {
    #[cfg(feature = "signal")]
    init_sig_intr();

    let mut stream = BufReader::new(std::io::stdin());
    let env = Environment::new();

    match repl(&mut stream, &env, Some(PROMPT)) {
        Err(e) => println!("{}", e),
        Ok(_) => {}
    }
}
pub fn repl(
    stream: &mut dyn BufRead,
    env: &Environment,
    prompt: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    let mut program: Vec<String> = Vec::new();

    'outer: loop {
        if let Some(p) = prompt {
            print!("{}", p);
            std::io::stdout().flush().unwrap();
        }
        let lisp = loop {
            buffer.clear();
            let n = stream.read_line(&mut buffer)?;
            if n == 0 {
                break 'outer;
            }
            if program.len() == 0 {
                if buffer.trim() == QUIT {
                    println!("Bye");
                    break 'outer;
                } else if buffer.trim() == "" {
                    continue 'outer;
                }
            }
            if buffer.as_bytes()[0] as char == ';' {
                continue;
            }
            program.push(buffer.trim().to_string());
            let lisp = program.join(" ");
            if false == count_parenthesis(&lisp) {
                continue;
            }
            break lisp;
        };
        debug!("{}", program.iter().cloned().collect::<String>());
        match do_core_logic(&lisp, env) {
            Ok(n) => println!("{}", n.to_string()),
            Err(e) => {
                if ErrCode::E9000.as_str() == e.get_code() {
                    env.set_force_stop(false);
                }
                print_error!(e);
            }
        }
        program.clear();
    }
    Ok(())
}
pub fn count_parenthesis(program: &String) -> bool {
    let mut left = 0;
    let mut right = 0;
    let mut search = true;

    for c in program.as_str().chars() {
        if c == '"' && search {
            search = false;
        } else if c == '"' && !search {
            search = true;
        }
        if c == '(' && search {
            left += 1;
        }
        if c == ')' && search {
            right += 1;
        }
    }
    return left <= right;
}
pub fn do_core_logic(program: &String, env: &Environment) -> ResultExpression {
    let mut token = tokenize(program);
    let mut c: i32 = 1;
    let mut ret = Expression::Nil();

    loop {
        let exp = parse(&token, &mut c, env)?;

        match exp.to_string().as_str() {
            TAIL_ON => {
                env.set_tail_recursion(true);
            }
            TAIL_OFF => {
                env.set_tail_recursion(false);
            }
            FORCE_STOP => {
                env.set_force_stop(true);
            }
            _ => {
                env.set_cont(&exp);
                ret = eval(&exp, env)?;
            }
        }
        debug!("{:?} c = {} token = {}", token.to_vec(), c, token.len());
        if c == token.len() as i32 {
            break;
        } else {
            for _ in 0..c as usize {
                token.remove(0);
            }
            c = 1;
        }
    }
    return Ok(ret);
}
struct TokenState {
    tokens: Vec<String>,
    name: String,
    left: i32,
    right: i32,
    string_mode: bool,
    quote_mode: bool,
    idx: usize,
}
impl TokenState {
    fn new() -> Self {
        TokenState {
            tokens: Vec::new(),
            name: String::new(),
            left: 0,
            right: 0,
            string_mode: false,
            quote_mode: false,
            idx: 0,
        }
    }
    fn push(&mut self, s: String) {
        self.tokens.push(s);
    }
    fn push_if_quote(&mut self, s: String) {
        if let Some(last) = self.tokens.last() {
            if self.quote_mode == true && last == "quote" {
                self.tokens.push(s);
                self.tokens.push(")".into());
                self.quote_mode = false;
            } else {
                self.tokens.push(s);
            }
        } else {
            self.tokens.push(s);
        }
    }
    fn set_quote(&mut self) {
        self.left = 0;
        self.right = 0;
        self.quote_mode = true;
        self.tokens.push("(".into());
        self.tokens.push("quote".into());
    }
    fn tokens(self) -> Vec<String> {
        self.tokens
    }
}
pub fn tokenize(program: &String) -> Vec<String> {
    let mut token = TokenState::new();
    let mut from = 0;

    macro_rules! set_token_name {
        ($c: expr) => {
            token.name.push($c);
            if program.len() - $c.len_utf8() == token.idx {
                // ex. <rust-elisp> abc
                token.push_if_quote(token.name.to_string());
            } else {
                // ex. <rust-elisp> abc def ghi
                match program.as_bytes()[token.idx + $c.len_utf8()] as char {
                    ' ' | '\r' | '\n' | '\t' => {
                        token.push_if_quote(token.name.to_string());
                        token.name.clear();
                    }
                    '(' | ')' => {
                        if token.name != "#\\" {
                            token.push_if_quote(token.name.to_string());
                            token.name.clear();
                        }
                    }
                    _ => {}
                }
            }
        };
    }

    //A String is a wrapper over a Vec<u8>.(https://doc.rust-lang.org/book/ch08-02-strings.html)
    for c in program.chars() {
        if token.string_mode {
            if c == '"' {
                // ex. <rust-elisp> "abc \""
                if program.as_bytes()[token.idx - 1] != BACKSLASH {
                    let ls = program.get(from..(token.idx + 1)).unwrap();
                    token.push_if_quote(ls.to_string());
                    token.string_mode = false;
                }
            }
        } else if token.name.starts_with("#\\") == true {
            set_token_name!(c);
        } else {
            match c {
                '\'' => {
                    token.set_quote();
                }
                '"' => {
                    from = token.idx;
                    token.string_mode = true;
                }
                '(' => {
                    token.left += 1;
                    token.push("(".into());
                }
                ')' => {
                    token.right += 1;
                    token.push(")".into());

                    if (token.quote_mode == true) && (token.left == token.right) {
                        token.push(")".into());
                        token.quote_mode = false;
                    }
                }
                ' ' | '\r' | '\n' | '\t' => {}
                _ => {
                    set_token_name!(c);
                }
            }
        }
        token.idx += c.len_utf8();
    }

    // For Occur charactor syntax error ex. <rust-elisp> "abc
    if token.string_mode {
        token.push_if_quote(program.get(from..token.idx).unwrap().to_string());
    }
    debug!("{:?}", token.tokens);
    return token.tokens();
}
pub fn parse(tokens: &Vec<String>, count: &mut i32, env: &Environment) -> ResultExpression {
    if tokens.len() == 0 {
        return Err(create_error!(ErrCode::E0001));
    }

    let token = &tokens[0];
    if "(" == token {
        if tokens.len() <= 1 {
            return Err(create_error!(ErrCode::E0001));
        }
        let mut list: Vec<Expression> = Vec::new();

        *count = 1;
        loop {
            if tokens[*count as usize] == ")" {
                *count += 1;
                break;
            }
            let mut c: i32 = 1;
            let o = parse(&tokens[*count as usize..].to_vec(), &mut c, env)?;
            list.push(o);

            *count += c;
            if tokens.len() <= *count as usize {
                return Err(create_error!(ErrCode::E0002));
            }
        }
        Ok(Environment::create_list(list))
    } else if ")" == token {
        Err(create_error!(ErrCode::E0003))
    } else {
        // string check ex. <rust-elisp> "abc
        if (token == "\"") || (token.starts_with("\"") && !token.ends_with("\"")) {
            return Err(create_error!(ErrCode::E0004));
        }
        atom(&token, env)
    }
}
fn atom(token: &String, env: &Environment) -> ResultExpression {
    let v = if let Ok(n) = token.parse::<i64>() {
        Expression::Integer(n)
    } else if let Ok(n) = token.parse::<f64>() {
        Expression::Float(n)
    } else if token == TRUE {
        Expression::Boolean(true)
    } else if token == FALSE {
        Expression::Boolean(false)
    } else if token == SPACE.1 {
        Expression::Char(char::from(SPACE.0))
    } else if token == TAB.1 {
        Expression::Char(char::from(TAB.0))
    } else if token == NEWLINE.1 {
        Expression::Char(char::from(NEWLINE.0))
    } else if token == CARRIAGERETRUN.1 {
        Expression::Char(char::from(CARRIAGERETRUN.0))
    } else if (token.starts_with("#\\")) && (token.as_str().chars().count() == 3) {
        let c = token.chars().collect::<Vec<char>>();
        Expression::Char(c[2])
    } else if (token.len() >= 2) && (token.starts_with("\"")) && (token.ends_with("\"")) {
        let s = token[1..token.len() - 1].to_string();
        Expression::String(s)
    } else if let Some(f) = env.get_builtin_func(token.as_str()) {
        Expression::BuildInFunction(token.clone(), f)
    } else if let Some(f) = env.get_builtin_ext_func(token.as_str()) {
        Expression::BuildInFunctionExt(f)
    } else {
        match Rat::from(&token) {
            Ok(n) => Expression::Rational(n),
            Err(n) => {
                if n.code != ErrCode::E1020 {
                    return Err(create_error!(n.code));
                }
                Expression::Symbol(token.to_string())
            }
        }
    };
    Ok(v)
}
pub fn eval(sexp: &Expression, env: &Environment) -> ResultExpression {
    #[cfg(feature = "signal")]
    catch_sig_intr_status(env);

    if env.is_force_stop() {
        return Err(create_error!(ErrCode::E9000));
    }
    if let Expression::Symbol(val) = sexp {
        match env.find(&val) {
            Some(v) => Ok(v),
            None => Err(create_error_value!(ErrCode::E1008, val)),
        }
    } else if let Expression::List(val) = sexp {
        debug!("eval = {:?}", get_ptr!(&val));

        let v = &*(referlence_list!(val));
        if v.len() == 0 {
            return Ok(sexp.clone());
        }
        return match &v[0] {
            Expression::BuildInFunction(_, f) => f(&v[..], env),
            Expression::BuildInFunctionExt(f) => f(&v[..], env),
            Expression::TailRecursion(f) => f.set_param(&v[..], env),
            Expression::Function(f) => f.execute(&v[..], env),
            _ => match eval(&v[0], env)? {
                Expression::Function(f) => f.execute(&v[..], env),
                Expression::BuildInFunction(_, f) => f(&v[..], env),
                Expression::BuildInFunctionExt(f) => f(&v[..], env),
                Expression::Continuation(f) => f.execute(&v[..], env),
                _ => Err(create_error!(ErrCode::E1006)),
            },
        };
    } else {
        Ok(sexp.clone())
    }
}
