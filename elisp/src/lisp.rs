/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::vec::Vec;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::buildin::identity;
use crate::number::Rat;

#[cfg(feature = "thread")]
use crate::env_thread::{ExtOperationRc, FunctionRc, LetLoopRc};
#[cfg(feature = "thread")]
pub type Environment = crate::env_thread::Environment;

#[cfg(not(feature = "thread"))]
use crate::env_single::{ExtOperationRc, FunctionRc, LetLoopRc};
#[cfg(not(feature = "thread"))]
pub type Environment = crate::env_single::Environment;

//========================================================================
lazy_static! {
    static ref ERRMSG_TBL: HashMap<&'static str, &'static str> = {
        let mut e: HashMap<&'static str, &'static str> = HashMap::new();
        e.insert("E0001", "Unexpected EOF while reading");
        e.insert("E0002", "Unexpected ')' while reading");
        e.insert("E0003", "Extra close parenthesis `)'");
        e.insert("E0004", "Charactor syntax error");
        e.insert("E1001", "Not Boolean");
        e.insert("E1002", "Not Integer");
        e.insert("E1003", "Not Number");
        e.insert("E1004", "Not Symbol");
        e.insert("E1005", "Not List");
        e.insert("E1006", "Not Function");
        e.insert("E1007", "Not Enough Parameter Counts");
        e.insert("E1008", "Undefine variable");
        e.insert("E1009", "Not Enough Data Type");
        e.insert("E1010", "Not Promise");
        e.insert("E1011", "Not Enough List Length");
        e.insert("E1012", "Not Cond Gramar");
        e.insert("E1013", "Calculate A Division By Zero");
        e.insert("E1014", "Not Found Program File");
        e.insert("E1015", "Not String");
        e.insert("E1016", "Not Program File");
        e.insert("E1017", "Not Case Gramar");
        e.insert("E9999", "System Panic");
        e
    };
}
pub struct RsError {
    pub code: &'static str,
    pub line: u32,
    pub file: &'static str,
    pub value: Option<String>,
}
impl RsError {
    pub fn get_code(&self) -> String {
        String::from(self.code)
    }
}
#[macro_export]
macro_rules! create_error {
    ($e: expr) => {
        RsError {
            code: $e,
            line: line!(),
            file: file!(),
            value: None,
        }
    };
}
#[macro_export]
macro_rules! create_error_value {
    ($e: expr, $v: expr) => {
        RsError {
            code: $e,
            line: line!(),
            file: file!(),
            value: Some($v.to_string()),
        }
    };
}
#[macro_export]
macro_rules! print_error {
    ($e: expr) => {
        if let Some(s) = $e.value {
            println!(
                "{}: {} ({}:{})",
                ERRMSG_TBL.get($e.code).unwrap(),
                s,
                $e.file,
                $e.line
            )
        } else {
            println!(
                "{} ({}:{})",
                ERRMSG_TBL.get($e.code).unwrap(),
                $e.file,
                $e.line
            )
        }
    };
}
//========================================================================
pub type ResultExpression = Result<Expression, RsError>;
pub type Operation = fn(&[Expression], &mut Environment) -> ResultExpression;
pub type ExtOperation = Fn(&[Expression], &mut Environment) -> ResultExpression;
//========================================================================
#[derive(Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    List(Vec<Expression>),
    Pair(Box<Expression>, Box<Expression>),
    Symbol(String),
    String(String),
    Function(FunctionRc),
    BuildInFunction(Operation),
    BuildInFunctionExt(ExtOperationRc),
    LetLoop(LetLoopRc),
    Loop(),
    TailLoop(),
    Nil(),
    TailRecursion(FunctionRc),
    Promise(Box<Expression>, Environment),
    Rational(Rat),
    CPS(RsCPS),
}
impl Expression {
    pub fn value_string(&self) -> String {
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
                    return v.to_string();
                }
            }
            Expression::Boolean(v) => (if *v { TRUE } else { FALSE }).to_string(),
            Expression::Symbol(v) => v.to_string(),
            Expression::String(v) => format!("\"{}\"", v),
            Expression::List(v) => Expression::list_string(&v[..]),
            Expression::Pair(car, cdr) => {
                String::from(format!("({} . {})", car.value_string(), cdr.value_string()))
            }
            Expression::Function(_) => String::from("Function"),
            Expression::BuildInFunction(_) => String::from("BuildIn Function"),
            Expression::BuildInFunctionExt(_) => String::from("BuildIn Function Ext"),
            Expression::LetLoop(_) => String::from("LetLoop"),
            Expression::Nil() => String::from("nil"),
            Expression::Loop() => String::from("let loop"),
            Expression::TailLoop() => String::from("tail loop"),
            Expression::TailRecursion(_) => String::from("Tail Recursion"),
            Expression::Promise(_, _) => String::from("Promise"),
            Expression::Rational(v) => v.to_string(),
            Expression::CPS(_) => String::from("CPS"),
        };
    }
    fn list_string(exp: &[Expression]) -> String {
        let mut s = String::from("(");

        let mut c = 1;
        let mut el = false;
        for e in exp {
            if let Expression::List(l) = e {
                s.push_str(Expression::list_string(&l[..]).as_str());
                el = true;
            } else {
                if el {
                    s.push_str(" ");
                }
                s.push_str(e.value_string().as_str());
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
pub trait TailRecursion {
    fn myname(&self) -> &String;

    fn parse_tail_recurcieve(&self, exp: &[Expression]) -> bool {
        let (mut n, mut c, mut tail) = (0, 0, false);
        for e in exp {
            if let Expression::List(l) = e {
                if 0 == l.len() {
                    continue;
                }
                if let Expression::Symbol(s) = &l[0] {
                    match s.as_str() {
                        "if" | "let" | "cond" | "else" => {
                            return self.parse_tail_recurcieve(&l[1..])
                        }
                        _ => {}
                    }
                    if *s == *self.myname() {
                        if (exp.len() - 1) == c {
                            debug!(
                                "tail recursion {} {} {} {}",
                                exp.len(),
                                c,
                                n,
                                *self.myname()
                            );
                            tail = true;
                        }
                        n = n + 1;
                    }
                }
            }
            c = c + 1;
        }
        if n == 1 && tail {
            return true;
        }
        return false;
    }
}
#[derive(Clone)]
pub struct RsCPS {
    name: String,
    list: LinkedList<(Expression, HashMap<String, Expression>)>,
    param: Vec<String>,
}
impl RsCPS {
    fn new(s: &String, param: Vec<String>) -> Self {
        let list = LinkedList::new();
        RsCPS {
            name: s.to_string(),
            list: list,
            param: param,
        }
    }
    fn add(&mut self, exp: Expression, env: &Environment) {
        let mut h = HashMap::new();
        for s in &self.param {
            if self.name == *s {
                continue;
            }
            if let Some(e) = env.find(s) {
                h.insert(s.clone(), e);
            }
        }
        self.list.push_front((exp, h));
    }
    pub fn execute(&self, exp: &Vec<Expression>, env: &mut Environment) -> ResultExpression {
        if exp.len() != 2 {
            return Err(create_error_value!("E1007", exp.len()));
        }
        env.regist(self.name.clone(), Expression::BuildInFunction(identity));
        let mut vec = Vec::new();
        vec.push(Expression::Nil());
        vec.push(exp[1].clone());
        for (e, h) in self.list.iter() {
            if let Expression::Function(f) = e {
                for (k, v) in h.iter() {
                    env.regist(k.clone(), v.clone());
                }
                let e = f.execute(&vec, env)?;
                debug!("@@@ CPS execute {} {}", self.name, e.value_string());
                vec[1] = e;
            }
        }
        Ok(vec[1].clone())
    }
}
#[derive(Clone)]
pub struct RsFunction {
    param: Vec<String>,
    body: Vec<Expression>,
    name: String,
    closure_env: Environment,
    tail_recurcieve: bool,
}
impl RsFunction {
    pub fn new(sexp: &[Expression], name: String, closure_env: Environment) -> Self {
        let mut param: Vec<String> = Vec::new();

        if let Expression::List(val) = &sexp[1] {
            for n in val {
                if let Expression::Symbol(s) = n {
                    param.push(s.to_string());
                }
            }
        }
        let mut vec: Vec<Expression> = Vec::new();
        vec.extend_from_slice(&sexp[2..]);
        RsFunction {
            param: param,
            body: vec,
            name: name,
            closure_env: closure_env,
            tail_recurcieve: false,
        }
    }
    pub fn set_tail_recurcieve(&mut self) {
        self.tail_recurcieve = self.parse_tail_recurcieve(self.body.as_slice());
    }
    pub fn set_param(&self, exp: &Vec<Expression>, env: &mut Environment) -> ResultExpression {
        // param eval
        let mut vec: Vec<Expression> = Vec::new();

        let mut idx = 0;
        for e in &exp[1 as usize..] {
            let v = eval(e, env)?;
            match v {
                Expression::Function(_) => {
                    if let Some(e) = env.find(&self.param[idx]) {
                        match e {
                            Expression::Function(f) => {
                                let mut cps = RsCPS::new(&self.param[idx], self.param.clone());
                                cps.add(Expression::Function(f), env);
                                cps.add(v, env);
                                vec.push(Expression::CPS(cps));
                            }
                            Expression::CPS(mut cps) => {
                                cps.add(v, env);
                                vec.push(Expression::CPS(cps));
                            }
                            _ => return Err(create_error!("E9999")),
                        }
                    }
                }
                v => vec.push(v),
            }
            idx += 1;
        }
        // env set
        let mut idx = 0;
        for s in &self.param {
            env.update(&s, vec[idx].clone());
            idx += 1;
        }
        Ok(Expression::TailLoop())
    }
    pub fn execute(&self, exp: &Vec<Expression>, env: &mut Environment) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error_value!("E1007", exp.len()));
        }
        // param eval
        let mut vec: Vec<Expression> = Vec::new();
        for e in &exp[1 as usize..] {
            vec.push(eval(e, env)?);
        }
        return self.execute_noeval(&vec);
    }
    pub fn execute_noeval(&self, exp: &Vec<Expression>) -> ResultExpression {
        if self.param.len() != exp.len() {
            return Err(create_error_value!("E1007", exp.len()));
        }
        // @@@ env.create();
        let mut env = Environment::new_next(&self.closure_env);
        let mut idx = 0;
        for s in &self.param {
            env.regist(s.to_string(), exp[idx].clone());
            idx += 1;
        }
        if self.tail_recurcieve == true && env.is_tail_recursion() == true {
            env.regist(
                self.name.to_string(),
                Environment::create_tail_recursion(self.clone()),
            );
        }
        // execute!
        let mut i = 0;
        for e in &self.body {
            loop {
                match eval(e, &mut env)? {
                    Expression::TailLoop() => {
                        continue;
                    }
                    v => {
                        i += 1;
                        if i == self.body.len() {
                            return Ok(v);
                        }
                        break;
                    }
                }
            }
        }
        Err(create_error!("E9999"))
    }
}
impl TailRecursion for RsFunction {
    fn myname(&self) -> &String {
        &self.name
    }
}
#[derive(Clone)]
pub struct RsLetLoop {
    param: Vec<String>,
    body: Vec<Expression>,
    name: String,
    tail_recurcieve: bool,
}
impl RsLetLoop {
    pub fn new(sexp: &[Expression], name: String, param: &Vec<String>) -> Self {
        let mut vec: Vec<Expression> = Vec::new();
        vec.extend_from_slice(&sexp[3..]);
        RsLetLoop {
            param: param.clone(),
            body: vec,
            name: name,
            tail_recurcieve: false,
        }
    }
    // exp is slice
    pub fn set_tail_recurcieve(&mut self) {
        self.tail_recurcieve = self.parse_tail_recurcieve(self.body.as_slice());
    }
    pub fn execute(&self, exp: &[Expression], env: &mut Environment) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error_value!("E1007", exp.len()));
        }

        let mut iter = exp.iter();
        iter.next();
        for s in &self.param {
            if let Some(e) = iter.next() {
                let v = eval(e, env)?;
                env.update(s, v);
            }
        }
        if self.tail_recurcieve == true {
            return Ok(Expression::Loop());
        } else {
            let mut i = 0;
            for exp in &self.body {
                let v = eval(&exp, env)?;
                i += 1;
                if i == self.body.len() {
                    return Ok(v);
                }
            }
        }
        return Err(create_error!("E9999"));
    }
}
impl TailRecursion for RsLetLoop {
    fn myname(&self) -> &String {
        &self.name
    }
}

//========================================================================
const PROMPT: &str = "<rust.elisp> ";
const QUIT: &str = "(quit)";
const TAIL_OFF: &str = "(tail-recursion-off)";
const TAIL_ON: &str = "(tail-recursion-on)";

struct ControlChar(u8, &'static str);
const SPACE: ControlChar = ControlChar(0x20, "#\\space");
const TAB: ControlChar = ControlChar(0x09, "#\\tab");
const NEWLINE: ControlChar = ControlChar(0x0A, "#\\newline");
const CARRIAGERETRUN: ControlChar = ControlChar(0x0D, "#\\return");

const TRUE: &'static str = "#t";
const FALSE: &'static str = "#f";
//========================================================================
pub fn do_interactive() {
    let mut stream = BufReader::new(std::io::stdin());
    let mut env = Environment::new();

    match repl(&mut stream, &mut env, false) {
        Err(e) => println!("{}", e),
        Ok(_) => {}
    }
}
pub fn repl(
    stream: &mut BufRead,
    env: &mut Environment,
    batch: bool,
) -> Result<(), Box<std::error::Error>> {
    let mut buffer = String::new();
    let mut program: Vec<String> = Vec::new();
    let mut w = std::io::stdout();
    let mut prompt = PROMPT;

    loop {
        if !batch {
            print!("{}", prompt);
        }
        w.flush().unwrap();
        buffer.clear();
        let n = stream.read_line(&mut buffer)?;
        if n == 0 {
            break;
        }
        if buffer.trim() == QUIT {
            println!("Bye");
            break;
        } else if buffer.trim() == TAIL_ON {
            env.set_tail_recursion(true);
            continue;
        } else if buffer.trim() == TAIL_OFF {
            env.set_tail_recursion(false);
            continue;
        } else if buffer.trim() == "" {
            continue;
        } else if buffer.as_bytes()[0] as char == ';' {
            continue;
        }
        program.push(buffer.trim().to_string());
        if false == count_parenthesis(program.join(" ")) {
            prompt = "";
            continue;
        }
        debug!("{}", program.iter().cloned().collect::<String>());
        {
            let mut token = tokenize(&program.join(" "));
            let mut c: i32 = 1;

            loop {
                let exp = match parse(&token, &mut c) {
                    Ok(v) => v,
                    Err(e) => {
                        print_error!(e);
                        break;
                    }
                };
                match eval(&exp, env) {
                    Ok(n) => println!("{}", n.value_string()),
                    Err(e) => print_error!(e),
                };
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
        }
        program.clear();
        prompt = PROMPT;
    }
    Ok(())
}
fn count_parenthesis(program: String) -> bool {
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
pub fn do_core_logic(program: &String, env: &mut Environment) -> ResultExpression {
    let token = tokenize(program);

    let mut c: i32 = 1;
    let exp = parse(&token, &mut c)?;
    return eval(&exp, env);
}
fn tokenize(program: &String) -> Vec<String> {
    let mut token: Vec<String> = Vec::new();
    let mut string_mode = false;
    let mut symbol_name = String::new();
    let mut from = 0;
    let mut i = 0;

    let mut s = program.clone();
    s = s.replace("\t", " ");
    s = s.replace("\n", " ");
    s = s.replace("\r", " ");
    let vc = s.as_bytes();

    //A String is a wrapper over a Vec<u8>.(https://doc.rust-lang.org/book/ch08-02-strings.html)
    for c in s.as_str().chars() {
        if string_mode {
            if c == '"' {
                if vc[i - 1] != 0x5c {
                    let ls = s.get(from..(i + 1)).unwrap();
                    token.push(ls.to_string());
                    string_mode = false;
                }
            }
        } else {
            if c == '"' {
                from = i;
                string_mode = true;
            } else if c == '(' {
                token.push(String::from("("));
            } else if c == ')' {
                token.push(String::from(")"));
            } else if c == ' ' {
                // Nop
            } else {
                symbol_name.push(c);
                if s.len() - c.len_utf8() == i {
                    token.push(String::from(symbol_name.as_str()));
                } else {
                    match vc[i + c.len_utf8()] as char {
                        '(' | ')' | ' ' => {
                            token.push(String::from(symbol_name.as_str()));
                            symbol_name.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
        i += c.len_utf8();
    }
    if string_mode {
        token.push(s.get(from..i).unwrap().to_string());
    }
    return token;
}
fn parse(tokens: &Vec<String>, count: &mut i32) -> ResultExpression {
    if tokens.len() == 0 {
        return Err(create_error!("E0001"));
    }

    let token = &tokens[0];
    if "(" == token {
        if tokens.len() <= 1 {
            return Err(create_error!("E0001"));
        }
        let mut list: Vec<Expression> = Vec::new();

        *count = 1;
        loop {
            if tokens[*count as usize] == ")" {
                *count += 1;
                break;
            }
            let mut c: i32 = 1;
            let o = parse(&tokens[*count as usize..].to_vec(), &mut c)?;
            list.push(o);

            *count += c;
            if tokens.len() <= *count as usize {
                return Err(create_error!("E0002"));
            }
        }
        Ok(Expression::List(list))
    } else if ")" == token {
        Err(create_error!("E0003"))
    } else {
        // string check
        if (token == "\"") || (token.as_str().starts_with("\"") && !token.as_str().ends_with("\""))
        {
            return Err(create_error!("E0004"));
        }
        Ok(atom(&token))
    }
}
fn atom(token: &String) -> Expression {
    if let Ok(n) = token.parse::<i64>() {
        return Expression::Integer(n);
    }
    if let Ok(n) = token.parse::<f64>() {
        return Expression::Float(n);
    }
    if token.as_str() == TRUE {
        return Expression::Boolean(true);
    }
    if token.as_str() == FALSE {
        return Expression::Boolean(false);
    }
    if token.as_str() == SPACE.1 {
        return Expression::Char(char::from(SPACE.0));
    }
    if token.as_str() == TAB.1 {
        return Expression::Char(char::from(TAB.0));
    }
    if token.as_str() == NEWLINE.1 {
        return Expression::Char(char::from(NEWLINE.0));
    }
    if token.as_str() == CARRIAGERETRUN.1 {
        return Expression::Char(char::from(CARRIAGERETRUN.0));
    }
    if (token.len() == 3) && (token.as_str().starts_with("#\\")) {
        let c = token.chars().collect::<Vec<char>>();
        return Expression::Char(c[2]);
    }
    if (token.len() >= 2) && (token.as_str().starts_with("\"")) && (token.as_str().ends_with("\""))
    {
        let s = token.as_str()[1..token.len() - 1].to_string();
        return Expression::String(s);
    }
    let mut v = Vec::new();
    for e in token.split("/") {
        if let Ok(n) = e.parse::<i64>() {
            v.push(n);
        }
    }
    if v.len() == 2 {
        Expression::Rational(Rat::new(v[0], v[1]))
    } else {
        Expression::Symbol(token.to_string())
    }
}
macro_rules! ret_clone_if_atom {
    ($e: expr) => {
        match $e {
            Expression::Integer(_) => return Ok($e.clone()),
            Expression::Boolean(_) => return Ok($e.clone()),
            Expression::Char(_) => return Ok($e.clone()),
            Expression::Float(_) => return Ok($e.clone()),
            Expression::String(_) => return Ok($e.clone()),
            Expression::Nil() => return Ok($e.clone()),
            Expression::Pair(_, _) => return Ok($e.clone()),
            Expression::Promise(_, _) => return Ok($e.clone()),
            Expression::Rational(_) => return Ok($e.clone()),
            _ => {}
        }
    };
}
pub fn eval(sexp: &Expression, env: &mut Environment) -> ResultExpression {
    ret_clone_if_atom!(sexp);

    if let Expression::Symbol(val) = sexp {
        match env.find(&val) {
            Some(v) => {
                ret_clone_if_atom!(v);
                return match v {
                    Expression::Function(_) => Ok(v),
                    Expression::TailRecursion(_) => Ok(v),
                    Expression::LetLoop(_) => Ok(v),
                    Expression::List(_) => Ok(v),
                    Expression::BuildInFunction(_) => Ok(v),
                    Expression::BuildInFunctionExt(_) => Ok(v),
                    Expression::CPS(_) => Ok(v),
                    _ => Err(create_error!("E9999")),
                };
            }
            None => {
                if let Some(f) = env.get_builtin_func(val.as_str()) {
                    Ok(Expression::BuildInFunction(f))
                } else if let Some(f) = env.get_builtin_ext_func(val.as_str()) {
                    Ok(Expression::BuildInFunctionExt(f))
                } else {
                    Err(create_error_value!("E1008", val))
                }
            }
        }
    } else if let Expression::List(v) = sexp {
        if v.len() == 0 {
            return Ok(sexp.clone());
        }
        return match eval(&v[0], env)? {
            Expression::LetLoop(f) => f.execute(v, env),
            Expression::Function(f) => f.execute(v, env),
            Expression::TailRecursion(f) => f.set_param(v, env),
            Expression::BuildInFunction(f) => f(&v[..], env),
            Expression::BuildInFunctionExt(f) => f(&v[..], env),
            Expression::CPS(f) => f.execute(v, env),
            _ => Err(create_error!("E1006")),
        };
    } else {
        Err(create_error!("E1009"))
    }
}
