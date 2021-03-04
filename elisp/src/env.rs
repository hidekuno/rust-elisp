/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::collections::BTreeMap;

use crate::buildin::create_function;
use crate::buildin::BuildInTable;
use crate::lisp::{BasicBuiltIn, Expression};

#[cfg(not(feature = "thread"))]
use crate::env_single::ExtFunctionRc;

#[cfg(feature = "thread")]
use crate::env_thread::ExtFunctionRc;

#[cfg(not(feature = "thread"))]
use crate::env_single::EnvTable;

#[cfg(feature = "thread")]
use crate::env_thread::EnvTable;

use crate::mut_env;
use crate::referlence_env;

pub(crate) struct GlobalTbl {
    pub(crate) builtin_tbl: BTreeMap<&'static str, BasicBuiltIn>,
    pub(crate) builtin_tbl_ext: BTreeMap<&'static str, ExtFunctionRc>,
    pub(crate) tail_recursion: bool,
    pub(crate) force_stop: bool,
    pub(crate) cont: Option<Expression>,
}
impl GlobalTbl {
    pub fn new() -> Self {
        let mut b: BTreeMap<&'static str, BasicBuiltIn> = BTreeMap::new();
        create_function(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: BTreeMap::new(),
            tail_recursion: true,
            force_stop: false,
            cont: None,
        }
    }
}
impl BuildInTable for BTreeMap<&'static str, BasicBuiltIn> {
    fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn) {
        self.insert(symbol, func);
    }
}
pub(crate) struct SimpleEnv {
    pub(crate) env_tbl: BTreeMap<String, Expression>,
    pub(crate) parent: Option<EnvTable>,
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
                Some(ref p) => referlence_env!(p).find(key),
                None => None,
            },
        }
    }
    pub fn update(&mut self, key: &String, exp: Expression) {
        if self.env_tbl.contains_key(key) {
            self.env_tbl.insert(key.to_string(), exp);
        } else {
            match self.parent {
                Some(ref p) => mut_env!(p).update(key, exp),
                None => {}
            }
        }
    }
}
#[test]
fn global_tbl() {
    let g = GlobalTbl::new();
    assert_eq!(g.tail_recursion, true);
    assert_eq!(g.force_stop, false);
    assert_eq!(g.builtin_tbl.len() > 0, true);
    assert_eq!(g.builtin_tbl_ext.len(), 0);
}
#[test]
fn simple_env() {
    let mut s = SimpleEnv::new(None);
    assert_eq!(
        if let Some(_) = s.parent {
            "exists"
        } else {
            "None"
        },
        "None"
    );
    s.regist("x".to_string(), Expression::Integer(10));
    assert_eq!(
        if let Some(x) = s.find(&"x".to_string()) {
            match x {
                Expression::Integer(y) => y,
                _ => -1,
            }
        } else {
            -1
        },
        10
    );
    s.update(&"x".to_string(), Expression::Integer(20));
    assert_eq!(
        if let Some(x) = s.find(&"x".to_string()) {
            match x {
                Expression::Integer(y) => y,
                _ => -1,
            }
        } else {
            -1
        },
        20
    );
}
