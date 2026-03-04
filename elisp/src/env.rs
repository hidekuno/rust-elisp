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
use crate::reference_env;

use std::hash::{BuildHasher, Hasher};

pub(crate) struct FnvHasher(u64);
impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher(0xcbf29ce484222325)
    }
}
impl Hasher for FnvHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x00000100000001b3);
        }
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}
#[derive(Clone, Default)]
pub(crate) struct FnvBuildHasher;
impl BuildHasher for FnvBuildHasher {
    type Hasher = FnvHasher;
    #[inline]
    fn build_hasher(&self) -> FnvHasher {
        FnvHasher::default()
    }
}

type Map<T, U> = std::collections::HashMap<T, U, FnvBuildHasher>;

impl BuildInTable for Map<&'static str, BasicBuiltIn> {
    fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn) {
        self.insert(symbol, func);
    }
}
pub(crate) struct GlobalTbl {
    pub(crate) builtin_tbl: Map<&'static str, BasicBuiltIn>,
    pub(crate) builtin_tbl_ext: Map<&'static str, ExtFunctionRc>,
    pub(crate) tail_recursion: bool,
    pub(crate) cont: Option<Expression>,
    pub(crate) eval_count: u32,
}
impl GlobalTbl {
    pub fn new() -> Self {
        let mut b: Map<&'static str, BasicBuiltIn> = Default::default();
        create_function(&mut b);
        GlobalTbl {
            builtin_tbl: b,
            builtin_tbl_ext: Default::default(),
            tail_recursion: true,
            cont: None,
            eval_count: 0,
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
                env_tbl: Default::default(),
                parent: Some(p),
            }
        } else {
            SimpleEnv {
                env_tbl: Default::default(),
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
                Some(ref p) => reference_env!(p).find(key),
                None => None,
            },
        }
    }
    pub fn update(&mut self, key: &str, exp: Expression) {
        if let Some(v) = self.env_tbl.get_mut(key) {
            *v = exp;
        } else if let Some(ref p) = self.parent {
            mut_env!(p).update(key, exp)
        }
    }
    #[cfg(feature = "thread")]
    pub fn regist_root(&mut self, key: String, exp: Expression) {
        match &self.parent {
            Some(p) => reference_env!(p).regist_root(key, exp),
            None => {
                self.env_tbl.insert(key, exp);
            }
        }
    }
}
#[test]
#[cfg(feature = "thread")]
fn test_regist_root() {
    let mut env = SimpleEnv::new(None);
    env.regist_root("a".to_string(), Expression::Integer(10));
}
#[test]
fn global_tbl() {
    let g = GlobalTbl::new();
    assert!(g.tail_recursion);
    assert!(!g.builtin_tbl.is_empty());
    assert_eq!(g.builtin_tbl_ext.len(), 0);
}
#[test]
fn simple_env() {
    let mut s = SimpleEnv::new(None);
    assert_eq!(if s.parent.is_some() { "exists" } else { "None" }, "None");

    s.regist("x".to_string(), Expression::Integer(10));
    assert_eq!(
        if let Some(Expression::Integer(x)) = s.find("x") {
            x
        } else {
            -1
        },
        10
    );
    s.update("x", Expression::Integer(20));
    assert_eq!(
        if let Some(Expression::Integer(x)) = s.find("x") {
            x
        } else {
            -1
        },
        20
    );
}
