/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
// cargo test --lib
#[macro_use]
extern crate lazy_static;
pub mod buildin;
pub mod chars;
pub mod io;
pub mod lisp;
pub mod list;
pub mod math;
pub mod number;
pub mod operation;
pub mod strings;
pub mod syntax;
pub mod util;

#[cfg(not(feature = "thread"))]
pub mod env_single;

#[cfg(feature = "thread")]
pub mod env_thread;

#[cfg(test)]
pub fn do_lisp(program: &str) -> String {
    let env = lisp::Environment::new();
    return do_lisp_env(program, &env);
}
#[cfg(test)]
pub fn do_lisp_env(program: &str, env: &lisp::Environment) -> String {
    match lisp::do_core_logic(&program.into(), env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom() {
        assert_eq!(do_lisp("10"), "10");
        assert_eq!(do_lisp("10.5"), "10.5");
        assert_eq!(do_lisp("1/2"), "1/2");
        assert_eq!(do_lisp("#t"), "#t");
        assert_eq!(do_lisp("#f"), "#f");
        assert_eq!(do_lisp("#\\a"), "#\\a");
        assert_eq!(do_lisp("\"abc\""), "\"abc\"");
        assert_eq!(do_lisp("#\\space"), "#\\space");
        assert_eq!(do_lisp("#\\tab"), "#\\tab");
        assert_eq!(do_lisp("#\\newline"), "#\\newline");
        assert_eq!(do_lisp("#\\return"), "#\\return");
        assert_eq!(do_lisp("+"), "BuildIn Function");
    }
    #[test]
    fn atom_utf8() {
        assert_eq!(do_lisp("#\\山"), "#\\山");
        assert_eq!(do_lisp("\"山田太郎\""), "\"山田太郎\"");
        assert_eq!(do_lisp("\"山田(太郎\""), "\"山田(太郎\"");

        let env = lisp::Environment::new();
        do_lisp_env("(define 山 200)", &env);
        assert_eq!(do_lisp_env("山", &env), "200");
    }
    #[test]
    fn tail_recurcieve_1() {
        // stack overflow check
        assert_eq!(
            do_lisp("(let loop ((i 0)) (if (<= 10000 i) i (loop (+ i 1))))"),
            "10000"
        );
    }
    #[test]
    fn tail_recurcieve_2() {
        // stack overflow check
        assert_eq!(
            do_lisp("(let loop ((i 0)) (cond ((<= 10000 i) i) (else (loop (+ i 1)))))"),
            "10000"
        );
    }
    #[test]
    fn tail_recurcieve_3() {
        // stack overflow check
        let env = lisp::Environment::new();
        do_lisp_env("(define (hoge i) (if (<= 10000 i) i (hoge (+ i 1))))", &env);
        assert_eq!(do_lisp_env("(hoge 0)", &env), "10000");
    }
    #[test]
    fn sequence() {
        assert_eq!(
            do_lisp("(let ((i 0)) (define i 100) (set! i (+ i 1)) i)"),
            "101"
        );
        assert_eq!(
            do_lisp("((lambda (a) (define i 100) (set! i (+ i a)) i)10)"),
            "110"
        );
    }
    #[test]
    fn force_stop() {
        let env = lisp::Environment::new();
        assert!(env.is_force_stop() == false);
        do_lisp_env("( force-stop )", &env);
        assert!(env.is_force_stop() == true);
        assert_eq!(do_lisp_env("a", &env), "E9000");
        env.set_force_stop(false);
        assert_eq!(do_lisp_env("100", &env), "100");
    }
    #[test]
    fn set_tail_recursion() {
        let env = lisp::Environment::new();
        assert!(env.is_tail_recursion() == true);
        do_lisp_env("(  tail-recursion-off )", &env);
        assert!(env.is_tail_recursion() == false);
        do_lisp_env("(  tail-recursion-on )", &env);
        assert!(env.is_tail_recursion() == true);
    }
}
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn syntax_error() {
        assert_eq!(do_lisp("("), "E0001");
        assert_eq!(do_lisp(")"), "E0003");
        assert_eq!(do_lisp("(list (+ 1 2) 3"), "E0002");
    }
    #[test]
    fn atom() {
        assert_eq!(do_lisp("\""), "E0004");
        assert_eq!(do_lisp("\"a"), "E0004");
        assert_eq!(do_lisp("a\""), "E0004");
        assert_eq!(do_lisp("3/0"), "E1013");
    }
    #[test]
    fn atom_utf8() {
        assert_eq!(do_lisp("\"山"), "E0004");
        assert_eq!(do_lisp("山"), "E1008");
    }
    #[test]
    fn sequence() {
        assert_eq!(
            do_lisp("(let ((i 0)) (define i 100) (set! i (+ i b)) i)"),
            "E1008"
        );
        assert_eq!(
            do_lisp("((lambda (a) (define i 100) (set! i (+ i b)) i)10)"),
            "E1008"
        );
    }
    #[test]
    fn force_stop() {
        assert_eq!(do_lisp("(force-stop 10)"), "E1008");
    }
    #[test]
    fn set_tail_recursion() {
        assert_eq!(do_lisp("(tail-recursion-off 20)"), "E1008");
        assert_eq!(do_lisp("(tail-recursion-on 30)"), "E1008");
    }
}
