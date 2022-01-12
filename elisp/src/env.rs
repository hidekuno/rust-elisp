/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
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

type Map<T, U> = std::collections::BTreeMap<T, U>;

impl BuildInTable for Map<&'static str, BasicBuiltIn> {
    fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn) {
        self.insert(symbol, func);
    }
}
pub(crate) struct GlobalTbl {
    pub(crate) builtin_tbl: Map<&'static str, BasicBuiltIn>,
    pub(crate) builtin_tbl_ext: Map<&'static str, ExtFunctionRc>,
    pub(crate) tail_recursion: bool,
    pub(crate) force_stop: bool,
    pub(crate) cont: Option<Expression>,
}
impl GlobalTbl {
    pub fn new() -> Self {
        let mut b: Map<&'static str, BasicBuiltIn> = Map::new();
        create_function(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: Map::new(),
            tail_recursion: true,
            force_stop: false,
            cont: None,
        }
    }
}
pub(crate) struct SimpleEnv {
    pub(crate) env_tbl: Map<String, Expression>,
    pub(crate) parent: Option<EnvTable>,
}
impl SimpleEnv {
    pub fn new(parent: Option<EnvTable>) -> Self {
        if let Some(p) = parent {
            SimpleEnv {
                env_tbl: Map::new(),
                parent: Some(p),
            }
        } else {
            SimpleEnv {
                env_tbl: Map::new(),
                parent,
            }
        }
    }
    pub fn regist(&mut self, key: String, exp: Expression) {
        self.env_tbl.insert(key, exp);
    }
    pub fn find(&self, key: &str) -> Option<Expression> {
        match self.env_tbl.get(key) {
            Some(v) => Some(v.clone()),
            None => match self.parent {
                Some(ref p) => referlence_env!(p).find(key),
                None => None,
            },
        }
    }
    pub fn update(&mut self, key: &str, exp: Expression) {
        if self.env_tbl.contains_key(key) {
            self.env_tbl.insert(key.to_string(), exp);
        } else if let Some(ref p) = self.parent {
            mut_env!(p).update(key, exp)
        }
    }
    #[cfg(feature = "thread")]
    pub fn regist_root(&mut self, key: String, exp: Expression) {
        match &self.parent {
            Some(p) => referlence_env!(p).regist_root(key, exp),
            None => {
                self.env_tbl.insert(key, exp);
            }
        }
    }
}
#[test]
fn global_tbl() {
    let g = GlobalTbl::new();
    assert!(g.tail_recursion);
    assert!(!g.force_stop);
    assert!(!g.builtin_tbl.is_empty());
    assert_eq!(g.builtin_tbl_ext.len(), 0);
}
#[test]
fn simple_env() {
    let mut s = SimpleEnv::new(None);
    assert_eq!(if s.parent.is_some() { "exists" } else { "None" }, "None");

    s.regist("x".to_string(), Expression::Integer(10));
    assert_eq!(
        if let Some(Expression::Integer(x)) = s.find(&"x".to_string()) {
            x
        } else {
            -1
        },
        10
    );
    s.update(&"x".to_string(), Expression::Integer(20));
    assert_eq!(
        if let Some(Expression::Integer(x)) = s.find(&"x".to_string()) {
            x
        } else {
            -1
        },
        20
    );
}
