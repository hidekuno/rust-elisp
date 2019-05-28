/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use std::vec::Vec;

use rand::Rng;

#[allow(unused_imports)]
use log::{debug, error, info, warn};
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
type ResultExpression = Result<Expression, RsError>;
type Operation = fn(&[Expression], &mut SimpleEnv) -> ResultExpression;
type ExtOperation = Fn(&[Expression], &mut SimpleEnv) -> ResultExpression;
//========================================================================
pub trait EvalResult {
    fn value_string(&self) -> String;
}
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
    Function(Rc<RsFunction>),
    BuildInFunction(Operation),
    BuildInFunctionExt(Rc<ExtOperation>),
    LetLoop(Rc<RsLetLoop>),
    Loop(),
    Nil(),
    TailRecursion(Rc<RsFunction>),
    Promise(Box<Expression>, SimpleEnv),
}
impl EvalResult for Expression {
    fn value_string(&self) -> String {
        return match self {
            Expression::Integer(v) => v.to_string(),
            Expression::Float(v) => v.to_string(),
            Expression::Char(v) => v.to_string(),
            Expression::Boolean(v) => (if *v { "#t" } else { "#f" }).to_string(),
            Expression::Symbol(v) => v.to_string(),
            Expression::String(v) => format!("\"{}\"", v),
            Expression::List(v) => list_string(&v[..]),
            Expression::Pair(car, cdr) => {
                String::from(format!("({} . {})", car.value_string(), cdr.value_string()))
            }
            Expression::Function(_) => String::from("Function"),
            Expression::BuildInFunction(_) => String::from("BuildIn Function"),
            Expression::BuildInFunctionExt(_) => String::from("BuildIn Function Ext"),
            Expression::LetLoop(_) => String::from("LetLoop"),
            Expression::Nil() => String::from("nil"),
            Expression::Loop() => String::from("loop"),
            Expression::TailRecursion(_) => String::from("Tail Recursion"),
            Expression::Promise(_, _) => String::from("Promise"),
        };
    }
}
fn list_string(exp: &[Expression]) -> String {
    let mut s = String::from("(");

    let mut c = 1;
    let mut el = false;
    for e in exp {
        if let Expression::List(l) = e {
            s.push_str(list_string(&l[..]).as_str());
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
pub trait TailRecursion {
    fn myname(&self) -> &String;

    fn parse_tail_recurcieve(&self, exp: &[Expression]) -> bool {
        let mut n = 0;
        for e in exp {
            if let Expression::List(l) = e {
                if 0 == l.len() {
                    continue;
                }
                if let Expression::Symbol(s) = &l[0] {
                    if s.as_str() == "if" || s.as_str() == "let" {
                        return self.parse_tail_recurcieve(&l[1..]);
                    }
                    if *s == *self.myname() {
                        n = n + 1;
                    }
                }
            }
        }
        if n == 1 {
            return true;
        }
        return false;
    }
}
#[derive(Clone)]
pub struct RsFunction {
    param: Vec<String>,
    body: Vec<Expression>,
    name: String,
    closure_env: LinkedList<HashMap<String, Expression>>,
    tail_recurcieve: bool,
}
impl RsFunction {
    fn new(sexp: &[Expression], _name: String) -> RsFunction {
        let mut _param: Vec<String> = Vec::new();
        let l: LinkedList<HashMap<String, Expression>> = LinkedList::new();

        if let Expression::List(val) = &sexp[1] {
            for n in val {
                if let Expression::Symbol(s) = n {
                    _param.push(s.to_string());
                }
            }
        }
        let mut vec: Vec<Expression> = Vec::new();
        vec.extend_from_slice(&sexp[2..]);
        RsFunction {
            param: _param,
            body: vec,
            name: _name,
            closure_env: l,
            tail_recurcieve: false,
        }
    }
    fn add_closure_env(&mut self, map: HashMap<String, Expression>) {
        self.closure_env.push_front(map);
    }
    fn set_tail_recurcieve(&mut self) {
        self.tail_recurcieve = self.parse_tail_recurcieve(self.body.as_slice());
    }
    fn set_param(&self, exp: &Vec<Expression>, env: &mut SimpleEnv) -> ResultExpression {
        // param eval
        let mut vec: Vec<Expression> = Vec::new();
        for e in &exp[1 as usize..] {
            let v = eval(e, env)?;
            vec.push(v);
        }
        // env set
        let mut idx = 0;
        for s in &self.param {
            env.update(&s, vec[idx].clone());
            idx += 1;
        }
        return Ok(Expression::TailRecursion(Rc::new(self.clone())));
    }
    fn execute(&mut self, exp: &Vec<Expression>, env: &mut SimpleEnv) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error_value!("E1007", exp.len()));
        }
        // param eval
        let mut vec: Vec<Expression> = Vec::new();
        for e in &exp[1 as usize..] {
            let v = eval(e, env)?;
            vec.push(v);
        }
        return self.execute_noeval(&vec, env);
    }
    fn execute_noeval(&mut self, exp: &Vec<Expression>, env: &mut SimpleEnv) -> ResultExpression {
        if self.param.len() != exp.len() {
            return Err(create_error_value!("E1007", exp.len()));
        }
        // closure set
        for h in self.closure_env.iter() {
            env.create();
            for (k, v) in h {
                env.regist(k.to_string(), v.clone());
            }
        }
        // param set
        env.create();
        let mut idx = 0;
        for s in &self.param {
            env.regist(s.to_string(), exp[idx].clone());
            idx += 1;
        }
        if self.tail_recurcieve == true {
            env.regist(
                self.name.to_string(),
                Expression::TailRecursion(Rc::new(self.clone())),
            );
        }

        // execute!
        let mut results: Vec<Expression> = Vec::new();
        for e in &self.body {
            loop {
                let v = eval(e, env)?;
                if let Expression::TailRecursion(_) = v {
                    continue;
                } else {
                    results.push(v);
                    break;
                }
            }
        }
        // param clear
        env.delete();

        // env(closure) clear and self(closure) saved
        for h in self.closure_env.iter_mut().rev() {
            for (k, _v) in h.clone() {
                if let Some(exp) = env.find(&k) {
                    h.insert(k.to_string(), (*exp).clone());
                }
            }
            env.delete();
        }

        // created function set clonsure
        if let Some(r) = results.pop() {
            if let Expression::Function(mut rc) = r {
                // https://doc.rust-lang.org/std/rc/struct.Rc.html#method.get_mut
                let f = Rc::make_mut(&mut rc);
                let mut closure_env: HashMap<String, Expression> = HashMap::new();

                let mut idx = 0;
                for s in &self.param {
                    closure_env.insert(s.to_string(), exp[idx].clone());
                    idx += 1;
                }
                if idx > 0 {
                    f.add_closure_env(closure_env);
                }
                return Ok(Expression::Function(rc));
            }
            return Ok(r);
        }
        return Err(create_error!("E9999"));
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
    fn new(sexp: &[Expression], _name: String, _param: &Vec<String>) -> RsLetLoop {
        let mut vec: Vec<Expression> = Vec::new();
        vec.extend_from_slice(&sexp[3..]);
        RsLetLoop {
            param: _param.clone(),
            body: vec,
            name: _name,
            tail_recurcieve: false,
        }
    }
    // exp is slice
    fn set_tail_recurcieve(&mut self) {
        self.tail_recurcieve = self.parse_tail_recurcieve(self.body.as_slice());
    }
    fn execute(&self, exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
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
            let mut results: Vec<Expression> = Vec::new();
            for exp in &self.body {
                let r = eval(&exp, env)?;
                results.push(r);
            }
            if let Some(r) = results.pop() {
                return Ok(r);
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

#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
}
impl Number {
    fn calc_template(
        x: Number,
        y: Number,
        fcalc: fn(x: f64, y: f64) -> f64,
        icalc: fn(x: i64, y: i64) -> i64,
    ) -> Number {
        match x {
            Number::Integer(a) => match y {
                Number::Integer(b) => Number::Integer(icalc(a, b)),
                Number::Float(b) => Number::Float(fcalc(a as f64, b)),
            },
            Number::Float(a) => match y {
                Number::Integer(b) => Number::Float(fcalc(a, b as f64)),
                Number::Float(b) => Number::Float(fcalc(a, b)),
            },
        }
    }
    fn cmp_template(
        x: Number,
        y: Number,
        fop: fn(x: f64, y: f64) -> bool,
        iop: fn(x: i64, y: i64) -> bool,
    ) -> bool {
        match x {
            Number::Integer(a) => match y {
                Number::Integer(b) => return iop(a, b),
                Number::Float(b) => return fop(a as f64, b),
            },
            Number::Float(a) => match y {
                Number::Integer(b) => return fop(a, b as f64),
                Number::Float(b) => return fop(a, b),
            },
        }
    }
}
//impl<T: Add<Output=T>> Add for Number<T> {
impl Add for Number {
    type Output = Number;
    fn add(self, other: Number) -> Number {
        return Number::calc_template(self, other, |x: f64, y: f64| x + y, |x: i64, y: i64| x + y);
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        return Number::calc_template(self, other, |x: f64, y: f64| x - y, |x: i64, y: i64| x - y);
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        return Number::calc_template(self, other, |x: f64, y: f64| x * y, |x: i64, y: i64| x * y);
    }
}
impl Div for Number {
    type Output = Number;
    fn div(self, other: Number) -> Number {
        if let Number::Integer(x) = self {
            if let Number::Integer(y) = other {
                if x == 0 && y == 0 {
                    return Number::Float(std::f64::NAN);
                }
                if y == 0 {
                    return Number::Float(std::f64::INFINITY);
                }
            }
        }
        return Number::calc_template(self, other, |x: f64, y: f64| x / y, |x: i64, y: i64| x / y);
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        return Number::cmp_template(
            *self,
            *other,
            |x: f64, y: f64| x == y,
            |x: i64, y: i64| x == y,
        );
    }
}
impl PartialOrd for Number {
    fn lt(&self, other: &Number) -> bool {
        return Number::cmp_template(
            *self,
            *other,
            |x: f64, y: f64| x < y,
            |x: i64, y: i64| x < y,
        );
    }
    fn le(&self, other: &Number) -> bool {
        return Number::cmp_template(
            *self,
            *other,
            |x: f64, y: f64| x <= y,
            |x: i64, y: i64| x <= y,
        );
    }
    fn gt(&self, other: &Number) -> bool {
        return Number::cmp_template(
            *self,
            *other,
            |x: f64, y: f64| x > y,
            |x: i64, y: i64| x > y,
        );
    }
    fn ge(&self, other: &Number) -> bool {
        return Number::cmp_template(
            *self,
            *other,
            |x: f64, y: f64| x >= y,
            |x: i64, y: i64| x >= y,
        );
    }
    fn partial_cmp(&self, _: &Number) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}
#[derive(Clone)]
pub struct SimpleEnv {
    env_tbl: LinkedList<HashMap<String, Expression>>,
    builtin_tbl: HashMap<&'static str, Operation>,
    builtin_tbl_ext: HashMap<&'static str, Rc<ExtOperation>>,
}
impl SimpleEnv {
    pub fn new() -> SimpleEnv {
        let mut l: LinkedList<HashMap<String, Expression>> = LinkedList::new();
        l.push_back(HashMap::new());

        let mut b: HashMap<&'static str, Operation> = HashMap::new();

        b.insert("+", |exp, env| calc(exp, env, |x, y| x + y));
        b.insert("-", |exp, env| calc(exp, env, |x, y| x - y));
        b.insert("*", |exp, env| calc(exp, env, |x, y| x * y));
        b.insert("/", |exp, env| calc(exp, env, |x, y| x / y));
        b.insert("=", |exp, env| op(exp, env, |x, y| x == y));
        b.insert("<", |exp, env| op(exp, env, |x, y| x < y));
        b.insert("<=", |exp, env| op(exp, env, |x, y| x <= y));
        b.insert(">", |exp, env| op(exp, env, |x, y| x > y));
        b.insert(">=", |exp, env| op(exp, env, |x, y| x >= y));
        b.insert("expt", expt);
        b.insert("modulo", modulo);
        b.insert("define", define);
        b.insert("lambda", lambda);
        b.insert("if", if_f);
        b.insert("and", and);
        b.insert("or", or);
        b.insert("not", not);
        b.insert("let", let_f);
        b.insert("time", time_f);
        b.insert("set!", set_f);

        b.insert("list", list);
        b.insert("null?", null_f);
        b.insert("length", length);
        b.insert("car", car);
        b.insert("cdr", cdr);
        b.insert("cadr", cadr);
        b.insert("cons", cons);
        b.insert("append", append);
        b.insert("last", last);
        b.insert("reverse", reverse);
        b.insert("iota", iota);
        b.insert("map", map);
        b.insert("filter", filter);
        b.insert("reduce", reduce);
        b.insert("for-each", for_each);

        b.insert("sqrt", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.sqrt()))
        });
        b.insert("sin", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.sin()))
        });
        b.insert("cos", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.cos()))
        });
        b.insert("tan", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.tan()))
        });
        b.insert("atan", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.atan()))
        });
        b.insert("exp", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.exp()))
        });
        b.insert("log", |exp, env| {
            Ok(Expression::Float(to_f64(exp, env)?.log((1.0 as f64).exp())))
        });
        b.insert("rand-integer", rand_integer);
        b.insert("rand-list", rand_list);

        if let Some(r) = b.get("/") {
            b.insert("quotient", *r);
        }
        b.insert("load-file", load_file);
        b.insert("display", display);
        b.insert("delay", delay);
        b.insert("force", force);

        SimpleEnv {
            env_tbl: l,
            builtin_tbl: b,
            builtin_tbl_ext: HashMap::new(),
        }
    }
    fn create(&mut self) {
        self.env_tbl.push_front(HashMap::new());
    }
    fn delete(&mut self) {
        self.env_tbl.pop_front();
    }
    fn regist(&mut self, key: String, exp: Expression) {
        match self.env_tbl.front_mut() {
            Some(m) => {
                m.insert(key, exp);
            }
            None => {}
        }
    }
    fn find(&self, key: &String) -> Option<&Expression> {
        for h in self.env_tbl.iter() {
            match h.get(key) {
                Some(v) => {
                    return Some(v);
                }
                None => {}
            }
        }
        None
    }
    fn update(&mut self, key: &String, exp: Expression) {
        for h in self.env_tbl.iter_mut() {
            match h.get(key) {
                Some(_) => {
                    h.insert(key.to_string(), exp);
                    return;
                }
                None => {}
            }
        }
    }
    fn cleanup(&mut self) {
        while self.env_tbl.len() >= 2 {
            self.delete();
        }
    }
    pub fn add_builtin_func(&mut self, key: &'static str, func: Operation) {
        self.builtin_tbl.insert(key, func);
    }
    pub fn add_builtin_closure<F>(&mut self, key: &'static str, c: F)
    where
        F: Fn(&[Expression], &mut SimpleEnv) -> ResultExpression + 'static,
        // F: ExtOperation + 'static, => type aliases cannot be used as traits
    {
        self.builtin_tbl_ext.insert(key, Rc::new(c));
    }
    #[allow(dead_code)]
    fn dump_env(&self) {
        debug!("======== dump_env start ============");
        let mut i = 1;
        for exp in self.env_tbl.iter() {
            for (k, v) in exp {
                debug!("{} {} nest:{}", k, v.value_string(), i);
            }
            i += 1;
        }
    }
    #[allow(dead_code)]
    fn dump_env_level(&self) {
        debug!(
            "======== dump_env level {} ============",
            self.env_tbl.len()
        );
    }
}

//========================================================================
const PROMPT: &str = "<rust.elisp> ";
const QUIT: &str = "(quit)";
const SAMPLE_INT: i64 = 10000000000000;
//========================================================================
fn set_f(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    match &exp[1] {
        Expression::Symbol(s) => {
            if let Some(_) = env.find(s) {
                let se = eval(&exp[2], env)?;
                env.update(s, se);
                return Ok(Expression::Symbol(s.to_string()));
            } else {
                return Err(create_error_value!("E1008", s));
            }
        }
        _ => {
            return Err(create_error!("E1004"));
        }
    }
}
fn time_f(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }

    let start = Instant::now();

    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{:06}", end.as_secs(), end.subsec_nanos() / 1_000_000);
    return result;
}
fn let_f(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut param: HashMap<String, Expression> = HashMap::new();

    let mut idx = 1;
    if let Expression::Symbol(_) = exp[idx] {
        idx += 1;
    }
    // Parameter Setup
    let mut param_list = Vec::new();
    if let Expression::List(l) = &exp[idx] {
        for plist in l {
            if let Expression::List(p) = plist {
                if p.len() != 2 {
                    return Err(create_error_value!("E1007", p.len()));
                }
                if let Expression::Symbol(s) = &p[0] {
                    let v = eval(&p[1], env)?;
                    param.insert(s.to_string(), v.clone());
                    param_list.push(s.clone());
                } else {
                    return Err(create_error!("E1004"));
                }
            } else {
                return Err(create_error!("E1005"));
            }
        }
        idx += 1;
    } else {
        return Err(create_error!("E1005"));
    }
    // Setup label name let
    if let Expression::Symbol(s) = &exp[1] {
        let mut letloop = RsLetLoop::new(exp, s.to_string(), &param_list);
        letloop.set_tail_recurcieve();
        param.insert(s.to_string(), Expression::LetLoop(Rc::new(letloop)));
    }

    // execute let
    let closure_env = param.clone();
    env.create();
    for (k, v) in param {
        env.regist(k, v);
    }
    let mut results: Vec<Expression> = Vec::new();
    for e in &exp[idx as usize..] {
        loop {
            let o = eval(e, env)?;
            if let Expression::Loop() = o {
                // tail recurcieve
                continue;
            } else {
                results.push(o);
                break;
            }
        }
    }
    env.delete();
    if let Some(r) = results.pop() {
        if let Expression::Function(mut rc) = r {
            // https://doc.rust-lang.org/std/rc/struct.Rc.html#method.get_mut
            let f = Rc::make_mut(&mut rc);
            f.add_closure_env(closure_env);
            return Ok(Expression::Function(rc));
        }
        return Ok(r);
    }
    return Err(create_error!("E9999"));
}
fn not(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;
    if let Expression::Boolean(b) = o {
        return Ok(Expression::Boolean(!b));
    }
    return Err(create_error!("E1001"));
}
fn or(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Expression::Boolean(b) = o {
            if b == true {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    return Ok(Expression::Boolean(false));
}
fn and(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Expression::Boolean(b) = o {
            if b == false {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    return Ok(Expression::Boolean(true));
}
fn expt(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut vec: Vec<i64> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Expression::Integer(i) = o {
            vec.push(i);
        } else {
            return Err(create_error!("E1002"));
        }
    }
    let m = vec[1].abs();
    let mut result: i64 = 1;
    for _i in 0..m {
        result *= vec[0];
    }
    if vec[1] < 0 {
        return Ok(Expression::Float(1 as f64 / result as f64));
    } else {
        return Ok(Expression::Integer(result));
    }
}
fn modulo(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut vec: Vec<i64> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;

        if let Expression::Integer(i) = o {
            vec.push(i);
        } else {
            return Err(create_error!("E1002"));
        }
    }
    if vec[1] == 0 {
        return Err(create_error!("E1013"));
    }
    return Ok(Expression::Integer(vec[0] % vec[1]));
}
fn lambda(exp: &[Expression], _env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::List(l) = &exp[1] {
        for e in l {
            match e {
                Expression::Symbol(_) => {}
                _ => return Err(create_error!("E1004")),
            }
        }
    } else {
        return Err(create_error!("E1005"));
    }
    return Ok(Expression::Function(Rc::new(RsFunction::new(
        exp,
        String::from("lambda"),
    ))));
}
fn define(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Symbol(v) = &exp[1] {
        let se = eval(&exp[2], env)?;
        env.regist(v.to_string(), se);
        return Ok(Expression::Symbol(v.to_string()));
    }
    if let Expression::List(l) = &exp[1] {
        if l.len() < 1 {
            return Err(create_error_value!("E1007", l.len()));
        }
        if let Expression::Symbol(s) = &l[0] {
            let mut f = exp.to_vec();

            let mut param: Vec<Expression> = Vec::new();
            for n in &l[1..] {
                match n {
                    Expression::Symbol(_) => {
                        param.push(n.clone());
                    }
                    _ => return Err(create_error!("E1004")),
                }
            }
            f[1] = Expression::List(param);
            let mut func = RsFunction::new(&f, s.to_string());
            func.set_tail_recurcieve();
            env.regist(s.to_string(), Expression::Function(Rc::new(func)));
            return Ok(Expression::Symbol(s.to_string()));
        } else {
            return Err(create_error!("E1004"));
        }
    }
    Err(create_error!("E1004"))
}
fn if_f(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let se = eval(&exp[1], env)?;

    if let Expression::Boolean(b) = se {
        if b == true {
            return eval(&exp[2], env);
        } else if 4 <= exp.len() {
            return eval(&exp[3], env);
        }
        return Ok(Expression::Nil());
    }
    return Err(create_error!("E1001"));
}
fn list(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    let mut list: Vec<Expression> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        list.push(o);
    }
    Ok(Expression::List(list))
}
fn null_f(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;
    if let Expression::List(l) = o {
        return Ok(Expression::Boolean(l.len() == 0));
    } else {
        return Ok(Expression::Boolean(false));
    }
}
fn length(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;
    if let Expression::List(l) = o {
        Ok(Expression::Integer(l.len() as i64))
    } else {
        Err(create_error!("E1005"))
    }
}
fn car(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;

    if let Expression::List(l) = o {
        if l.len() <= 0 {
            return Err(create_error!("E1011"));
        }
        return Ok(l[0].clone());
    } else if let Expression::Pair(car, _cdr) = o {
        return Ok((*car).clone());
    } else {
        Err(create_error!("E1005"))
    }
}
fn cdr(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;

    if let Expression::List(l) = o {
        if l.len() <= 0 {
            return Err(create_error!("E1011"));
        }
        if l.len() == 1 {
            let list: Vec<Expression> = Vec::new();
            return Ok(Expression::List(list));
        }
        return Ok(Expression::List(l[1 as usize..].to_vec()));
    } else if let Expression::Pair(_car, cdr) = o {
        return Ok((*cdr).clone());
    } else {
        Err(create_error!("E1005"))
    }
}
fn cadr(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;

    if let Expression::List(l) = o {
        if l.len() <= 1 {
            return Err(create_error!("E1011"));
        }
        return Ok(l[1].clone());
    } else {
        Err(create_error!("E1005"))
    }
}
fn cons(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let car = eval(&exp[1], env)?;
    let cdr = eval(&exp[2], env)?;

    if let Expression::List(mut l) = cdr {
        let mut v: Vec<Expression> = Vec::new();
        v.push(car);
        v.append(&mut l);
        Ok(Expression::List(v))
    } else {
        Ok(Expression::Pair(Box::new(car), Box::new(cdr)))
    }
}
fn append(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() <= 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut v: Vec<Expression> = Vec::new();

    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Expression::List(mut l) = o {
            v.append(&mut l);
        } else {
            return Err(create_error!("E1005"));
        }
    }
    return Ok(Expression::List(v));
}
fn last(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;

    if let Expression::List(l) = o {
        if 0 == l.len() {
            return Err(create_error!("E1011"));
        }
        return Ok(l[l.len() - 1].clone());
    }
    if let Expression::Pair(car, _) = o {
        return Ok(*car.clone());
    }

    return Err(create_error!("E1005"));
}

fn reverse(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let o = eval(&exp[1], env)?;
    if let Expression::List(l) = o {
        let mut v = l.clone();
        v.reverse();
        return Ok(Expression::List(v));
    }
    Err(create_error!("E1005"))
}
fn iota(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() <= 1 || 4 <= exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut i = 0;
    let mut max = 0;
    let mut l = Vec::new();
    let mut it = exp.iter();
    it.next();
    if let Some(e) = it.next() {
        let o = eval(e, env)?;
        if let Expression::Integer(e) = o {
            max = e;
        } else {
            return Err(create_error!("E1002"));
        }
    }
    if let Some(e) = it.next() {
        let o = eval(e, env)?;
        if let Expression::Integer(e) = o {
            i = e;
            max += i;
        } else {
            return Err(create_error!("E1002"));
        }
    }
    for v in i..max {
        l.push(Expression::Integer(v));
    }
    Ok(Expression::List(l))
}
fn map(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(mut rc) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[2], env)? {
            let mut result: Vec<Expression> = Vec::new();
            for e in l {
                let func = Rc::make_mut(&mut rc);
                result.push(func.execute_noeval(&[e.clone()].to_vec(), env)?);
            }
            return Ok(Expression::List(result));
        } else {
            Err(create_error!("E1005"))
        }
    } else {
        Err(create_error!("E1006"))
    }
}
fn filter(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(mut rc) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[2], env)? {
            let mut result: Vec<Expression> = Vec::new();
            for e in &l {
                let func = Rc::make_mut(&mut rc);
                if let Expression::Boolean(b) = func.execute_noeval(&[e.clone()].to_vec(), env)? {
                    if b {
                        result.push(e.clone());
                    }
                } else {
                    return Err(create_error!("E1001"));
                }
            }
            return Ok(Expression::List(result));
        } else {
            Err(create_error!("E1005"))
        }
    } else {
        Err(create_error!("E1006"))
    }
}
fn reduce(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(mut rc) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[3], env)? {
            if l.len() == 0 {
                return eval(&exp[2], env);
            }
            let mut result = l[0].clone();
            // not carfully length,  safety
            for e in &l[1 as usize..] {
                let func = Rc::make_mut(&mut rc);
                result = func.execute_noeval(&[result.clone(), e.clone()].to_vec(), env)?;
            }
            return Ok(result);
        } else {
            Err(create_error!("E1005"))
        }
    } else {
        Err(create_error!("E1006"))
    }
}
fn for_each(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Function(mut rc) = eval(&exp[1], env)? {
        if let Expression::List(l) = eval(&exp[2], env)? {
            for e in l {
                let func = Rc::make_mut(&mut rc);
                func.execute(&[Expression::Nil(), e.clone()].to_vec(), env)?;
            }
        } else {
            return Err(create_error!("E1005"));
        }
    } else {
        return Err(create_error!("E1006"));
    }
    Ok(Expression::Nil())
}
fn rand_integer(exp: &[Expression], _env: &mut SimpleEnv) -> ResultExpression {
    if 1 < exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    return Ok(Expression::Integer(x.abs() / SAMPLE_INT));
}
fn rand_list(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if 2 < exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    if let Expression::Integer(i) = eval(&exp[1], env)? {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::new();
        for _ in 0..i {
            let x: i64 = rng.gen();
            vec.push(Expression::Integer(x.abs() / SAMPLE_INT));
        }
        return Ok(Expression::List(vec));
    } else {
        return Err(create_error!("E1002"));
    }
}
fn load_file(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::String(s) = v {
        if false == Path::new(&s).exists() {
            return Err(create_error!("E1014"));
        }
        let file = match File::open(s) {
            Err(e) => return Err(create_error_value!("E1014", e)),
            Ok(file) => file,
        };
        let meta = match file.metadata() {
            Err(e) => return Err(create_error_value!("E9999", e)),
            Ok(meta) => meta,
        };
        if true == meta.is_dir() {
            return Err(create_error!("E1016"));
        }
        let mut stream = BufReader::new(file);
        match repl(&mut stream, env, true) {
            Err(e) => return Err(create_error_value!("E9999", e)),
            Ok(_) => return Ok(Expression::Nil()),
        }
    }
    return Err(create_error!("E1015"));
}
fn display(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 2 {
        return Ok(Expression::Nil());
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        print!("{} ", o.value_string());
    }
    return Ok(Expression::Nil());
}

fn delay(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    Ok(Expression::Promise(Box::new(exp[1].clone()), env.clone()))
}
fn force(exp: &[Expression], env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::Promise(p, mut pe) = v {
        return eval(&(*p), &mut pe);
    } else {
        return Ok(v);
    }
}

fn to_f64(exp: &[Expression], env: &mut SimpleEnv) -> Result<f64, RsError> {
    if exp.len() != 2 {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let v = eval(&exp[1], env)?;
    match v {
        Expression::Float(f) => return Ok(f),
        Expression::Integer(i) => return Ok(i as f64),
        _ => return Err(create_error!("E1003")),
    }
}
fn calc(
    exp: &[Expression],
    env: &mut SimpleEnv,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {
    let mut result: Number = Number::Integer(0);
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        let param = match o {
            Expression::Float(v) => Number::Float(v),
            _ => match o {
                Expression::Integer(v) => Number::Integer(v),
                _ => {
                    return Err(create_error!("E1003"));
                }
            },
        };
        if first == true {
            result = param;
            first = false;
            continue;
        }
        result = f(result, param);
    }
    match result {
        Number::Integer(a) => Ok(Expression::Integer(a)),
        Number::Float(a) => Ok(Expression::Float(a)),
    }
}
fn op(
    exp: &[Expression],
    env: &mut SimpleEnv,
    f: fn(x: &Number, y: &Number) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error_value!("E1007", exp.len()));
    }
    let mut vec: Vec<Number> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;

        match o {
            Expression::Float(f) => vec.push(Number::Float(f)),
            Expression::Integer(i) => vec.push(Number::Integer(i)),
            _ => return Err(create_error!("E1003")),
        }
    }
    return Ok(Expression::Boolean(f(&vec[0], &vec[1])));
}
pub fn do_interactive() {
    let mut env = SimpleEnv::new();
    let mut stream = BufReader::new(std::io::stdin());

    match repl(&mut stream, &mut env, false) {
        Err(e) => println!("{}", e),
        Ok(_) => {}
    }
}
fn repl(
    stream: &mut BufRead,
    env: &mut SimpleEnv,
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
        //do_core_logic(program.iter().cloned().collect::<String>());
        match do_core_logic(program.join(" "), env) {
            Ok(n) => println!("{}", n.value_string()),
            Err(e) => print_error!(e),
        }
        // for error_handle
        env.cleanup();
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
pub fn do_core_logic(program: String, env: &mut SimpleEnv) -> ResultExpression {
    let token = tokenize(program);

    let mut c: i32 = 1;
    let exp = parse(&token, &mut c)?;

    return eval(&exp, env);
}
fn tokenize(program: String) -> Vec<String> {
    let mut token: Vec<String> = Vec::new();
    let mut string_mode = false;
    let mut symbol_name = String::new();
    let mut from = 0;
    let mut i = 0;

    let mut s = String::from(program);
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
        Err(create_error!("E0002"))
    } else {
        let exp = atom(&token);
        Ok(exp)
    }
}
fn atom(token: &String) -> Expression {
    if let Ok(n) = token.parse::<i64>() {
        return Expression::Integer(n);
    }
    if let Ok(n) = token.parse::<f64>() {
        return Expression::Float(n);
    }
    if token.as_str() == "#t" {
        return Expression::Boolean(true);
    }
    if token.as_str() == "#f" {
        return Expression::Boolean(false);
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
    return Expression::Symbol(token.to_string());
}
macro_rules! ret_clone_if_atom {
    ($e: expr) => {
        match $e {
            Expression::Boolean(v) => return Ok(Expression::Boolean(*v)),
            Expression::Char(v) => return Ok(Expression::Char(*v)),
            Expression::String(_) => return Ok($e.clone()),
            Expression::Integer(v) => return Ok(Expression::Integer(*v)),
            Expression::Float(v) => return Ok(Expression::Float(*v)),
            Expression::Nil() => return Ok(Expression::Nil()),
            Expression::Pair(_, _) => return Ok($e.clone()),
            Expression::Promise(_, _) => return Ok($e.clone()),
            _ => {}
        }
    };
}
pub fn eval(sexp: &Expression, env: &mut SimpleEnv) -> ResultExpression {
    ret_clone_if_atom!(sexp);
    if let Expression::Symbol(val) = sexp {
        match env.find(&val) {
            Some(v) => {
                ret_clone_if_atom!(v);
                match v {
                    Expression::Function(_) => return Ok(v.clone()),
                    Expression::TailRecursion(_) => return Ok(v.clone()),
                    Expression::LetLoop(_) => return Ok(v.clone()),
                    Expression::List(_) => return Ok(v.clone()),
                    _ => {}
                }
            }
            None => {}
        }
        if let Some(f) = env.builtin_tbl.get(val.as_str()) {
            return Ok(Expression::BuildInFunction(*f));
        }
        if let Some(f) = env.builtin_tbl_ext.get(val.as_str()) {
            return Ok(Expression::BuildInFunctionExt(f.clone()));
        }
        return Err(create_error_value!("E1008", val));
    }
    if let Expression::List(l) = sexp {
        if l.len() == 0 {
            return Ok(sexp.clone());
        }
        let v = &l;
        let e = eval(&v[0], env)?;
        match e {
            Expression::LetLoop(f) => return f.execute(v, env),
            Expression::Function(mut rc) => {
                let f = Rc::make_mut(&mut rc);
                let result = f.execute(v, env);
                if let Expression::Symbol(s) = &v[0] {
                    // For ex. (define (counter) (let ((c 0)) (lambda () (set! c (+ 1 c)) c)))
                    env.update(s, Expression::Function(rc));
                }
                return result;
            }
            Expression::TailRecursion(f) => return f.set_param(v, env),
            Expression::BuildInFunction(f) => return f(&v[..], env),
            Expression::BuildInFunctionExt(f) => return (*f)(&v[..], env),
            _ => return Err(create_error!("E1006")),
        }
    }
    Err(create_error!("E1009"))
}
