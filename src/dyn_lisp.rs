/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::any::Any;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::time::Instant;
use std::vec::Vec;

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
    RsLetLoopDesc,
    RsNilDesc,
}
pub trait Expression: ExpressionClone {
    fn data_type(&self) -> &DataType;
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
    data_type: DataType,
    value: i64,
}
impl RsInteger {
    fn new(p: i64) -> RsInteger {
        RsInteger {
            data_type: DataType::RsIntegerDesc,
            value: p,
        }
    }
}
impl Expression for RsInteger {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsFloat {
    data_type: DataType,
    value: f64,
}
impl RsFloat {
    fn new(p: f64) -> RsFloat {
        RsFloat {
            data_type: DataType::RsFloatDesc,
            value: p,
        }
    }
}
impl Expression for RsFloat {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsBoolean {
    data_type: DataType,
    value: bool,
}
impl RsBoolean {
    fn new(p: bool) -> RsBoolean {
        RsBoolean {
            data_type: DataType::RsBooleanDesc,
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
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Copy, Clone)]
pub struct RsChar {
    data_type: DataType,
    value: char,
}
impl RsChar {
    fn new(p: char) -> RsChar {
        RsChar {
            data_type: DataType::RsCharDesc,
            value: p,
        }
    }
}
impl Expression for RsChar {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsSymbol {
    data_type: DataType,
    value: String,
}
impl RsSymbol {
    fn new(p: String) -> RsSymbol {
        RsSymbol {
            data_type: DataType::RsSymbolDesc,
            value: p,
        }
    }
}
impl Expression for RsSymbol {
    fn value_string(&self) -> String {
        self.value.to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsNil {
    data_type: DataType,
}
impl RsNil {
    fn new() -> RsNil {
        RsNil {
            data_type: DataType::RsNilDesc,
        }
    }
}
impl Expression for RsNil {
    fn value_string(&self) -> String {
        "nil".to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsList {
    data_type: DataType,
    value: Vec<PtrExpression>,
}
impl RsList {
    fn new() -> RsList {
        RsList {
            data_type: DataType::RsListDesc,
            value: Vec::new(),
        }
    }
}
impl Expression for RsList {
    fn value_string(&self) -> String {
        "List".to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Clone)]
pub struct RsBuildInFunction {
    data_type: DataType,
    name: String,
    func: fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression,
}
impl RsBuildInFunction {
    fn new(
        _func: fn(&Vec<PtrExpression>, &mut SimpleEnv) -> ResultExpression,
        _name: String,
    ) -> RsBuildInFunction {
        RsBuildInFunction {
            data_type: DataType::RsBuildInFunctionDesc,
            func: _func,
            name: _name,
        }
    }
    fn execute(&self, exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
        let f = self.func;
        return f(exp, env);
    }
}
impl Expression for RsBuildInFunction {
    fn value_string(&self) -> String {
        "BuildIn Function".to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsFunction {
    data_type: DataType,
    param: RsList,
    body: Vec<PtrExpression>,
    name: String,
    closure_env: LinkedList<HashMap<String, PtrExpression>>,
}
impl RsFunction {
    fn new(sexp: &Vec<PtrExpression>, _name: String) -> RsFunction {
        let mut _param = RsList::new();
        let l: LinkedList<HashMap<String, PtrExpression>> = LinkedList::new();

        if let Some(val) = sexp[1].as_any().downcast_ref::<RsList>() {
            for n in &val.value[..] {
                _param.value.push(Box::new(RsSymbol::new(n.value_string())));
            }
        }
        let mut vec: Vec<PtrExpression> = Vec::new();
        vec.extend_from_slice(&sexp[2..]);
        RsFunction {
            data_type: DataType::RsFunctionDesc,
            param: _param,
            body: vec,
            name: _name,
            closure_env: l,
        }
    }
    fn set_closure_env(&mut self, param: HashMap<String, PtrExpression>) {
        self.closure_env.push_back(param);
    }
    fn execute(&mut self, exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
        if self.param.value.len() != (exp.len() - 1) {
            return Err(create_error!("E1007"));
        }
        // param eval
        let mut vec: Vec<PtrExpression> = Vec::new();
        for e in &exp[1 as usize..] {
            let v = eval(e, env)?;
            vec.push(v);
        }
        // closure set
        for h in self.closure_env.iter() {
            env.create();
            for (k, v) in h {
                env.regist(k.to_string(), v.clone_box());
            }
        }
        // param set
        env.create();
        let mut idx = 0;
        for p in &self.param.value[..] {
            if let Some(s) = p.as_any().downcast_ref::<RsSymbol>() {
                env.regist(s.value.to_string(), vec[idx].clone_box());
            }
            idx += 1;
        }
        let mut results: Vec<ResultExpression> = Vec::new();
        for e in &self.body {
            results.push(eval(e, env));
        }
        // param clear
        env.delete();

        // clouser env clear
        let mut l: LinkedList<HashMap<String, PtrExpression>> = LinkedList::new();
        for h in self.closure_env.iter_mut().rev() {
            let mut nh: HashMap<String, PtrExpression> = HashMap::new();
            for (k, _v) in h {
                if let Some(exp) = env.find(k.to_string()) {
                    nh.insert(k.to_string(), exp.clone());
                }
            }
            l.push_back(nh);
            env.delete();
        }
        self.closure_env = l;
        if let Some(r) = results.pop() {
            return r;
        }
        return Err(create_error!("E9999"));
    }
}
impl Expression for RsFunction {
    fn value_string(&self) -> String {
        "Function".to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
    }
}
#[derive(Clone)]
pub struct RsLetLoop {
    data_type: DataType,
    param: Vec<String>,
    body: Vec<PtrExpression>,
    name: String,
    tail_recurcieve: bool,
}
impl RsLetLoop {
    fn new(
        sexp: &Vec<PtrExpression>,
        _name: String,
        map: &mut HashMap<String, PtrExpression>,
    ) -> RsLetLoop {
        let mut vec: Vec<PtrExpression> = Vec::new();
        vec.extend_from_slice(&sexp[3..]);

        let mut _param: Vec<String> = Vec::new();
        for k in map.keys() {
            _param.push((*k).to_string());
        }
        RsLetLoop {
            data_type: DataType::RsLetLoopDesc,
            param: _param,
            body: vec,
            name: _name,
            tail_recurcieve: false,
        }
    }
    // exp is slice
    fn set_tail_recurcieve(&mut self) {
        self.tail_recurcieve = self._set_tail_recurcieve(self.body.as_slice());
    }
    fn _set_tail_recurcieve(&self, exp: &[PtrExpression]) -> bool {
        for e in exp {
            if let Some(l) = e.as_any().downcast_ref::<RsList>() {
                if 0 == l.value.len() {
                    continue;
                }
                if let Some(n) = l.value[0].as_any().downcast_ref::<RsSymbol>() {
                    if n.value.as_str() == "if" {
                        return self._set_tail_recurcieve(&l.value[1..]);
                    }
                    if n.value == self.name {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    fn execute(&self, exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
        if self.param.len() != (exp.len() - 1) {
            return Err(create_error!("E1007"));
        }
        let mut vec: Vec<PtrExpression> = Vec::new();
        for e in &exp[1 as usize..] {
            let v = eval(e, env)?;
            vec.push(v);
        }
        let mut idx = 0;
        for s in &self.param {
            env.regist(s.to_string(), vec[idx].clone_box());
            idx += 1;
        }
        if self.tail_recurcieve == true {
            return Ok(Box::new(self.clone()));
        } else {
            let mut results: Vec<PtrExpression> = Vec::new();
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
impl Expression for RsLetLoop {
    fn value_string(&self) -> String {
        "Named Let".to_string()
    }
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
    fn as_any(&self) -> &Any {
        self
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
        match self {
            Number::Integer(x) => match other {
                Number::Integer(y) => {
                    if x == 0 && y == 0 {
                        return Number::Float(std::f64::NAN);
                    }
                    if y == 0 {
                        return Number::Float(std::f64::INFINITY);
                    }
                }
                Number::Float(_) => {}
            },
            Number::Float(_) => {}
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
            calc(exp, env, |x: Number, y: Number| x + y)
        });
        b.insert("-", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: Number, y: Number| x - y)
        });
        b.insert("*", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: Number, y: Number| x * y)
        });
        b.insert("/", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            calc(exp, env, |x: Number, y: Number| x / y)
        });
        b.insert("=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: &Number, y: &Number| x == y)
        });
        b.insert("<", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: &Number, y: &Number| x < y)
        });
        b.insert("<=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: &Number, y: &Number| x <= y)
        });
        b.insert(">", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: &Number, y: &Number| x > y)
        });
        b.insert(">=", |exp: &Vec<PtrExpression>, env: &mut SimpleEnv| {
            op(exp, env, |x: &Number, y: &Number| x >= y)
        });
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
        for h in self.env_tbl.iter() {
            match h.get(&key) {
                Some(v) => {
                    return Some(v);
                }
                None => {}
            }
        }
        None
    }
    fn update(&mut self, key: String, exp: PtrExpression) {
        for h in self.env_tbl.iter_mut() {
            match h.get(&key) {
                Some(_) => {
                    h.insert(key.to_string(), exp);
                    return;
                }
                None => {}
            }
        }
    }
    #[allow(dead_code)]
    fn dump_env(&self) {
        println!("======== dump_env start ============");
        let mut i = 1;
        for exp in self.env_tbl.iter() {
            for (k, v) in exp {
                println!("{} {} nest:{}", k, v.value_string(), i);
            }
            i += 1;
        }
    }
}
//========================================================================
const PROMPT: &str = "<rust.elisp> ";
const QUIT: &str = "(quit)";
//========================================================================
fn set_f(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error!("E1007"));
    }
    if let Some(s) = exp[1].as_any().downcast_ref::<RsSymbol>() {
        if let Some(_) = env.find(s.value.to_string()) {
            let se = eval(&exp[2], env)?;
            env.update(s.value.to_string(), se);
            return Ok(Box::new(s.clone()));
        } else {
            return Err(create_error!("E1008"));
        }
    } else {
        return Err(create_error!("E1004"));
    }
}
fn time_f(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error!("E1007"));
    }

    let start = Instant::now();

    let result = eval(&exp[1], env);
    let end = start.elapsed();

    println!("{}.{:03}", end.as_secs(), end.subsec_nanos() / 1_000_000);
    return result;
}
fn let_f(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error!("E1007"));
    }
    let mut param: HashMap<String, PtrExpression> = HashMap::new();

    let mut idx = 1;
    if let Some(_) = exp[idx].as_any().downcast_ref::<RsSymbol>() {
        idx += 1;
    }
    // Parameter Setup
    if let Some(l) = exp[idx].as_any().downcast_ref::<RsList>() {
        for plist in &l.value {
            if let Some(p) = plist.as_any().downcast_ref::<RsList>() {
                if p.value.len() != 2 {
                    return Err(create_error!("E1007"));
                }
                if let Some(s) = p.value[0].as_any().downcast_ref::<RsSymbol>() {
                    let v = eval(&p.value[1], env)?;
                    param.insert(s.value_string(), v);
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
    if let Some(s) = exp[1].as_any().downcast_ref::<RsSymbol>() {
        let mut letloop = Box::new(RsLetLoop::new(&exp, s.value_string(), &mut param));
        letloop.set_tail_recurcieve();
        param.insert(s.value_string(), letloop);
    }

    // execute let
    let closure_env = param.clone();
    env.create();
    for (k, v) in param {
        env.regist(k, v);
    }
    let mut results: Vec<PtrExpression> = Vec::new();
    for e in &exp[idx as usize..] {
        loop {
            match eval(e, env) {
                Ok(o) => {
                    // tail recurcieve
                    if let Some(_) = o.as_any().downcast_ref::<RsLetLoop>() {
                        continue;
                    } else {
                        results.push(o);
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    env.delete();
    if let Some(r) = results.pop() {
        if let Some(f) = r.as_any().downcast_ref::<RsFunction>() {
            let mut c = f.clone();
            c.set_closure_env(closure_env);
            return Ok(Box::new(c));
        }
        return Ok(r);
    }
    return Err(create_error!("E9999"));
}
fn not(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error!("E1007"));
    }
    let o = eval(&exp[1], env)?;
    if let Some(b) = o.as_any().downcast_ref::<RsBoolean>() {
        return Ok(Box::new(RsBoolean::new(!b.value)));
    }
    return Err(create_error!("E1001"));
}
fn or(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error!("E1007"));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Some(b) = o.as_any().downcast_ref::<RsBoolean>() {
            if b.value == true {
                return Ok(o.clone_box());
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    return Ok(Box::new(RsBoolean::new(false)));
}
fn and(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error!("E1007"));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Some(b) = o.as_any().downcast_ref::<RsBoolean>() {
            if b.value == false {
                return Ok(o.clone_box());
            }
        } else {
            return Err(create_error!("E1001"));
        }
    }
    return Ok(Box::new(RsBoolean::new(true)));
}
fn expt(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error!("E1007"));
    }
    let mut vec: Vec<i64> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Some(i) = o.as_any().downcast_ref::<RsInteger>() {
            vec.push(i.value);
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
        return Ok(Box::new(RsFloat::new(1 as f64 / result as f64)));
    } else {
        return Ok(Box::new(RsInteger::new(result)));
    }
}
fn modulo(exp: &Vec<PtrExpression>, env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error!("E1007"));
    }
    let mut vec: Vec<i64> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;
        if let Some(i) = o.as_any().downcast_ref::<RsInteger>() {
            vec.push(i.value);
        } else {
            return Err(create_error!("E1002"));
        }
    }
    if vec[1] == 0 {
        return Err(create_error!("E1013"));
    }
    return Ok(Box::new(RsInteger::new(vec[0] % vec[1])));
}
fn lambda(exp: &Vec<PtrExpression>, _env: &mut SimpleEnv) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error!("E1007"));
    }
    if let Some(l) = exp[1].as_any().downcast_ref::<RsList>() {
        for e in &l.value {
            match e.data_type() {
                DataType::RsSymbolDesc => {}
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
                match (*n).data_type() {
                    DataType::RsSymbolDesc => {}
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
    if exp.len() < 3 {
        return Err(create_error!("E1007"));
    }
    let se = eval(&exp[1], env)?;

    if let Some(b) = se.as_any().downcast_ref::<RsBoolean>() {
        if b.value == true {
            return eval(&exp[2], env);
        } else if 4 <= exp.len() {
            return eval(&exp[3], env);
        }
        return Ok(Box::new(RsNil::new()));
    }
    return Err(create_error!("E1001"));
}
fn calc(
    exp: &Vec<PtrExpression>,
    env: &mut SimpleEnv,
    f: fn(x: Number, y: Number) -> Number,
) -> ResultExpression {
    let mut result: Number = Number::Integer(0);
    let mut first: bool = true;

    if 2 >= exp.len() {
        return Err(create_error!("E1007"));
    }
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;

        let param = match o.as_any().downcast_ref::<RsFloat>() {
            Some(v) => Number::Float(v.value),
            None => match o.as_any().downcast_ref::<RsInteger>() {
                Some(v) => Number::Integer(v.value),
                None => {
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
        Number::Integer(a) => {
            return Ok(Box::new(RsInteger::new(a)));
        }
        Number::Float(a) => {
            return Ok(Box::new(RsFloat::new(a)));
        }
    }
}
fn op(
    exp: &Vec<PtrExpression>,
    env: &mut SimpleEnv,
    f: fn(x: &Number, y: &Number) -> bool,
) -> ResultExpression {
    if 3 != exp.len() {
        return Err(create_error!("E1007"));
    }
    let mut vec: Vec<Number> = Vec::new();
    for e in &exp[1 as usize..] {
        let o = eval(e, env)?;

        if let Some(f) = o.as_any().downcast_ref::<RsFloat>() {
            vec.push(Number::Float(f.value));
        } else if let Some(i) = o.as_any().downcast_ref::<RsInteger>() {
            vec.push(Number::Integer(i.value));
        } else {
            return Err(create_error!("E1003"));
        }
    }
    return Ok(Box::new(RsBoolean::new(f(&vec[0], &vec[1]))));
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
        match $e.data_type() {
            DataType::RsBooleanDesc
            | DataType::RsCharDesc
            | DataType::RsIntegerDesc
            | DataType::RsFloatDesc
            | DataType::RsNilDesc => {
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
                if let Some(_) = v.as_any().downcast_ref::<RsLetLoop>() {
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
        if let Some(s) = v[0].as_any().downcast_ref::<RsSymbol>() {
            let e = eval(&v[0], env)?;
            if let Some(ll) = e.as_any().downcast_ref::<RsLetLoop>() {
                return ll.execute(v, env);
            }
            if let Some(f) = e.as_any().downcast_ref::<RsFunction>() {
                let mut c = f.clone();
                let result = c.execute(v, env);
                // For ex. (define (counter) (let ((c 0)) (lambda () (set! c (+ 1 c)) c)))
                env.update(s.value_string(), Box::new(c));
                return result;
            }
            if let Some(b) = e.as_any().downcast_ref::<RsBuildInFunction>() {
                return b.execute(v, env);
            }
        } else if let Some(_) = v[0].as_any().downcast_ref::<RsList>() {
            let e = eval(&v[0], env)?;
            if let Some(f) = e.as_any().downcast_ref::<RsFunction>() {
                return (f.clone()).execute(v, env);
            } else {
                return Err(create_error!("E1006"));
            }
        }
    }
    Err(create_error!("E1009"))
}
