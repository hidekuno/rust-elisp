// cargo test --lib
#[macro_use]
extern crate lazy_static;

pub mod lisp;

#[allow(dead_code)]
fn do_lisp(program: &str) -> String {
    let mut env = lisp::SimpleEnv::new();
    return do_lisp_env(program, &mut env);
}
#[allow(dead_code)]
fn do_lisp_env(program: &str, env: &mut lisp::SimpleEnv) -> String {
    match lisp::do_core_logic(program.to_string(), env) {
        Ok(v) => {
            return v.value_string();
        }
        Err(e) => {
            return String::from(e.get_code());
        }
    }
}
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn atom() {
        assert!(do_lisp("10") == "10".to_string());
        assert!(do_lisp("10.5") == "10.5".to_string());
        assert!(do_lisp("#t") == "#t".to_string());
        assert!(do_lisp("#\\a") == "a".to_string());
    }
    #[test]
    fn plus() {
        assert!(do_lisp("(+ 1 2)") == "3".to_string());
    }
    #[test]
    fn minus() {
        assert!(do_lisp("(- 6 1)") == "5".to_string());
    }
    #[test]
    fn multi() {
        assert!(do_lisp("(* 3 6)") == "18".to_string());
    }
    #[test]
    fn div() {
        assert!(do_lisp("(/ 9 3)") == "3".to_string());
    }
    #[test]
    fn eq() {
        assert!(do_lisp("(= 5 5)") == "#t".to_string());
    }
    #[test]
    fn not_eq() {
        assert!(do_lisp("(= 5 6)") == "#f".to_string());
    }
    #[test]
    fn than() {
        assert!(do_lisp("(> 6 5)") == "#t".to_string());
        assert!(do_lisp("(> 6 6)") == "#f".to_string());
    }
    #[test]
    fn less() {
        assert!(do_lisp("(< 5 6)") == "#t".to_string());
        assert!(do_lisp("(> 6 6)") == "#f".to_string());
    }
    #[test]
    fn than_eq() {
        assert!(do_lisp("(>= 6 6)") == "#t".to_string());
        assert!(do_lisp("(>= 6 5)") == "#t".to_string());
        assert!(do_lisp("(>= 5 6)") == "#f".to_string());
    }
    #[test]
    fn less_eq() {
        assert!(do_lisp("(<= 6 6)") == "#t".to_string());
        assert!(do_lisp("(<= 5 6)") == "#t".to_string());
        assert!(do_lisp("(<= 6 5)") == "#f".to_string());
    }
    #[test]
    fn define() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        assert!(do_lisp_env("a", &mut env) == "100".to_string());
        do_lisp_env("(define a 10.5)", &mut env);
        assert!(do_lisp_env("a", &mut env) == "10.5".to_string());
        do_lisp_env("(define a #t)", &mut env);
        assert!(do_lisp_env("a", &mut env) == "#t".to_string());
        do_lisp_env("(define a #\\A)", &mut env);
        assert!(do_lisp_env("a", &mut env) == "A".to_string());

        do_lisp_env("(define (fuga a b)(* a b))", &mut env);
        assert!(do_lisp_env("(fuga 6 8)", &mut env) == "48".to_string());
        do_lisp_env("(define (hoge a b) a)", &mut env);
        assert!(do_lisp_env("(hoge 6 8)", &mut env) == "6".to_string());
    }
    #[test]
    fn lambda() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &mut env);
        assert!(do_lisp_env("(hoge 6 8)", &mut env) == "14".to_string());
        do_lisp_env("(define hoge (lambda (a b) b))", &mut env);
        assert!(do_lisp_env("(hoge 6 8)", &mut env) == "8".to_string());
    }
    #[test]
    fn if_f() {
        assert!(do_lisp("(if (<= 1 6) #\\a #\\b)") == "a".to_string());
        assert!(do_lisp("(if (<= 9 6) #\\a #\\b)") == "b".to_string());
    }
}
mod error_tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn plus() {
        assert!(do_lisp("(+ 1 a)") == "E1008".to_string());
    }
    #[test]
    fn minus() {
        assert!(do_lisp("(- 6 a)") == "E1008".to_string());
    }
    #[test]
    fn multi() {
        assert!(do_lisp("(* 3 a)") == "E1008".to_string());
    }
    #[test]
    fn div() {
        assert!(do_lisp("(/ 9 a)") == "E1008".to_string());
    }
    #[test]
    fn eq() {
        assert!(do_lisp("(= 5 a)") == "E1008".to_string());
    }
    #[test]
    fn not_eq() {
        assert!(do_lisp("(= 5 a)") == "E1008".to_string());
    }
    #[test]
    fn than() {
        assert!(do_lisp("(> 6 a)") == "E1008".to_string());
    }
    #[test]
    fn less() {
        assert!(do_lisp("(< 5 a)") == "E1008".to_string());
    }
    #[test]
    fn than_eq() {
        assert!(do_lisp("(>= 6 a)") == "E1008".to_string());
    }
    #[test]
    fn less_eq() {
        assert!(do_lisp("(<= 6 a)") == "E1008".to_string());
    }
    #[test]
    fn define() {
        let mut env = lisp::SimpleEnv::new();
        assert!(do_lisp_env("(define)", &mut env) == "E1007".to_string());
        assert!(do_lisp_env("(define a)", &mut env) == "E1007".to_string());
        assert!(do_lisp_env("(define 1 10)", &mut env) == "E1004".to_string());
        assert!(do_lisp_env("(define (hoge a 1) (+ 100 a))", &mut env) == "E1004".to_string());
        assert!(do_lisp_env("(define (hoge 1 a) (+ 100 a))", &mut env) == "E1004".to_string());
        assert!(do_lisp_env("(define (100 a b) (+ 100 a))", &mut env) == "E1004".to_string());
        assert!(do_lisp_env("(define () (+ 100 a))", &mut env) == "E1007".to_string());

        assert!(do_lisp_env("(define a ga)", &mut env) == "E1008".to_string());
    }
    #[test]
    fn lambda() {
        let mut env = lisp::SimpleEnv::new();
        assert!(do_lisp_env("(lambda)", &mut env) == "E1007".to_string());
        assert!(do_lisp_env("(lambda (a b))", &mut env) == "E1007".to_string());
        assert!(do_lisp_env("(lambda (a b) (+ a b)(- a b))", &mut env) == "E1007".to_string());
        assert!(do_lisp_env("(lambda  a (+ a b))", &mut env) == "E1005".to_string());
        assert!(do_lisp_env("(lambda (a 1) (+ a 10))", &mut env) == "E1004".to_string());

        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &mut env);
        assert!(do_lisp_env("(hoge 10 ga)", &mut env) == "E1008".to_string());

        do_lisp_env("(define hoge (lambda (a b) (+ ga b)))", &mut env);
        assert!(do_lisp_env("(hoge 10 20)", &mut env) == "E1008".to_string());
    }
    #[test]
    fn if_f() {
        assert!(do_lisp("(if (<= 1 6) #t)") == "E1004".to_string());
        assert!(do_lisp("(if (<= 1 6) #t #f 10)") == "E1004".to_string());
        assert!(do_lisp("(if (<= 1 6) a #\\b)") == "E1008".to_string());
        assert!(do_lisp("(if (<= 9 6) #\\a b)") == "E1008".to_string());
        assert!(do_lisp("(if 9 #\\a b)") == "E1001".to_string());
    }
}
