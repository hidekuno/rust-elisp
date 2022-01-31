/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use crate::create_error;
use crate::create_error_value;
use crate::mut_obj;
use crate::reference_obj;

use crate::buildin::BuildInTable;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error};
use crate::lisp::{HashTableRc, TreeMapRc};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryInto;

trait Map<T> {
    fn create_map() -> Expression;
    fn get(&self, key: &str) -> ResultExpression;
    fn insert(&mut self, key: String, exp: Expression);
    fn remove(&mut self, key: String) -> bool;
    fn has_key(&self, key: String) -> bool;
    fn clear(&mut self);
    fn get_map(exp: &Expression, env: &Environment) -> Result<T, Error>;
    fn keys(&self) -> Expression;
    fn values(&self) -> Expression;
}
impl Map<HashTableRc> for HashTableRc {
    fn create_map() -> Expression {
        Environment::create_hash_table(HashMap::new())
    }
    fn insert(&mut self, key: String, exp: Expression) {
        let mut m = mut_obj!(self);
        m.insert(key, exp);
    }
    fn get(&self, key: &str) -> ResultExpression {
        let m = &*reference_obj!(self);

        if let Some(exp) = m.get(key) {
            Ok(exp.clone())
        } else {
            Err(create_error!(ErrCode::E1021))
        }
    }
    fn has_key(&self, key: String) -> bool {
        let m = &*reference_obj!(self);
        m.get(&key).is_some()
    }
    fn remove(&mut self, key: String) -> bool {
        let mut m = mut_obj!(self);
        m.remove(&key).is_some()
    }
    fn clear(&mut self) {
        let mut m = mut_obj!(self);
        m.clear();
    }
    fn get_map(exp: &Expression, env: &Environment) -> Result<HashTableRc, Error> {
        match eval(exp, env)? {
            Expression::HashTable(v) => Ok(v),
            e => Err(create_error_value!(ErrCode::E1023, e)),
        }
    }
    fn keys(&self) -> Expression {
        let m = &*reference_obj!(self);
        let mut v = Vec::new();
        for key in m.keys() {
            v.push(Expression::Symbol(key.to_string()));
        }
        Environment::create_list(v)
    }
    fn values(&self) -> Expression {
        let m = &*reference_obj!(self);
        let mut v = Vec::new();
        for value in m.values() {
            v.push(value.clone());
        }
        Environment::create_list(v)
    }
}
impl Map<TreeMapRc> for TreeMapRc {
    fn create_map() -> Expression {
        Environment::create_tree_map(BTreeMap::new())
    }
    fn insert(&mut self, key: String, exp: Expression) {
        let mut v = mut_obj!(self);
        v.insert(key, exp);
    }
    fn get(&self, key: &str) -> ResultExpression {
        let v = &*reference_obj!(self);

        if let Some(exp) = v.get(key) {
            Ok(exp.clone())
        } else {
            Err(create_error!(ErrCode::E1021))
        }
    }
    fn has_key(&self, key: String) -> bool {
        let v = &*reference_obj!(self);
        v.get(&key).is_some()
    }
    fn remove(&mut self, key: String) -> bool {
        let mut v = mut_obj!(self);
        v.remove(&key).is_some()
    }
    fn clear(&mut self) {
        let mut v = mut_obj!(self);
        v.clear();
    }
    fn get_map(exp: &Expression, env: &Environment) -> Result<TreeMapRc, Error> {
        match eval(exp, env)? {
            Expression::TreeMap(v) => Ok(v),
            e => Err(create_error_value!(ErrCode::E1024, e)),
        }
    }
    fn keys(&self) -> Expression {
        let m = &*reference_obj!(self);
        let mut v = Vec::new();
        for key in m.keys() {
            v.push(Expression::Symbol(key.to_string()));
        }
        Environment::create_list(v)
    }
    fn values(&self) -> Expression {
        let m = &*reference_obj!(self);
        let mut v = Vec::new();
        for value in m.values() {
            v.push(value.clone());
        }
        Environment::create_list(v)
    }
}
pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("make-hash-table", make_map::<HashTableRc>);
    b.regist("hash-table-put!", map_put::<HashTableRc>);
    b.regist("hash-table-get", map_get::<HashTableRc>);
    b.regist("hash-table-exists?", map_exists::<HashTableRc>);
    b.regist("hash-table-contains?", map_exists::<HashTableRc>);
    b.regist("hash-table-size", hash_table_size);
    b.regist("hash-table-delete!", map_delete::<HashTableRc>);
    b.regist("hash-table-clear!", map_clear::<HashTableRc>);
    b.regist("hash-table-keys", map_keys::<HashTableRc>);
    b.regist("hash-table-values", map_values::<HashTableRc>);
    b.regist("alist->hash-table", map_alist::<HashTableRc>);

    b.regist("make-tree-map", make_map::<TreeMapRc>);
    b.regist("tree-map-put!", map_put::<TreeMapRc>);
    b.regist("tree-map-get", map_get::<TreeMapRc>);
    b.regist("tree-map-exists?", map_exists::<TreeMapRc>);
    b.regist("tree-map-delete!", map_delete::<TreeMapRc>);
    b.regist("tree-map-clear!", map_clear::<TreeMapRc>);
    b.regist("tree-map-keys", map_keys::<TreeMapRc>);
    b.regist("tree-map-values", map_values::<TreeMapRc>);
    b.regist("alist->tree-map", map_alist::<TreeMapRc>);
}

fn make_map<T>(exp: &[Expression], _env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(T::create_map())
}
fn map_put<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 4 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut map = T::get_map(&exp[1], env)?;

    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    let value = eval(&exp[3], env)?;
    map.insert(key, value);

    Ok(Expression::Nil())
}
fn map_get<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let map = T::get_map(&exp[1], env)?;

    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    map.get(&key)
}
fn map_delete<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut map = T::get_map(&exp[1], env)?;

    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    Ok(Expression::Boolean(map.remove(key)))
}
fn map_clear<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut map = T::get_map(&exp[1], env)?;
    map.clear();

    Ok(Expression::Nil())
}
fn map_exists<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let map = T::get_map(&exp[1], env)?;

    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    Ok(Expression::Boolean(map.has_key(key)))
}

fn map_keys<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let map = T::get_map(&exp[1], env)?;
    Ok(map.keys())
}
fn map_values<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let map = T::get_map(&exp[1], env)?;
    Ok(map.values())
}
fn map_alist<T>(exp: &[Expression], env: &Environment) -> ResultExpression
where
    T: Map<T>,
{
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let l = match eval(&exp[1], env)? {
        Expression::List(l) => l,
        e => return Err(create_error_value!(ErrCode::E1005, e)),
    };
    let l = &*reference_obj!(l);
    if l.is_empty() {
        return Err(create_error_value!(ErrCode::E1021, l.len()));
    }

    let m = T::create_map();
    let mut map = T::get_map(&m, env)?;

    for e in l {
        match e {
            // Proprietary implementation
            Expression::List(l) => {
                let l = &*reference_obj!(l);
                if l.len() != 2 {
                    return Err(create_error_value!(ErrCode::E1021, l.len()));
                }
                match &l[0] {
                    Expression::Symbol(s) => {
                        map.insert(s.to_string(), l[1].clone());
                    }
                    e => return Err(create_error_value!(ErrCode::E1004, e)),
                }
            }
            e => return Err(create_error_value!(ErrCode::E1005, e)),
        }
    }
    Ok(m)
}
fn hash_table_size(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let hash = reference_obj!(hash);

    Ok(Expression::Integer(hash.len().try_into().unwrap()))
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn make_hash_table() {
        assert_eq!(do_lisp("(make-hash-table)"), "HashTable");
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        assert_eq!(do_lisp_env("a", &env), "HashTable");
    }
    #[test]
    fn hash_table_put() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        assert_eq!(do_lisp_env("(hash-table-put! a 'abc 10)", &env), "nil");
        assert_eq!(
            do_lisp_env("(hash-table-put! a 'abc (list 1 2 3))", &env),
            "nil"
        );
    }
    #[test]
    fn hash_table_get() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def (list 1 2 3))", &env);

        assert_eq!(do_lisp_env("(hash-table-get a 'abc)", &env), "10");
        assert_eq!(do_lisp_env("(hash-table-get a 'def)", &env), "(1 2 3)");
    }
    #[test]
    fn hash_table_exists() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);

        assert_eq!(do_lisp_env("(hash-table-exists? a 'abc)", &env), "#t");
        assert_eq!(do_lisp_env("(hash-table-exists? a 'def)", &env), "#f");

        assert_eq!(do_lisp_env("(hash-table-contains? a 'abc)", &env), "#t");
        assert_eq!(do_lisp_env("(hash-table-contains? a 'def)", &env), "#f");
    }
    #[test]
    fn hash_table_size() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def 20)", &env);

        assert_eq!(do_lisp_env("(hash-table-size a)", &env), "2");
    }
    #[test]
    fn hash_table_delete() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def 20)", &env);

        assert_eq!(do_lisp_env("(hash-table-delete! a 'abc)", &env), "#t");
        assert_eq!(do_lisp_env("(hash-table-delete! a 'abc)", &env), "#f");
    }
    #[test]
    fn hash_table_clear() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def 20)", &env);

        assert_eq!(do_lisp_env("(hash-table-clear! a)", &env), "nil");
        assert_eq!(do_lisp_env("(hash-table-size a)", &env), "0");
    }
    #[test]
    fn hash_table_keys() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def 20)", &env);
        assert_eq!(do_lisp_env("(sort (hash-table-keys a))", &env), "(abc def)");
    }
    #[test]
    fn hash_table_values() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-hash-table))", &env);
        do_lisp_env("(hash-table-put! a 'abc 10)", &env);
        do_lisp_env("(hash-table-put! a 'def 20)", &env);
        assert_eq!(do_lisp_env("(sort (hash-table-values a))", &env), "(10 20)");
    }
    #[test]
    fn alist_hash_table() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (alist->hash-table '((a 10)(b 20)(c 30))))", &env);
        assert_eq!(do_lisp_env("(sort (hash-table-keys a))", &env), "(a b c)");
        assert_eq!(
            do_lisp_env("(sort (hash-table-values a))", &env),
            "(10 20 30)"
        );
    }

    #[test]
    fn make_tree_map() {
        assert_eq!(do_lisp("(make-tree-map)"), "TreeMap");
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        assert_eq!(do_lisp_env("a", &env), "TreeMap");
    }
    #[test]
    fn tree_map_put() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        assert_eq!(do_lisp_env("(tree-map-put! a 'abc 10)", &env), "nil");
        assert_eq!(
            do_lisp_env("(tree-map-put! a 'abc (list 1 2 3))", &env),
            "nil"
        );
    }
    #[test]
    fn tree_map_get() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);
        do_lisp_env("(tree-map-put! a 'def (list 1 2 3))", &env);

        assert_eq!(do_lisp_env("(tree-map-get a 'abc)", &env), "10");
        assert_eq!(do_lisp_env("(tree-map-get a 'def)", &env), "(1 2 3)");
    }
    #[test]
    fn tree_map_exists() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);

        assert_eq!(do_lisp_env("(tree-map-exists? a 'abc)", &env), "#t");
        assert_eq!(do_lisp_env("(tree-map-exists? a 'def)", &env), "#f");
    }
    #[test]
    fn tree_map_delete() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);
        do_lisp_env("(tree-map-put! a 'def 20)", &env);

        assert_eq!(do_lisp_env("(tree-map-delete! a 'abc)", &env), "#t");
        assert_eq!(do_lisp_env("(tree-map-delete! a 'abc)", &env), "#f");
    }
    #[test]
    fn tree_map_clear() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);
        do_lisp_env("(tree-map-put! a 'def 20)", &env);

        assert_eq!(do_lisp_env("(tree-map-clear! a)", &env), "nil");
    }
    #[test]
    fn tree_map_keys() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);
        do_lisp_env("(tree-map-put! a 'def 20)", &env);
        assert_eq!(do_lisp_env("(sort (tree-map-keys a))", &env), "(abc def)");
    }
    #[test]
    fn tree_map_values() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (make-tree-map))", &env);
        do_lisp_env("(tree-map-put! a 'abc 10)", &env);
        do_lisp_env("(tree-map-put! a 'def 20)", &env);
        assert_eq!(do_lisp_env("(tree-map-values a)", &env), "(10 20)");
    }
    #[test]
    fn alist_tree_map() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (alist->tree-map '((a 10)(b 20)(c 30))))", &env);
        assert_eq!(do_lisp_env("(sort (tree-map-keys a))", &env), "(a b c)");
        assert_eq!(do_lisp_env("(tree-map-values a)", &env), "(10 20 30)");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::do_lisp;
    #[test]
    fn make_hash_table() {
        assert_eq!(do_lisp("(make-hash-table 10)"), "E1007");
    }
    #[test]
    fn hash_table_put() {
        assert_eq!(do_lisp("(hash-table-put!)"), "E1007");
        assert_eq!(do_lisp("(hash-table-put! 10 20 30 40)"), "E1007");
        assert_eq!(do_lisp("(hash-table-put! 10 20 30)"), "E1023");
        assert_eq!(
            do_lisp("(hash-table-put! (make-hash-table) \"ABC\" 30)"),
            "E1004"
        );
        assert_eq!(
            do_lisp("(hash-table-put! (make-hash-table) 'ABC a)"),
            "E1008"
        );
    }
    #[test]
    fn hash_table_get() {
        assert_eq!(do_lisp("(hash-table-get)"), "E1007");
        assert_eq!(do_lisp("(hash-table-get 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(hash-table-get 10 20)"), "E1023");
        assert_eq!(
            do_lisp("(hash-table-get (make-hash-table) \"ABC\")"),
            "E1004"
        );
        assert_eq!(do_lisp("(hash-table-get (make-hash-table) 'abc)"), "E1021");
    }
    #[test]
    fn hash_table_exists() {
        assert_eq!(do_lisp("(hash-table-exists?)"), "E1007");
        assert_eq!(do_lisp("(hash-table-exists? 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(hash-table-exists? 10 20)"), "E1023");
        assert_eq!(
            do_lisp("(hash-table-exists? (make-hash-table) \"ABC\")"),
            "E1004"
        );
    }
    #[test]
    fn hash_table_size() {
        assert_eq!(do_lisp("(hash-table-size)"), "E1007");
        assert_eq!(do_lisp("(hash-table-size 10 20)"), "E1007");
        assert_eq!(do_lisp("(hash-table-size 10)"), "E1023");
    }
    #[test]
    fn hash_table_delete() {
        assert_eq!(do_lisp("(hash-table-delete!)"), "E1007");
        assert_eq!(do_lisp("(hash-table-delete! 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(hash-table-delete! 10 20)"), "E1023");
        assert_eq!(
            do_lisp("(hash-table-delete! (make-hash-table) \"ABC\")"),
            "E1004"
        );
    }
    #[test]
    fn hash_table_clear() {
        assert_eq!(do_lisp("(hash-table-clear!)"), "E1007");
        assert_eq!(do_lisp("(hash-table-clear! 10 20)"), "E1007");
        assert_eq!(do_lisp("(hash-table-clear! 10)"), "E1023");
    }
    #[test]
    fn hash_table_keys() {
        assert_eq!(do_lisp("(hash-table-keys)"), "E1007");
        assert_eq!(do_lisp("(hash-table-keys 10 20)"), "E1007");
        assert_eq!(do_lisp("(hash-table-keys 10)"), "E1023");
    }
    #[test]
    fn hash_table_values() {
        assert_eq!(do_lisp("(hash-table-values)"), "E1007");
        assert_eq!(do_lisp("(hash-table-values 10 20)"), "E1007");
        assert_eq!(do_lisp("(hash-table-values 10)"), "E1023");
    }
    #[test]
    fn alist_hash_table() {
        assert_eq!(do_lisp("(alist->hash-table)"), "E1007");
        assert_eq!(do_lisp("(alist->hash-table 10 20)"), "E1007");
        assert_eq!(do_lisp("(alist->hash-table 10)"), "E1005");
        assert_eq!(do_lisp("(alist->hash-table (list))"), "E1021");
        assert_eq!(do_lisp("(alist->hash-table (list 10))"), "E1005");
        assert_eq!(do_lisp("(alist->hash-table (list (list 10)))"), "E1021");
        assert_eq!(do_lisp("(alist->hash-table (list (list 10 10)))"), "E1004");
    }

    #[test]
    fn make_tree_map() {
        assert_eq!(do_lisp("(make-tree-map 10)"), "E1007");
    }
    #[test]
    fn tree_map_put() {
        assert_eq!(do_lisp("(tree-map-put!)"), "E1007");
        assert_eq!(do_lisp("(tree-map-put! 10 20 30 40)"), "E1007");
        assert_eq!(do_lisp("(tree-map-put! 10 20 30)"), "E1024");
        assert_eq!(
            do_lisp("(tree-map-put! (make-tree-map) \"ABC\" 30)"),
            "E1004"
        );
        assert_eq!(do_lisp("(tree-map-put! (make-tree-map) 'ABC a)"), "E1008");
    }
    #[test]
    fn tree_map_get() {
        assert_eq!(do_lisp("(tree-map-get)"), "E1007");
        assert_eq!(do_lisp("(tree-map-get 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(tree-map-get 10 20)"), "E1024");
        assert_eq!(do_lisp("(tree-map-get (make-tree-map) \"ABC\")"), "E1004");
        assert_eq!(do_lisp("(tree-map-get (make-tree-map) 'abc)"), "E1021");
    }
    #[test]
    fn tree_map_exists() {
        assert_eq!(do_lisp("(tree-map-exists?)"), "E1007");
        assert_eq!(do_lisp("(tree-map-exists? 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(tree-map-exists? 10 20)"), "E1024");
        assert_eq!(
            do_lisp("(tree-map-exists? (make-tree-map) \"ABC\")"),
            "E1004"
        );
    }
    #[test]
    fn tree_map_delete() {
        assert_eq!(do_lisp("(tree-map-delete!)"), "E1007");
        assert_eq!(do_lisp("(tree-map-delete! 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(tree-map-delete! 10 20)"), "E1024");
        assert_eq!(
            do_lisp("(tree-map-delete! (make-tree-map) \"ABC\")"),
            "E1004"
        );
    }
    #[test]
    fn tree_map_clear() {
        assert_eq!(do_lisp("(tree-map-clear!)"), "E1007");
        assert_eq!(do_lisp("(tree-map-clear! 10 20)"), "E1007");
        assert_eq!(do_lisp("(tree-map-clear! 10)"), "E1024");
    }
    #[test]
    fn alist_tree_map() {
        assert_eq!(do_lisp("(alist->tree-map)"), "E1007");
        assert_eq!(do_lisp("(alist->tree-map 10 20)"), "E1007");
        assert_eq!(do_lisp("(alist->tree-map 10)"), "E1005");
        assert_eq!(do_lisp("(alist->tree-map (list))"), "E1021");
        assert_eq!(do_lisp("(alist->tree-map (list 10))"), "E1005");
        assert_eq!(do_lisp("(alist->tree-map (list (list 10)))"), "E1021");
        assert_eq!(do_lisp("(alist->tree-map (list (list 10 10)))"), "E1004");
    }
}
