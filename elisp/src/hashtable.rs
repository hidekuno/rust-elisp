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
use std::collections::HashMap;
use std::convert::TryInto;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("make-hash-table", make_hash_table);
    b.regist("hash-table-put!", hash_table_put);
    b.regist("hash-table-get", hash_table_get);
    b.regist("hash-table-exists?", hash_table_exists);
    b.regist("hash-table-contains?", hash_table_exists);
    b.regist("hash-table-size", hash_table_size);
    b.regist("hash-table-delete!", hash_table_delete);
    b.regist("hash-table-clear!", hash_table_clear);
}

fn make_hash_table(exp: &[Expression], _env: &Environment) -> ResultExpression {
    if exp.len() != 1 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(Environment::create_hash_table(HashMap::new()))
}
fn hash_table_put(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 4 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    let value = eval(&exp[3], env)?;

    let mut hash = mut_obj!(hash);
    hash.insert(key, value);

    Ok(Expression::Nil())
}
fn hash_table_get(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    let hash = reference_obj!(hash);
    if let Some(exp) = hash.get(&key) {
        Ok(exp.clone())
    } else {
        Err(create_error!(ErrCode::E1021))
    }
}
fn hash_table_exists(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    let hash = reference_obj!(hash);

    Ok(Expression::Boolean(hash.get(&key).is_some()))
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
fn hash_table_delete(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let key = match eval(&exp[2], env)? {
        Expression::Symbol(v) => v,
        e => return Err(create_error_value!(ErrCode::E1004, e)),
    };
    let mut hash = mut_obj!(hash);
    Ok(Expression::Boolean(hash.remove(&key).is_some()))
}
fn hash_table_clear(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let hash = match eval(&exp[1], env)? {
        Expression::HashTable(v) => v,
        e => return Err(create_error_value!(ErrCode::E1023, e)),
    };
    let mut hash = mut_obj!(hash);

    hash.clear();
    Ok(Expression::Nil())
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
        assert_eq!(do_lisp("(hash-table-contains?)"), "E1007");
        assert_eq!(do_lisp("(hash-table-contains? 10 20 30)"), "E1007");
        assert_eq!(do_lisp("(hash-table-contains? 10 20)"), "E1023");
        assert_eq!(
            do_lisp("(hash-table-contains? (make-hash-table) \"ABC\")"),
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
}
