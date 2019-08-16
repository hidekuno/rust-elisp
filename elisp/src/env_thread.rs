/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::buildin::{create_function, BuildInTable};
use crate::lisp::{Expression, Operation, ResultExpression, RsFunction};
//========================================================================
type ExtOperation =
    Box<Fn(&[Expression], &Environment) -> ResultExpression + Sync + Send + 'static>;
type EnvTable = Arc<Mutex<SimpleEnv>>;
//------------------------------------------------------------------------
pub type FunctionRc = Arc<RsFunction>;
pub type ExtOperationRc = Arc<ExtOperation>;
//========================================================================
#[derive(Clone)]
pub struct Environment {
    core: EnvTable,
    globals: Arc<Mutex<GlobalTbl>>,
}
impl Environment {
    pub fn new() -> Self {
        Environment {
            core: Arc::new(Mutex::new(SimpleEnv::new(None))),
            globals: Arc::new(Mutex::new(GlobalTbl::new())),
        }
    }
    pub fn new_next(parent: &Environment) -> Self {
        Environment {
            core: Arc::new(Mutex::new(SimpleEnv::new(Some(parent.core.clone())))),
            globals: parent.globals.clone(),
        }
    }
    pub fn create_func(func: RsFunction) -> Expression {
        Expression::Function(Arc::new(func))
    }
    pub fn create_tail_recursion(func: RsFunction) -> Expression {
        Expression::TailRecursion(Arc::new(func))
    }
    pub fn regist(&self, key: String, exp: Expression) {
        self.core.lock().unwrap().regist(key, exp);
    }
    pub fn find(&self, key: &String) -> Option<Expression> {
        self.core.lock().unwrap().find(key)
    }
    pub fn update(&self, key: &String, exp: Expression) {
        self.core.lock().unwrap().update(key, exp);
    }
    pub fn get_builtin_func(&self, key: &str) -> Option<Operation> {
        match self.globals.lock().unwrap().builtin_tbl.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn get_builtin_ext_func(&self, key: &str) -> Option<ExtOperationRc> {
        match self.globals.lock().unwrap().builtin_tbl_ext.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn add_builtin_ext_func<F>(&self, key: &'static str, c: F)
    where
        F: Fn(&[Expression], &Environment) -> ResultExpression + Sync + Send + 'static,
    {
        self.globals
            .lock()
            .unwrap()
            .builtin_tbl_ext
            .insert(key, Arc::new(Box::new(c)));
    }
    pub fn set_tail_recursion(&self, b: bool) {
        self.globals.lock().unwrap().tail_recursion = b;
    }
    pub fn is_tail_recursion(&self) -> bool {
        self.globals.lock().unwrap().tail_recursion
    }
    pub fn set_force_stop(&self, b: bool) {
        self.globals.lock().unwrap().force_stop = b;
    }
    pub fn get_force_stop(&self) -> bool {
        self.globals.lock().unwrap().force_stop
    }
}
impl BuildInTable for HashMap<&'static str, Operation> {
    fn regist(&mut self, symbol: &'static str, func: Operation) {
        self.insert(symbol, func);
    }
}
struct GlobalTbl {
    builtin_tbl: HashMap<&'static str, Operation>,
    builtin_tbl_ext: HashMap<&'static str, ExtOperationRc>,
    tail_recursion: bool,
    force_stop: bool,
}
impl GlobalTbl {
    fn new() -> Self {
        let mut b: HashMap<&'static str, Operation> = HashMap::new();
        create_function::<HashMap<&'static str, Operation>>(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: HashMap::new(),
            tail_recursion: true,
            force_stop: false,
        }
    }
}
pub struct SimpleEnv {
    env_tbl: HashMap<String, Expression>,
    parent: Option<EnvTable>,
}
impl SimpleEnv {
    fn new(parent: Option<EnvTable>) -> Self {
        if let Some(p) = parent {
            SimpleEnv {
                env_tbl: HashMap::new(),
                parent: Some(p.clone()),
            }
        } else {
            SimpleEnv {
                env_tbl: HashMap::new(),
                parent: parent,
            }
        }
    }
    fn regist(&mut self, key: String, exp: Expression) {
        self.env_tbl.insert(key, exp);
    }
    fn find(&self, key: &String) -> Option<Expression> {
        match self.env_tbl.get(key) {
            Some(v) => Some(v.clone()),
            None => match self.parent {
                // p.borrow().find(key), cannot return value referencing temporary value
                Some(ref p) => p.lock().unwrap().find(key),
                None => None,
            },
        }
    }
    fn update(&mut self, key: &String, exp: Expression) {
        if self.env_tbl.contains_key(key) {
            self.env_tbl.insert(key.to_string(), exp);
        } else {
            match self.parent {
                Some(ref p) => p.lock().unwrap().update(key, exp),
                None => {}
            }
        }
    }
}
