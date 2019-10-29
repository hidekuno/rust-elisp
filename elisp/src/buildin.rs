/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::lisp::BasicBuiltIn;

use crate::chars;
use crate::io;
use crate::list;
use crate::math;
use crate::operation;
use crate::strings;
use crate::syntax;
use crate::util;

pub trait BuildInTable {
    fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn);
}

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    chars::create_function(b);

    list::create_function(b);

    math::create_function(b);

    strings::create_function(b);

    operation::create_function(b);

    syntax::create_function(b);

    io::create_function(b);

    util::create_function(b);
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::collections::HashMap;

    #[test]
    fn test_dyn_dispatch() {
        fn create_function_dyn_dispatch(b: &mut dyn BuildInTable) {
            create_function(b);
        }

        #[cfg(not(feature = "thread"))]
        impl BuildInTable for HashMap<&'static str, BasicBuiltIn> {
            fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn) {
                self.insert(symbol, func);
            }
        }

        #[cfg(feature = "thread")]
        impl BuildInTable for BTreeMap<&'static str, BasicBuiltIn> {
            fn regist(&mut self, symbol: &'static str, func: BasicBuiltIn) {
                self.insert(symbol, func);
            }
        }

        let mut b = BTreeMap::new();
        let mut h = HashMap::new();
        create_function_dyn_dispatch(&mut b);
        create_function_dyn_dispatch(&mut h);
    }
}
