/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use crate::env::{GlobalTbl, SimpleEnv};
use crate::lisp::{BasicBuiltIn, Expression, Function, ResultExpression};
//========================================================================
pub(crate) type ExtFunction = dyn Fn(&[Expression], &Environment) -> ResultExpression;
pub(crate) type EnvTable = Rc<RefCell<SimpleEnv>>;
//------------------------------------------------------------------------
pub type FunctionRc = Rc<Function>;
pub type ExtFunctionRc = Rc<ExtFunction>;
pub type ListRc = Rc<RefCell<Vec<Expression>>>;

#[macro_export]
macro_rules! referlence_list {
    ($e: expr) => {
        $e.borrow();
    };
}
#[macro_export]
macro_rules! mut_list {
    ($e: expr) => {
        $e.borrow_mut();
    };
}

#[macro_export]
macro_rules! referlence_env {
    ($e: expr) => {
        $e.borrow();
    };
}

#[macro_export]
macro_rules! mut_env {
    ($e: expr) => {
        $e.borrow_mut();
    };
}

#[macro_export]
macro_rules! get_ptr {
    ($e: expr) => {
        std::rc::Rc::as_ptr($e)
    };
}
#[derive(Clone)]
pub struct Environment {
    core: EnvTable,
    globals: Rc<RefCell<GlobalTbl>>,
}
impl Environment {
    pub fn new() -> Self {
        Environment {
            core: Rc::new(RefCell::new(SimpleEnv::new(None))),
            globals: Rc::new(RefCell::new(GlobalTbl::new())),
        }
    }
    pub fn with_parent(parent: &Environment) -> Self {
        Environment {
            core: Rc::new(RefCell::new(SimpleEnv::new(Some(parent.core.clone())))),
            globals: parent.globals.clone(),
        }
    }
    pub fn create_func(func: Function) -> Expression {
        Expression::Function(Rc::new(func))
    }
    pub fn create_list(l: Vec<Expression>) -> Expression {
        Expression::List(Rc::new(RefCell::new(l)))
    }
    pub fn create_tail_recursion(func: Function) -> Expression {
        Expression::TailRecursion(Rc::new(func))
    }
    pub fn regist(&self, key: String, exp: Expression) {
        self.core.borrow_mut().regist(key, exp);
    }
    #[inline]
    pub fn find(&self, key: &String) -> Option<Expression> {
        self.core.borrow().find(key)
    }
    #[inline]
    pub fn update(&self, key: &String, exp: Expression) {
        self.core.borrow_mut().update(key, exp);
    }
    pub fn get_builtin_func(&self, key: &str) -> Option<BasicBuiltIn> {
        match self.globals.borrow().builtin_tbl.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn get_builtin_ext_func(
        &self,
        key: &str,
    ) -> Option<Rc<dyn Fn(&[Expression], &Environment) -> ResultExpression + 'static>> {
        match self.globals.borrow().builtin_tbl_ext.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn add_builtin_ext_func<F>(&self, key: &'static str, c: F)
    where
        F: Fn(&[Expression], &Environment) -> ResultExpression + 'static,
    {
        self.globals
            .borrow_mut()
            .builtin_tbl_ext
            .insert(key, Rc::new(c));
    }
    pub fn set_tail_recursion(&self, b: bool) {
        self.globals.borrow_mut().tail_recursion = b;
    }
    pub fn is_tail_recursion(&self) -> bool {
        self.globals.borrow_mut().tail_recursion
    }
    pub fn set_force_stop(&self, b: bool) {
        self.globals.borrow_mut().force_stop = b;
    }
    pub fn is_force_stop(&self) -> bool {
        self.globals.borrow_mut().force_stop
    }
    pub fn get_function_list(&self) -> Option<String> {
        self.get_environment_list(|_k, v| match v {
            Expression::Function(_) => true,
            _ => false,
        })
    }
    pub fn get_variable_list(&self) -> Option<String> {
        self.get_environment_list(|_k, v| match v {
            Expression::Function(_) => false,
            _ => true,
        })
    }
    fn get_environment_list(&self, f: fn(&String, &Expression) -> bool) -> Option<String> {
        let mut list = Vec::new();
        let e = self.core.borrow();
        for (k, v) in e.env_tbl.iter() {
            if f(k, v) {
                list.push(k.as_str());
            }
        }
        if list.len() == 0 {
            None
        } else {
            Some(list.join("\n"))
        }
    }
    pub fn get_builtin_func_list(&self) -> String {
        let mut s = String::new();
        for (i, (k, _)) in self.globals.borrow().builtin_tbl.iter().enumerate() {
            s.push_str(k);
            s.push_str(if (i + 1) % 10 == 0 { ",\n" } else { ",  " });
        }
        s
    }
    pub fn get_builtin_ext_list(&self) -> String {
        let mut s = String::new();
        for (k, _) in self.globals.borrow().builtin_tbl_ext.iter() {
            s.push_str(k);
            s.push_str("\n");
        }
        s
    }
    #[inline]
    pub fn set_cont(&self, e: &Expression) {
        self.globals.borrow_mut().cont = Some(e.clone());
    }
    #[inline]
    pub fn get_cont(&self) -> Option<Expression> {
        return self.globals.borrow().cont.clone();
    }
}
