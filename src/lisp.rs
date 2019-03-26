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

use crate::lisp::DataType::RsBooleanDesc;
use crate::lisp::DataType::RsCharDesc;
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
    RsCharDesc,
    RsBooleanDesc,
    RsListDesc,
    RsSymbolDesc,
    RsFunctionDesc,
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
pub struct RsFunction {
    type_id: DataType,
    param: RsList,
    body: RsList,
    name: String,
}
impl RsFunction {
    fn new(sexp: &Vec<PtrExpression>, _name: String) -> RsFunction {
        let mut _param = RsList::new();
        let mut _body = RsList::new();

        if let Some(val) = sexp[1].as_any().downcast_ref::<RsList>() {
            for n in &val.value[..] {
                _param.value.push(Box::new(RsSymbol::new(n.value_string())));
            }
        }
        if let Some(val) = sexp[2].as_any().downcast_ref::<RsList>() {
            _body.value.extend_from_slice(&val.value[..]);
        }
        RsFunction {
            type_id: RsFunctionDesc,
            param: _param,
            body: _body,
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
                    Err(e) => return Err(e),
                }
            }
            i += 1;
        }
        let list = Box::new(self.body.clone()) as PtrExpression;
        let result = eval(&list, env);
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
        b.insert("+", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: i64, y: i64| x + y)
        });
        b.insert("-", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: i64, y: i64| x - y)
        });
        b.insert("*", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: i64, y: i64| x * y)
        });
        b.insert("/", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: i64, y: i64| x / y)
        });

        b.insert("define", define);
        b.insert("lambda", lambda);
        b.insert("if", if_f);

        b.insert("=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: i64, y: i64| x == y)
        });
        b.insert("<", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: i64, y: i64| x < y)
        });
        b.insert(">", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: i64, y: i64| x > y)
        });
        b.insert("<=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: i64, y: i64| x <= y)
        });
        b.insert(">=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: i64, y: i64| x >= y)
        });

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
    match (exp[1]).type_id() {
        RsListDesc => {}
        _ => return Err(create_error!("E1005")),
    }
    match (exp[2]).type_id() {
        RsListDesc => {}
        _ => return Err(create_error!("E1005")),
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
    f: fn(x: i64, y: i64) -> i64,
) -> ResultExpression {
    let mut result: i64 = 0;
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error!("E1007"));
    }
    for n in &exp[1 as usize..] {
        let o = eval(n, env)?;
        if let Some(v) = o.as_any().downcast_ref::<RsInteger>() {
            if first == true {
                result = v.value;
                first = false;
            } else {
                result = f(result, v.value);
            }
        } else {
            return Err(create_error!("E1003"));
        }
    }
    return Ok(Box::new(RsInteger::new(result)));
}
fn op(
    exp: &Vec<PtrExpression>,
    env: &mut SimpleEnv,
    f: fn(x: i64, y: i64) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error!("E1007"));
    }
    let a = eval(&exp[1], env)?;
    let b = eval(&exp[2], env)?;

    if let Some(x) = a.as_any().downcast_ref::<RsInteger>() {
        if let Some(y) = b.as_any().downcast_ref::<RsInteger>() {
            return Ok(Box::new(RsBoolean::new(f(x.value, y.value))));
        }
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
    match token.parse::<i64>() {
        Ok(n) => return Box::new(RsInteger::new(n)),
        _ => {
            if token.as_str() == "#t" {
                return Box::new(RsBoolean::new(true));
            }
            if token.as_str() == "#f" {
                return Box::new(RsBoolean::new(false));
            }
            if (token.len() == 3) && (&token.as_str()[0..1] == "#\\") {
                let c = token.chars().collect::<Vec<char>>();
                return Box::new(RsChar::new(c[2]));
            }
            return Box::new(RsSymbol::new(token.to_string()));
        }
    }
}
fn eval(sexp: &PtrExpression, env: &mut SimpleEnv) -> ResultExpression {
    if let Some(val) = (*sexp).as_any().downcast_ref::<RsInteger>() {
        return Ok(Box::new(val.clone()));
    }
    if let Some(val) = (*sexp).as_any().downcast_ref::<RsChar>() {
        return Ok(Box::new(val.clone()));
    }
    if let Some(val) = (*sexp).as_any().downcast_ref::<RsBoolean>() {
        return Ok(Box::new(val.clone()));
    }
    if let Some(val) = (*sexp).as_any().downcast_ref::<RsSymbol>() {
        match env.find(val.value.to_string()) {
            Some(v) => {
                if let Some(val) = v.as_any().downcast_ref::<RsInteger>() {
                    return Ok(Box::new(val.clone()));
                }
                if let Some(val) = v.as_any().downcast_ref::<RsFunction>() {
                    return Ok(Box::new(val.clone()));
                }
            }
            None => {}
        }
        return Err(create_error!("E1008"));
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
            if let Some(exp) = env.find(sym.value.to_string()) {
                if let Some(f) = exp.as_any().downcast_ref::<RsFunction>() {
                    let mut func = f.clone();
                    return func.execute(v, env);
                }
            }
        }
    }
    Err(create_error!("E1009"))
}
