/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::lisp::create_function;
use crate::lisp::Expression;
use crate::lisp::ResultExpression;
use crate::lisp::RsFunction;
use crate::lisp::RsLetLoop;
//========================================================================
type Operation = fn(&[Expression], &mut Environment) -> ResultExpression;
type ExtOperation = Fn(&[Expression], &mut Environment) -> ResultExpression;
type EnvTable = Arc<Mutex<SimpleEnv>>;
//------------------------------------------------------------------------
pub type FunctionRc = Arc<RsFunction>;
pub type LetLoopRc = Arc<RsLetLoop>;
pub type ExtOperationRc = Arc<ExtOperation>;
//========================================================================
#[derive(Clone)]
pub struct Environment {
    core: Arc<Mutex<SimpleEnv>>,
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
    pub fn create_let_loop(letloop: RsLetLoop) -> Expression {
        Expression::LetLoop(Arc::new(letloop))
    }
    pub fn create_func(func: RsFunction) -> Expression {
        Expression::Function(Arc::new(func))
    }
    pub fn create_tail_recursion(func: RsFunction) -> Expression {
        Expression::TailRecursion(Arc::new(func))
    }
    pub fn regist(&mut self, key: String, exp: Expression) {
        self.core.lock().unwrap().regist(key, exp);
    }
    pub fn find(&self, key: &String) -> Option<Expression> {
        self.core.lock().unwrap().find(key)
    }
    pub fn update(&mut self, key: &String, exp: Expression) {
        self.core.lock().unwrap().update(key, exp);
    }
    pub fn get_builtin_func(&self, key: &str) -> Option<Operation> {
        match self.globals.lock().unwrap().builtin_tbl.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn get_builtin_ext_func(
        &self,
        key: &str,
    ) -> Option<Arc<Fn(&[Expression], &mut Environment) -> ResultExpression + 'static>> {
        match self.globals.lock().unwrap().builtin_tbl_ext.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn add_builtin_func(&mut self, key: &'static str, func: Operation) {
        self.globals.lock().unwrap().builtin_tbl.insert(key, func);
    }
    pub fn add_builtin_closure<F>(&mut self, key: &'static str, c: F)
    where
        F: Fn(&[Expression], &mut Environment) -> ResultExpression + 'static,
    {
        self.globals
            .lock()
            .unwrap()
            .builtin_tbl_ext
            .insert(key, Arc::new(c));
    }
}
unsafe impl Send for Environment {}

#[derive(Clone)]
pub struct GlobalTbl {
    builtin_tbl: HashMap<&'static str, Operation>,
    builtin_tbl_ext: HashMap<&'static str, Arc<ExtOperation>>,
}
impl GlobalTbl {
    fn new() -> Self {
        let mut b: HashMap<&'static str, Operation> = HashMap::new();
        create_function(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: HashMap::new(),
        }
    }
}
#[derive(Clone)]
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