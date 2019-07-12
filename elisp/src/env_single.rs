/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::buildin::create_function;
use crate::lisp::Expression;
use crate::lisp::ResultExpression;
use crate::lisp::RsFunction;

//========================================================================
type Operation = fn(&[Expression], &mut Environment) -> ResultExpression;
type ExtOperation = Fn(&[Expression], &mut Environment) -> ResultExpression;
type EnvTable = Rc<RefCell<SimpleEnv>>;
//------------------------------------------------------------------------
pub type FunctionRc = Rc<RsFunction>;
pub type ExtOperationRc = Rc<ExtOperation>;

#[derive(Clone)]
pub struct Environment {
    core: Rc<RefCell<SimpleEnv>>,
    globals: Rc<RefCell<GlobalTbl>>,
}
impl Environment {
    pub fn new() -> Self {
        Environment {
            core: Rc::new(RefCell::new(SimpleEnv::new(None))),
            globals: Rc::new(RefCell::new(GlobalTbl::new())),
        }
    }
    pub fn new_next(parent: &Environment) -> Self {
        Environment {
            core: Rc::new(RefCell::new(SimpleEnv::new(Some(parent.core.clone())))),
            globals: parent.globals.clone(),
        }
    }
    pub fn create_func(func: RsFunction) -> Expression {
        Expression::Function(Rc::new(func))
    }
    pub fn create_tail_recursion(func: RsFunction) -> Expression {
        Expression::TailRecursion(Rc::new(func))
    }
    pub fn regist(&mut self, key: String, exp: Expression) {
        self.core.borrow_mut().regist(key, exp);
    }
    pub fn find(&self, key: &String) -> Option<Expression> {
        self.core.borrow().find(key)
    }
    pub fn update(&mut self, key: &String, exp: Expression) {
        self.core.borrow_mut().update(key, exp);
    }
    pub fn get_builtin_func(&self, key: &str) -> Option<Operation> {
        match self.globals.borrow().builtin_tbl.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn get_builtin_ext_func(
        &self,
        key: &str,
    ) -> Option<Rc<Fn(&[Expression], &mut Environment) -> ResultExpression + 'static>> {
        match self.globals.borrow().builtin_tbl_ext.get(key) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }
    pub fn add_builtin_func(&mut self, key: &'static str, func: Operation) {
        self.globals.borrow_mut().builtin_tbl.insert(key, func);
    }
    pub fn add_builtin_closure<F>(&mut self, key: &'static str, c: F)
    where
        F: Fn(&[Expression], &mut Environment) -> ResultExpression + 'static,
    {
        self.globals
            .borrow_mut()
            .builtin_tbl_ext
            .insert(key, Rc::new(c));
    }
    pub fn set_tail_recursion(&mut self, b: bool) {
        self.globals.borrow_mut().tail_recursion = b;
    }
    pub fn is_tail_recursion(&self) -> bool {
        self.globals.borrow_mut().tail_recursion
    }
}
struct GlobalTbl {
    builtin_tbl: BTreeMap<&'static str, Operation>,
    builtin_tbl_ext: BTreeMap<&'static str, Rc<ExtOperation>>,
    tail_recursion: bool,
}
impl GlobalTbl {
    pub fn new() -> Self {
        let mut b: BTreeMap<&'static str, Operation> = BTreeMap::new();
        create_function(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: BTreeMap::new(),
            tail_recursion: true,
        }
    }
}
pub struct SimpleEnv {
    env_tbl: BTreeMap<String, Expression>,
    parent: Option<EnvTable>,
}
impl SimpleEnv {
    pub fn new(parent: Option<EnvTable>) -> Self {
        if let Some(p) = parent {
            SimpleEnv {
                env_tbl: BTreeMap::new(),
                parent: Some(p.clone()),
            }
        } else {
            SimpleEnv {
                env_tbl: BTreeMap::new(),
                parent: parent,
            }
        }
    }
    pub fn regist(&mut self, key: String, exp: Expression) {
        self.env_tbl.insert(key, exp);
    }
    pub fn find(&self, key: &String) -> Option<Expression> {
        match self.env_tbl.get(key) {
            Some(v) => Some(v.clone()),
            None => match self.parent {
                // p.borrow().find(key), cannot return value referencing temporary value
                Some(ref p) => p.borrow().find(key),
                None => None,
            },
        }
    }
    pub fn update(&mut self, key: &String, exp: Expression) {
        if self.env_tbl.contains_key(key) {
            self.env_tbl.insert(key.to_string(), exp);
        } else {
            match self.parent {
                Some(ref p) => p.borrow_mut().update(key, exp),
                None => {}
            }
        }
    }
}
