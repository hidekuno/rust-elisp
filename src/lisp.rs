/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::vec::Vec;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::any::Any;

use crate::lisp::DataType::RsIntegerDesc;
use crate::lisp::DataType::RsCharDesc;
use crate::lisp::DataType::RsListDesc;
use crate::lisp::DataType::RsSymbolDesc;
//========================================================================
lazy_static! {
    static ref ERRMSG_TBL: HashMap<&'static str, &'static str> = {
        let mut e: HashMap<&'static str, &'static str> = HashMap::new();
		e.insert("E0001","Unexpected EOF while reading");
		e.insert("E0002","Unexpected ')' while reading");
		e.insert("E0003","Extra close parenthesis `)'");
		e.insert("E0004","Charactor syntax error");
		e.insert("E1001","Not Boolean");
		e.insert("E1002","Not Integer");
		e.insert("E1003","Not Number");
		e.insert("E1004","Not Symbol");
		e.insert("E1005","Not List");
		e.insert("E1006","Not Function");
		e.insert("E1007","Not Enough Parameter Counts");
		e.insert("E1008","Undefine variable");
		e.insert("E1009","Not Enough Data Type");
		e.insert("E1010","Not Promise");
		e.insert("E1011","Not Enough List Length");
		e.insert("E1012","Not Cond Gramar");
		e.insert("E1013","Calculate A Division By Zero");
		e.insert("E1014","Not Found Program File");
		e.insert("E1015","Not String");
		e.insert("E9999","System Panic");

        e
	};
}
//========================================================================
pub enum DataType {
    RsIntegerDesc,
    RsCharDesc,
    RsListDesc,
    RsSymbolDesc,
}
pub trait Expression {
    fn type_id(&self) -> &DataType;
    fn value_string(&self) -> String;
    fn as_any(&self) -> &Any;
}
pub struct RsInteger {
    type_id: DataType,
    value: i64
}
impl RsInteger {
    fn new(p: i64) -> RsInteger {
        RsInteger{type_id: RsIntegerDesc, value:p}
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
pub struct RsChar {
    type_id: DataType,
    value: char
}
impl RsChar {
    fn new(p: char) -> RsChar {
        RsChar{type_id:RsCharDesc, value:p}
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
pub struct RsList {
    type_id: DataType,
    value: Vec<Box<Expression>>
}
impl RsList {
    fn new() -> RsList {
        RsList{type_id:RsListDesc, value:Vec::new()}
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
pub struct RsSymbol {
    type_id: DataType,
    value: String
}
impl RsSymbol {
    fn new(p: String) -> RsSymbol {
        RsSymbol{type_id:RsSymbolDesc, value:p}
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

pub struct SimpleEnv {
    env_tbl:     LinkedList<HashMap<String,Box<Expression>>>,
    builtin_tbl: HashMap<&'static str,fn(&Vec<Box<Expression>>, &mut SimpleEnv) -> Result<Box<Expression>, &'static str>>
}

impl SimpleEnv {
    fn new() -> SimpleEnv {
        let mut e: HashMap<String,Box<Expression>> = HashMap::new();
        let mut l: LinkedList<HashMap<String,Box<Expression>>> = LinkedList::new();
        l.push_back(e);

        let mut b: HashMap<&'static str, fn(&Vec<Box<Expression>>, &mut SimpleEnv) -> Result<Box<Expression>, &'static str>> = HashMap::new();
        b.insert("+", |exp: &Vec<Box<Expression>>,env: &mut SimpleEnv| calc(exp, env, |x: i64, y: i64| x + y));
        b.insert("-", |exp: &Vec<Box<Expression>>,env: &mut SimpleEnv| calc(exp, env, |x: i64, y: i64| x - y));
        b.insert("*", |exp: &Vec<Box<Expression>>,env: &mut SimpleEnv| calc(exp, env, |x: i64, y: i64| x * y));
        b.insert("/", |exp: &Vec<Box<Expression>>,env: &mut SimpleEnv| calc(exp, env, |x: i64, y: i64| x / y));

        b.insert("define", define);
        SimpleEnv {env_tbl: l, builtin_tbl: b}
    }
    fn create(&mut self) {
        let mut e: HashMap<String,Box<Expression>> = HashMap::new();
        self.env_tbl.push_front(e);
    }
    fn regist(&mut self, key: String, exp: Box<Expression>) {
        match self.env_tbl.front_mut() {
            Some(m) => { m.insert(key, exp); },
            None => {},
        }
    }
    fn find(&self, key: String) -> Option<&Box<Expression>> {
        for exp in self.env_tbl.iter() {
            match exp.get(&key) {
                Some(v) => {
                    return Some(v);
                },
                None => {},
            }
        }
        None
    }
}
//========================================================================
const PROMPT: &str = "<rust.elisp> ";
const QUIT: &str = "(quit)";
//========================================================================
fn define(exp: &Vec<Box<Expression>>, env: &mut SimpleEnv) -> Result<Box<Expression>, &'static str> {

    if let Some(v) = exp[1].as_any().downcast_ref::<RsSymbol>() {
        match eval(&exp[2],env) {
            Ok(se) => {
                env.regist(v.value.to_string(), se);
                return Ok(Box::new(RsSymbol::new(v.value.to_string())));
            },
            Err(e) => {return Err(e);},
        }
    }
    Err("E1004")
}
fn calc(exp: &Vec<Box<Expression>>, env: &mut SimpleEnv, f: fn(x:i64, y:i64)->i64)
        -> Result<Box<Expression>, &'static str> {
    let mut result: i64 = 0;
    let mut first: bool = true;

    for n in &exp[1 as usize..] {
        match eval(n,env) {
            Ok(o)  => {
                if let Some(v) = o.as_any().downcast_ref::<RsInteger>() {
                    if first == true {
                        result = v.value;
                        first = false;
                    } else {
                        result = f(result, v.value);
                    }
                }
            },
            Err(e) => { },
        }
    }
    return Ok(Box::new(RsInteger::new(result)));
}
pub fn do_interactive() {
    let mut env = SimpleEnv::new();

    let mut stream =  BufReader::new(std::io::stdin());
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
        do_core_logic(program.join(" "), env);
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
fn do_core_logic(program: String, env: &mut SimpleEnv) {

    let token = tokenize(program);

    let mut c: i32 = 1;
    match parse(&token, &mut c) {
        Ok(exp)  => {
            match eval(&exp, env) {
                Ok(n)  => {println!("{}",n.value_string());},
                Err(e) => {println!("{}",ERRMSG_TBL.get(e).unwrap()); },
            }
        },
        Err(e) => {
            println!("{}",ERRMSG_TBL.get(e).unwrap());
        },
    }
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
                if vc[i-1] != 0x5c {
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
                    match vc[i+1] {
                        0x28|0x29|0x20 => {token.push(String::from(symbol_name.as_str())); symbol_name.clear();}
                        _ => {}
                    }
                }
            }
        }
        i+=1;
    }
    return token;
}
fn parse(tokens: &Vec<String>, count: &mut i32) -> Result<Box<Expression>, &'static str> {

    if tokens.len() == 0 {
        return Err("E0001");
    }

    let token = &tokens[0];
    if "(" == token {
        if tokens.len() <= 1 {
            return Err("E0001");
        }
        let mut list = RsList::new();

        *count = 1;
        loop {
            if tokens[*count as usize] == ")" {
                *count += 1;
                break;
            }
            let mut c: i32 =  1;
            match parse(&tokens[*count as usize..].to_vec(), &mut c) {
                Ok(n)  => list.value.push(n),
                Err(e) => return Err(e),
            }
            *count += c;
            if tokens.len() <= *count as usize {
                return Err("E0002");
            }
        }
        Ok(Box::new(list))

    } else if ")" == token {
        Err("E0002")
    } else {
        let exp = atom(&token);
        Ok(exp)
    }
}
fn atom(token: &String) -> Box<Expression> {
    match token.parse() {
        Ok(n)  => return Box::new(RsInteger::new(n)),
        Err(e) => return Box::new(RsSymbol::new(token.to_string())),
    }
}
fn eval(sexp: &Box<Expression>, env: &mut SimpleEnv) -> Result<Box<Expression>, &'static str> {

    if let Some(val) = (*sexp).as_any().downcast_ref::<RsInteger>() {
        return Ok(Box::new(RsInteger::new(val.value)));
    }
    if let Some(val) = (*sexp).as_any().downcast_ref::<RsChar>() {
        return Ok(Box::new(RsChar::new(val.value)));
    }

    if let Some(val) = (*sexp).as_any().downcast_ref::<RsSymbol>() {

//        match env.find(val.value.to_string()) {
//            Some(v) => return Ok(*v),
//            None    => return Err("E1008"),
//        }
    }

    if let Some(l) = (*sexp).as_any().downcast_ref::<RsList>() {
        if l.value.len() == 0 {
            return Ok(Box::new(RsList::new()));
        }
        let v = &l.value;
        if let Some(sym) = v[0].as_any().downcast_ref::<RsSymbol>() {
            if let Some(f) = env.builtin_tbl.get(&sym.value.as_str()) {
                return f(v, env);
            }
        }
    }
    Err("E1008")
}
