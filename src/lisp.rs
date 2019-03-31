/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::any::Any;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::vec::Vec;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::cmp::Ordering;

use crate::lisp::DataType::RsBooleanDesc;
use crate::lisp::DataType::RsBuildInFunctionDesc;
use crate::lisp::DataType::RsCharDesc;
use crate::lisp::DataType::RsFloatDesc;
use crate::lisp::DataType::RsFunctionDesc;
use crate::lisp::DataType::RsIntegerDesc;
use crate::lisp::DataType::RsListDesc;
use crate::lisp::DataType::RsSymbolDesc;
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
        e.insert("E9999", "System Panic");
        e
    };
}
pub struct RsError {
    code: &'static str,
    line: u32,
    file: &'static str,
}
impl RsError {
    pub fn get_code(&self) -> String {
        String::from(self.code)
    }
}
macro_rules! create_error {
    ($e: expr) => {
        RsError {
            code: $e,
            line: line!(),
            file: file!(),
        }
    };
}
macro_rules! print_error {
    ($e: expr) => {
        println!(
            "{} ({}:{})",
            ERRMSG_TBL.get($e.code).unwrap(),
            $e.file,
            $e.line
        )
    };
}
//========================================================================
type PtrExpression = Box<Expression>;
type ResultExpression = Result<PtrExpression, RsError>;
//========================================================================
#[derive(Copy, Clone)]
pub enum DataType {
    RsIntegerDesc,
    RsFloatDesc,
    RsCharDesc,
    RsBooleanDesc,
    RsListDesc,
    RsSymbolDesc,
    RsFunctionDesc,
    RsBuildInFunctionDesc,
}
pub trait Expression: ExpressionClone {
    fn type_id(&self) -> &DataType;
    fn value_string(&self) -> String;
    fn as_any(&self) -> &Any;
}
pub trait ExpressionClone {
    fn clone_box(&self) -> PtrExpression;
}
impl<T: 'static + Expression + Clone> ExpressionClone for T {
    fn clone_box(&self) -> PtrExpression {
        Box::new(self.clone())
    }
}
impl Clone for PtrExpression {
    fn clone(&self) -> PtrExpression {
        self.clone_box()
    }
}

#[derive(Copy, Clone)]
pub struct RsInteger {
    type_id: DataType,
    value: i64,
}
impl RsInteger {
    fn new(p: i64) -> RsInteger {
        RsInteger {
            type_id: RsIntegerDesc,
            value: p,
        }
    }
}
impl Expression for RsInteger {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsFloat {
    type_id: DataType,
    value: f64,
}
impl RsFloat {
    fn new(p: f64) -> RsFloat {
        RsFloat {
            type_id: RsFloatDesc,
            value: p,
        }
    }
}
impl Expression for RsFloat {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsBoolean {
    type_id: DataType,
    value: bool,
}
impl RsBoolean {
    fn new(p: bool) -> RsBoolean {
        RsBoolean {
            type_id: RsBooleanDesc,
            value: p,
        }
    }
}
impl Expression for RsBoolean {
    fn value_string(&self) -> String {
        let mut b: String = String::from("#f");
        if self.value == true {
            b = String::from("#t");
        }
        b
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsChar {
    type_id: DataType,
    value: char,
}
impl RsChar {
    fn new(p: char) -> RsChar {
        RsChar {
            type_id: RsCharDesc,
            value: p,
        }
    }
}
impl Expression for RsChar {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsSymbol {
    type_id: DataType,
    value: String,
}
impl RsSymbol {
    fn new(p: String) -> RsSymbol {
        RsSymbol {
            type_id: RsSymbolDesc,
            value: p,
        }
    }
}
impl Expression for RsSymbol {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsList {
    type_id: DataType,
    value: Vec<PtrExpression>,
}
impl RsList {
    fn new() -> RsList {
        RsList {
            type_id: RsListDesc,
            value: Vec::new(),
        }
    }
}
impl Expression for RsList {
    fn value_string(&self) -> String {
        "List".to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Clone)]
pub struct RsBuildInFunction {
    type_id: DataType,
    name: String,
    func: fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression,
}
impl RsBuildInFunction {
    fn new(
        _func: fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression,
        _name: String,
    ) -> RsBuildInFunction {
        RsBuildInFunction {
            type_id: RsBuildInFunctionDesc,
            func: _func,
            name: _name,
        }
    }
    fn execute(&mut self, exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
        let f = self.func;
        return f(exp, env);
    }
}
impl Expression for RsBuildInFunction {
    fn value_string(&self) -> String {
        "BuildIn Function".to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsFunction {
    type_id: DataType,
    param: RsList,
    body: PtrExpression,
    name: String,
}
impl RsFunction {
    fn new(sexp: &Vec<PtrExpression>, _name: String) -> RsFunction {
        let mut _param = RsList::new();

        if let Some(val) = sexp[1].as_any().downcast_ref::<RsList>() {
            for n in &val.value[..] {
                _param.value.push(Box::new(RsSymbol::new(n.value_string())));
            }
        }
        RsFunction {
            type_id: RsFunctionDesc,
            param: _param,
            body: sexp[2].clone(),
            name: _name,
        }
    }
    fn execute(&mut self, exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
        // Bind lambda function' parameters.
        if self.param.value.len() != (exp.len() - 1) {
            return Err(create_error!("E1007"));
        }
        env.create();
        let mut i = 1;
        for p in &self.param.value[..] {
            if let Some(s) = p.as_any().downcast_ref::<RsSymbol>() {
                match eval(&exp[i].clone(), env) {
                    Ok(result) => env.regist(s.value.to_string(), result),
                    Err(e) => {
                        env.delete();
                        return Err(e);
                    }
                }
            }
            i += 1;
        }
        let result = eval(&self.body, env);
        env.delete();
        return result;
    }
}
impl Expression for RsFunction {
    fn value_string(&self) -> String {
        "Function".to_string()
    }
    fn type_id(&self) -> &DataType {
        &self.type_id
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct Number {
    value: PtrExpression
}
impl Number {
    fn calc_template(self,
                     other: Number,
                     fcalc: fn (x: f64, y: f64) -> f64,
                     icalc: fn (x: i64, y: i64) -> i64,
    ) -> Number {
        if let Some(sf) = self.value.as_any().downcast_ref::<RsFloat>() {
            if let Some(f) = other.value.as_any().downcast_ref::<RsFloat>() {
                return Number{value: Box::new(RsFloat::new(fcalc(sf.value, f.value)))};
            }

        }
        if let Some(sf) = self.value.as_any().downcast_ref::<RsInteger>() {
            if let Some(i) = other.value.as_any().downcast_ref::<RsInteger>() {
                return Number{value: Box::new(RsInteger::new(icalc(sf.value, i.value)))};
            }
        }
        self
    }
    fn cmp_template(&self,
                    other: &Number,
                    fop: fn (x: f64, y: f64) -> bool,
                    iop: fn (x: i64, y: i64) -> bool
    ) -> bool {
        if let Some(sf) = self.value.as_any().downcast_ref::<RsFloat>() {
            if let Some(f) = other.value.as_any().downcast_ref::<RsFloat>() {
                return fop(sf.value, f.value);
            }
        }
        else if let Some(sf) = self.value.as_any().downcast_ref::<RsInteger>() {
            if let Some(i) = other.value.as_any().downcast_ref::<RsInteger>() {
                return iop(sf.value, i.value);
            }
        }
        true
    }
}
//impl<T: Add<Output=T>> Add for Number<T> {
impl Add for Number {
    type Output = Number;
    fn add(self, other: Number) -> Number {
        return self.calc_template(other, |x:f64, y:f64| x + y,|x:i64, y:i64| x + y);
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        return self.calc_template(other, |x:f64, y:f64| x - y,|x:i64, y:i64| x - y);
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        return self.calc_template(other, |x:f64, y:f64| x * y,|x:i64, y:i64| x * y);
    }
}
impl Div for Number {
    type Output = Number;
    fn div(self, other: Number) -> Number {
        return self.calc_template(other, |x:f64, y:f64| x * y,|x:i64, y:i64| x * y);
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        return self.cmp_template(other, |x:f64, y:f64| x == y,|x:i64, y:i64| x == y);
    }
}
impl PartialOrd for Number {
    fn lt(&self, other: &Number) -> bool {
        return self.cmp_template(other, |x:f64, y:f64| x < y,|x:i64, y:i64| x < y);
    }
    fn le(&self, other: &Number) -> bool {
        return self.cmp_template(other, |x:f64, y:f64| x <= y,|x:i64, y:i64| x <= y);
    }
    fn gt(&self, other: &Number) -> bool {
        return self.cmp_template(other, |x:f64, y:f64| x > y,|x:i64, y:i64| x > y);
    }
    fn ge(&self, other: &Number) -> bool {
        return self.cmp_template(other, |x:f64, y:f64| x >= y,|x:i64, y:i64| x >= y);
    }
    fn partial_cmp(&self, _: &Number) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}
pub struct SimpleEnv {
    env_tbl: LinkedList<HashMap<String, PtrExpression>>,
    builtin_tbl: HashMap<&'static str, fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression>,
}
impl SimpleEnv {
    pub fn new() -> SimpleEnv {
        let mut l: LinkedList<HashMap<String, PtrExpression>> = LinkedList::new();
        l.push_back(HashMap::new());

        let mut b: HashMap<
            &'static str,
            fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression,
        > = HashMap::new();
        b.insert("+", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{calc(exp,env,|x: Number, y: Number| x + y)});
        b.insert("-", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{calc(exp,env,|x: Number, y: Number| x - y)});
        b.insert("*", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{calc(exp,env,|x: Number, y: Number| x * y)});
        b.insert("/", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{calc(exp,env,|x: Number, y: Number| x / y)});
        b.insert("=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{op(exp,env,|x: &Number, y: &Number| x == y)});
        b.insert("<", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{op(exp,env,|x: &Number, y: &Number| x < y)});
        b.insert("<=",|exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{op(exp,env,|x: &Number, y: &Number| x <= y)});
        b.insert(">", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{op(exp,env,|x: &Number, y: &Number| x > y)});
        b.insert(">=",|exp: &Vec<PtrExpression>, env: &mut SimpleEnv|{op(exp,env,|x: &Number, y: &Number| x >= y)});

        b.insert("define", define);
        b.insert("lambda", lambda);
        b.insert("if", if_f);

        SimpleEnv {
            env_tbl: l,
            builtin_tbl: b,
        }
    }
    fn create(&mut self) {
        self.env_tbl.push_front(HashMap::new());
    }
    fn delete(&mut self) {
        self.env_tbl.pop_front();
    }
    fn regist(&mut self, key: String, exp: PtrExpression) {
        match self.env_tbl.front_mut() {
            Some(m) => {
                m.insert(key, exp);
            }
            None => {}
        }
    }
    fn find(&self, key: String) -> Option<&PtrExpression> {
        for exp in self.env_tbl.iter() {
            match exp.get(&key) {
                Some(v) => {
                    return Some(v);
                }
                None => {}
            }
        }
        None
    }
}
//========================================================================
const PROMPT: &str = "<rust.elisp> ";
const QUIT: &str = "(quit)";
//========================================================================
fn lambda(exp: &Vec<PtrExpression>, _env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error!("E1007"));
    }
    if let Some(l) = exp[1].as_any().downcast_ref::<RsList>() {
        for e in &l.value {
            match e.type_id() {
                RsSymbolDesc => {}
                _ => return Err(create_error!("E1004")),
            }
        }
    } else {
        return Err(create_error!("E1005"));
    }
    return Ok(Box::new(RsFunction::new(exp, String::from("lambda"))));
}
fn define(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error!("E1007"));
    }
    if let Some(v) = exp[1].as_any().downcast_ref::<RsSymbol>() {
        let se = eval(&exp[2], env)?;
        env.regist(v.value.to_string(), se);
        return Ok(Box::new(v.clone()));
    }
    if let Some(l) = exp[1].as_any().downcast_ref::<RsList>() {
        if l.value.len() < 1 {
            return Err(create_error!("E1007"));
        }
        if let Some(s) = l.value[0].as_any().downcast_ref::<RsSymbol>() {
            let mut f = exp.clone();
            let mut param = RsList::new();
            for n in &l.value[1..] {
                match (*n).type_id() {
                    RsSymbolDesc => {}
                    _ => return Err(create_error!("E1004")),
                }
                param.value.push((*n).clone());
            }
            f[1] = Box::new(param);
            env.regist(
                s.value.to_string(),
                Box::new(RsFunction::new(&f, s.value.to_string())),
            );
            return Ok(Box::new(s.clone()));
        } else {
            return Err(create_error!("E1004"));
        }
    }
    Err(create_error!("E1004"))
}
fn if_f(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error!("E1004"));
    }
    let se = eval(&exp[1], env)?;

    if let Some(b) = se.as_any().downcast_ref::<RsBoolean>() {
        if b.value == true {
            return eval(&exp[2], env);
        } else {
            return eval(&exp[3], env);
        }
    }
    return Err(create_error!("E1001"));
}
fn calc(
    exp: &Vec<PtrExpression>,
    env: &mut SimpleEnv,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {

    let mut result = Number{value: Box::new(RsInteger::new(0))};
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error!("E1007"));
    }
    for n in &exp[1 as usize..] {
        let o = eval(n, env)?;
        let mut param  = Number{value: o.clone_box()};
        if first == true {
            result = param;
            first = false;
            continue;
        }
        if let Some(_) = o.as_any().downcast_ref::<RsFloat>() {
            if let Some(sf) = result.value.as_any().downcast_ref::<RsInteger>() {
                result =  Number{value: Box::new(RsFloat::new(sf.value as f64))};
            }
        } else if let Some(i) = o.as_any().downcast_ref::<RsInteger>() {
            if let Some(_) = result.value.as_any().downcast_ref::<RsFloat>() {
                param = Number{value: Box::new(RsFloat::new(i.value  as f64))};
            }
        } else {
            return Err(create_error!("E1003"));
        }
        result = f(result, param);
    }
    return Ok(result.value.clone_box());
}
fn op(
    exp: &Vec<PtrExpression>,
    env: &mut SimpleEnv,
    f: fn(x: &Number, y: &Number) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error!("E1007"));
    }
    let a = eval(&exp[1], env)?;
    let b = eval(&exp[2], env)?;

    if let Some(_) = a.as_any().downcast_ref::<RsFloat>() {
        let x = Number{value: a.clone_box()};
        if let Some(i) = b.as_any().downcast_ref::<RsInteger>() {
            let y = Number{value: Box::new(RsFloat::new(i.value as f64))};
            return Ok(Box::new(RsBoolean::new(f(&x, &y))));
        }
        let y = Number{value: b.clone_box()};
        return Ok(Box::new(RsBoolean::new(f(&x, &y))));
    }
    if let Some(i) = a.as_any().downcast_ref::<RsInteger>() {
        let y = Number{value: b.clone_box()};

        if let Some(_) = b.as_any().downcast_ref::<RsFloat>() {
            let x = Number{value: Box::new(RsFloat::new(i.value as f64))};
            return Ok(Box::new(RsBoolean::new(f(&x, &y))));
        }
        let x = Number{value: a.clone_box()};
        return Ok(Box::new(RsBoolean::new(f(&x, &y))));
    }
    return Err(create_error!("E1003"));
}
pub fn do_interactive() {
    let mut env = SimpleEnv::new();

    let mut stream = BufReader::new(std::io::stdin());
    repl(&mut stream, &mut env);
}
fn repl(stream: &mut BufRead, env: &mut SimpleEnv) {
    let mut buffer = String::new();
    let mut program: Vec<String> = Vec::new();
    let mut w = std::io::stdout();
    let mut prompt = PROMPT;

    loop {
        print!("{}", prompt);
        w.flush().unwrap();
        buffer.clear();
        stream.read_line(&mut buffer).unwrap();

        if buffer.trim() == QUIT {
            println!("Bye");
            break;
        } else if buffer.trim() == "" {
            continue;
        } else if buffer.trim() == ";" {
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
        program.clear();
        prompt = PROMPT;
    }
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

                if s.len() - 1 == i {
                    token.push(String::from(symbol_name.as_str()));
                } else {
                    match vc[i + 1] {
                        0x28 | 0x29 | 0x20 => {
                            token.push(String::from(symbol_name.as_str()));
                            symbol_name.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
        i += 1;
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
        let mut list = RsList::new();

        *count = 1;
        loop {
            if tokens[*count as usize] == ")" {
                *count += 1;
                break;
            }
            let mut c: i32 = 1;
            match parse(&tokens[*count as usize..].to_vec(), &mut c) {
                Ok(n) => list.value.push(n),
                Err(e) => return Err(e),
            }
            *count += c;
            if tokens.len() <= *count as usize {
                return Err(create_error!("E0002"));
            }
        }
        Ok(Box::new(list))
    } else if ")" == token {
        Err(create_error!("E0002"))
    } else {
        let exp = atom(&token);
        Ok(exp)
    }
}
fn atom(token: &String) -> PtrExpression {
    if let Ok(n) = token.parse::<i64>() {
        return Box::new(RsInteger::new(n));
    }
    if let Ok(n) = token.parse::<f64>() {
        return Box::new(RsFloat::new(n));
    }
    if token.as_str() == "#t" {
        return Box::new(RsBoolean::new(true));
    }
    if token.as_str() == "#f" {
        return Box::new(RsBoolean::new(false));
    }
    if (token.len() == 3) && (&token.as_str()[0..2] == "#\\") {
        let c = token.chars().collect::<Vec<char>>();
        return Box::new(RsChar::new(c[2]));
    }
    return Box::new(RsSymbol::new(token.to_string()));
}
macro_rules! ret_clone_if_atom {
    ($e: expr) => {
        match $e.type_id() {
            RsBooleanDesc | RsCharDesc | RsIntegerDesc | RsFloatDesc => {
                return Ok($e.clone_box());
            }
            _ => {}
        }
    };
}
fn eval(sexp: &PtrExpression, env: &mut SimpleEnv) -> ResultExpression {
    ret_clone_if_atom!(sexp);

    if let Some(val) = (*sexp).as_any().downcast_ref::<RsSymbol>() {
        match env.find(val.value.to_string()) {
            Some(v) => {
                ret_clone_if_atom!(v);
                if let Some(_) = v.as_any().downcast_ref::<RsFunction>() {
                    return Ok(v.clone_box());
                }
            }
            None => {}
        }
        if let Some(f) = env.builtin_tbl.get(val.value.as_str()) {
            return Ok(Box::new(RsBuildInFunction::new(*f, val.value.to_string())));
        }
        return Err(create_error!("E1008"));
    }
    if let Some(l) = (*sexp).as_any().downcast_ref::<RsList>() {
        if l.value.len() == 0 {
            return Ok(sexp.clone_box());
        }
        let v = &l.value;
        if let Some(_) = v[0].as_any().downcast_ref::<RsSymbol>() {
            let e = eval(&v[0], env)?;
            if let Some(f) = e.as_any().downcast_ref::<RsFunction>() {
                let mut func = f.clone();
                return func.execute(v, env);
            }
            if let Some(f) = e.as_any().downcast_ref::<RsBuildInFunction>() {
                let mut func = f.clone();
                return func.execute(v, env);
            }
        }
    }
    Err(create_error!("E1009"))
}
