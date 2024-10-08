/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::vec::Vec;

use crate::env::{GlobalTbl, SimpleEnv};
use crate::lisp::{BasicBuiltIn, Expression, Function, ResultExpression};
//========================================================================
pub(crate) type ExtFunction =
    Box<dyn Fn(&[Expression], &Environment) -> ResultExpression + Sync + Send + 'static>;
pub(crate) type EnvTable = Arc<Mutex<SimpleEnv>>;
//------------------------------------------------------------------------
pub type FunctionRc = Arc<Function>;
pub type ExtFunctionRc = Arc<ExtFunction>;
pub type ListRc = Arc<RwLock<Vec<Expression>>>;
pub type HashTableRc = Arc<RwLock<HashMap<String, Expression>>>;
pub type TreeMapRc = Arc<RwLock<BTreeMap<String, Expression>>>;
pub type StringRc = Arc<String>;
//========================================================================
#[macro_export]
macro_rules! reference_obj {
    // Arc<Mutex<Vec<Expression>>> is slowly(30%)
    //
    // ($e: expr) => {{
    //     debug!("lock {}:{}", file!(), line!());
    //     let v = $e.lock().unwrap();
    //     v.to_vec()
    // }};
    ($e: expr) => {
        $e.read().unwrap()
    };
}
#[macro_export]
macro_rules! mut_obj {
    // Case of Arc<Mutex<Vec<Expression>>>
    //
    // ($e: expr) => {{
    //     $e.lock().unwrap()
    // }};
    ($e: expr) => {
        $e.write().unwrap()
    };
}

#[macro_export]
macro_rules! reference_env {
    ($e: expr) => {
        $e.lock().unwrap()
    };
}

#[macro_export]
macro_rules! mut_env {
    ($e: expr) => {
        $e.lock().unwrap()
    };
}
#[macro_export]
macro_rules! get_ptr {
    ($e: expr) => {
        std::sync::Arc::as_ptr($e)
    };
}

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
    pub fn with_parent(parent: &Environment) -> Self {
        Environment {
            core: Arc::new(Mutex::new(SimpleEnv::new(Some(parent.core.clone())))),
            globals: parent.globals.clone(),
        }
    }
    pub fn create_func(func: Function) -> Expression {
        Expression::Function(Arc::new(func))
    }
    pub fn create_list(l: Vec<Expression>) -> Expression {
        Expression::List(Arc::new(RwLock::new(l)))
    }
    pub fn create_string(s: String) -> Expression {
        Expression::String(Arc::new(s))
    }
    pub fn create_vector(l: Vec<Expression>) -> Expression {
        Expression::Vector(Arc::new(RwLock::new(l)))
    }
    pub fn create_hash_table(h: HashMap<String, Expression>) -> Expression {
        Expression::HashTable(Arc::new(RwLock::new(h)))
    }
    pub fn create_tree_map(m: BTreeMap<String, Expression>) -> Expression {
        Expression::TreeMap(Arc::new(RwLock::new(m)))
    }
    pub fn create_tail_recursion(func: Function) -> Expression {
        Expression::TailRecursion(Arc::new(func))
    }
    pub fn regist(&self, key: String, exp: Expression) {
        self.core.lock().unwrap().regist(key, exp);
    }
    pub fn regist_root(&self, key: String, exp: Expression) {
        self.core.lock().unwrap().regist_root(key, exp);
    }
    #[inline]
    pub fn find(&self, key: &str) -> Option<Expression> {
        self.core.lock().unwrap().find(key)
    }
    #[inline]
    pub fn update(&self, key: &str, exp: Expression) {
        self.core.lock().unwrap().update(key, exp);
    }
    #[inline]
    pub fn get_builtin_func(&self, key: &str) -> Option<BasicBuiltIn> {
        self.globals.lock().unwrap().builtin_tbl.get(key).cloned()
    }
    #[inline]
    pub fn get_builtin_ext_func(&self, key: &str) -> Option<ExtFunctionRc> {
        self.globals
            .lock()
            .unwrap()
            .builtin_tbl_ext
            .get(key)
            .cloned()
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
    pub fn is_force_stop(&self) -> bool {
        self.globals.lock().unwrap().force_stop
    }
    #[inline]
    pub fn set_cont(&self, e: &Expression) {
        self.globals.lock().unwrap().cont = Some(e.clone());
    }
    #[inline]
    pub fn get_cont(&self) -> Option<Expression> {
        return self.globals.lock().unwrap().cont.clone();
    }
    pub fn inc_eval_count(&self) -> u32 {
        self.globals.lock().unwrap().eval_count += 1;
        self.globals.lock().unwrap().eval_count
    }
    pub fn reset_eval_count(&self) {
        self.globals.lock().unwrap().eval_count = 0;
    }
    pub fn set_limit_stop(&self, b: bool) {
        self.globals.lock().unwrap().limit_stop = b;
    }
    pub fn is_limit_stop(&self) -> bool {
        self.globals.lock().unwrap().limit_stop
    }
    pub fn as_ptr(&self) -> *const Environment {
        Arc::as_ptr(&self.core) as *const Environment
    }
    pub fn as_mut_ptr(&self) -> *mut Environment {
        Arc::as_ptr(&self.core) as *mut Environment
    }
}
impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
#[test]
fn test_env_api() {
    use crate::do_lisp_env;

    let env = Environment::new();
    env.regist_root("a".to_string(), Expression::Integer(10));
    assert_eq!(do_lisp_env("a", &env), "10");
    assert_eq!(env.inc_eval_count(), 1);

    let env: Environment = Default::default();
    env.as_ptr();
    env.as_mut_ptr();
}
