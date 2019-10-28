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
pub mod strings;

#[cfg(not(feature = "thread"))]
pub mod env_single;

#[cfg(feature = "thread")]
pub mod env_thread;

#[cfg(test)]
fn do_lisp(program: &str) -> String {
    let env = lisp::Environment::new();
    return do_lisp_env(program, &env);
}
#[cfg(test)]
fn do_lisp_env(program: &str, env: &lisp::Environment) -> String {
    match lisp::do_core_logic(&program.into(), env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[cfg(test)]
macro_rules! assert_str {
    ($a: expr,
     $b: expr) => {
        assert!($a == $b.to_string())
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn atom() {
        assert_str!(do_lisp("10"), "10");
        assert_str!(do_lisp("10.5"), "10.5");
        assert_str!(do_lisp("1/2"), "1/2");
        assert_str!(do_lisp("#t"), "#t");
        assert_str!(do_lisp("#f"), "#f");
        assert_str!(do_lisp("#\\a"), "#\\a");
        assert_str!(do_lisp("\"abc\""), "\"abc\"");
        assert_str!(do_lisp("#\\space"), "#\\space");
        assert_str!(do_lisp("#\\tab"), "#\\tab");
        assert_str!(do_lisp("#\\newline"), "#\\newline");
        assert_str!(do_lisp("#\\return"), "#\\return");
        assert_str!(do_lisp("+"), "BuildIn Function");
    }
    #[test]
    fn atom_utf8() {
        assert_str!(do_lisp("#\\山"), "#\\山");
        assert_str!(do_lisp("\"山田太郎\""), "\"山田太郎\"");
        assert_str!(do_lisp("\"山田(太郎\""), "\"山田(太郎\"");

        let env = lisp::Environment::new();
        do_lisp_env("(define 山 200)", &env);
        assert_str!(do_lisp_env("山", &env), "200");
    }
    #[test]
    fn plus() {
        assert_str!(do_lisp("(+ 1 2)"), "3");
        assert_str!(do_lisp("(+ 1.25 2.25)"), "3.5");
        assert_str!(do_lisp("(+ 2.5 1)"), "3.5");
        assert_str!(do_lisp("(+ 3 1.5)"), "4.5");
        assert_str!(do_lisp("(+ (* 1 2)(* 3 4))"), "14");
        assert_str!(do_lisp("(+ 1/2 1)"), "3/2");
    }
    #[test]
    fn minus() {
        assert_str!(do_lisp("(- 6 1)"), "5");
        assert_str!(do_lisp("(- 5.75 1.5)"), "4.25");
        assert_str!(do_lisp("(- 6 1.5)"), "4.5");
        assert_str!(do_lisp("(- 6.5 3)"), "3.5");
        assert_str!(do_lisp("(- (* 3 4)(* 1 2))"), "10");
        assert_str!(do_lisp("(- 1 1/2)"), "1/2");
    }
    #[test]
    fn multi() {
        assert_str!(do_lisp("(* 3 6)"), "18");
        assert_str!(do_lisp("(* 0.5 5.75)"), "2.875");
        assert_str!(do_lisp("(* 3.5 6)"), "21");
        assert_str!(do_lisp("(* 6 3.5)"), "21");
        assert_str!(do_lisp("(* (+ 3 4)(+ 1 2))"), "21");
        assert_str!(do_lisp("(* 1/2 1)"), "1/2");
    }
    #[test]
    fn div() {
        assert_str!(do_lisp("(/ 4 3)"), "4/3");
        assert_str!(do_lisp("(/ 1 2)"), "1/2");
        assert_str!(do_lisp("(/ 9 3)"), "3");
        assert_str!(do_lisp("(/ 0.75 0.25)"), "3");
        assert_str!(do_lisp("(/ 9.5 5)"), "1.9");
        assert_str!(do_lisp("(/ 6 2.5)"), "2.4");
        assert_str!(do_lisp("(/ 0 0)"), "NaN");
        assert_str!(do_lisp("(/ 9 0)"), "inf");
        assert_str!(do_lisp("(/ 10 0.0)"), "inf");
        assert_str!(do_lisp("(+ 10 (/ 0 0))"), "NaN");
        assert_str!(do_lisp("(+ 10 (/ 9 0))"), "inf");
        assert_str!(do_lisp("(/ 0 9)"), "0");
        assert_str!(do_lisp("(/ 0.0 9)"), "0");
        assert_str!(do_lisp("(/ (+ 4 4)(+ 2 2))"), "2");
    }
    #[test]
    fn max_f() {
        assert_str!(do_lisp("(max 10 12 11 1 2)"), "12");
        assert_str!(do_lisp("(max 10 12 11 1 12)"), "12");
        assert_str!(do_lisp("(max 10 12 13.5 1 1)"), "13.5");
        assert_str!(do_lisp("(max 10 123/11 10.5 1 1)"), "123/11");
    }
    #[test]
    fn min_f() {
        assert_str!(do_lisp("(min 10 12 11 3 9)"), "3");
        assert_str!(do_lisp("(min 3 12 11 3 12)"), "3");
        assert_str!(do_lisp("(min 10 12 0.5 1 1)"), "0.5");
        assert_str!(do_lisp("(min 10 1/11 10.5 1 1)"), "1/11");
    }
    #[test]
    fn eq() {
        assert_str!(do_lisp("(= 5 5)"), "#t");
        assert_str!(do_lisp("(= 5.5 5.5)"), "#t");
        assert_str!(do_lisp("(= 5 5.0)"), "#t");
        assert_str!(do_lisp("(= 5.0 5)"), "#t");
        assert_str!(do_lisp("(= 5 6)"), "#f");
        assert_str!(do_lisp("(= 5.5 6.6)"), "#f");
        assert_str!(do_lisp("(= 5 6.6)"), "#f");
        assert_str!(do_lisp("(= 5.0 6)"), "#f");
        assert_str!(do_lisp("(= (+ 1 1)(+ 0 2))"), "#t");
    }
    #[test]
    fn than() {
        assert_str!(do_lisp("(> 6 5)"), "#t");
        assert_str!(do_lisp("(> 6.5 5.5)"), "#t");
        assert_str!(do_lisp("(> 6.1 6)"), "#t");
        assert_str!(do_lisp("(> 6 5.9)"), "#t");
        assert_str!(do_lisp("(> 6 6)"), "#f");
        assert_str!(do_lisp("(> 4.5 5.5)"), "#f");
        assert_str!(do_lisp("(> 4 5.5)"), "#f");
        assert_str!(do_lisp("(> 4.5 5)"), "#f");
        assert_str!(do_lisp("(> (+ 3 3) 5)"), "#t");
    }
    #[test]
    fn less() {
        assert_str!(do_lisp("(< 5 6)"), "#t");
        assert_str!(do_lisp("(< 5.6 6.5)"), "#t");
        assert_str!(do_lisp("(< 5 6.1)"), "#t");
        assert_str!(do_lisp("(< 5 6.5)"), "#t");
        assert_str!(do_lisp("(> 6 6)"), "#f");
        assert_str!(do_lisp("(> 6.5 6.6)"), "#f");
        assert_str!(do_lisp("(> 6 6.0)"), "#f");
        assert_str!(do_lisp("(> 5.9 6)"), "#f");
        assert_str!(do_lisp("(< 5 (+ 3 3))"), "#t");
    }
    #[test]
    fn than_eq() {
        assert_str!(do_lisp("(>= 6 6)"), "#t");
        assert_str!(do_lisp("(>= 6 5)"), "#t");
        assert_str!(do_lisp("(>= 6.1 5)"), "#t");
        assert_str!(do_lisp("(>= 7.6 7.6)"), "#t");
        assert_str!(do_lisp("(>= 6.3 5.2)"), "#t");
        assert_str!(do_lisp("(>= 6 5.1)"), "#t");
        assert_str!(do_lisp("(>= 5 6)"), "#f");
        assert_str!(do_lisp("(>= 5.1 6.2)"), "#f");
        assert_str!(do_lisp("(>= 5.9 6)"), "#f");
        assert_str!(do_lisp("(>= 5 6.1)"), "#f");
        assert_str!(do_lisp("(>= (+ 2 3 1) 6)"), "#t");
    }
    #[test]
    fn less_eq() {
        assert_str!(do_lisp("(<= 6 6)"), "#t");
        assert_str!(do_lisp("(<= 6 5)"), "#f");
        assert_str!(do_lisp("(<= 6.1 5)"), "#f");
        assert_str!(do_lisp("(<= 7.6 7.6)"), "#t");
        assert_str!(do_lisp("(<= 6.3 5.2)"), "#f");
        assert_str!(do_lisp("(<= 6 5.1)"), "#f");
        assert_str!(do_lisp("(<= 5 6)"), "#t");
        assert_str!(do_lisp("(<= 5.1 6.2)"), "#t");
        assert_str!(do_lisp("(<= 5.9 6)"), "#t");
        assert_str!(do_lisp("(<= 5 6.1)"), "#t");
        assert_str!(do_lisp("(<= (+ 3 3) 6)"), "#t");
    }
    #[test]
    fn ash() {
        assert_str!(do_lisp("(ash 10 1)"), "20");
        assert_str!(do_lisp("(ash 10 -1)"), "5");
        assert_str!(do_lisp("(ash 10 0)"), "10");
    }
    #[test]
    fn logand() {
        assert_str!(do_lisp("(logand 10 2)"), "2");
        assert_str!(do_lisp("(logand 10 2 3)"), "2");
    }
    #[test]
    fn logior() {
        assert_str!(do_lisp("(logior 10 2)"), "10");
        assert_str!(do_lisp("(logior 10 2 3)"), "11");
    }
    #[test]
    fn logxor() {
        assert_str!(do_lisp("(logxor 10 2)"), "8");
        assert_str!(do_lisp("(logxor 10 2 2)"), "10");
    }
    #[test]
    fn lognot() {
        assert_str!(do_lisp("(lognot 10)"), "-11");
    }
    #[test]
    fn even() {
        assert_str!(do_lisp("(even? 2)"), "#t");
        assert_str!(do_lisp("(even? 4)"), "#t");
        assert_str!(do_lisp("(even? 0)"), "#t");
        assert_str!(do_lisp("(even? 1)"), "#f");
        assert_str!(do_lisp("(even? 5)"), "#f");
    }
    #[test]
    fn odd() {
        assert_str!(do_lisp("(odd? 2)"), "#f");
        assert_str!(do_lisp("(odd? 4)"), "#f");
        assert_str!(do_lisp("(odd? 0)"), "#f");
        assert_str!(do_lisp("(odd? 1)"), "#t");
        assert_str!(do_lisp("(odd? 5)"), "#t");
    }
    #[test]
    fn zero() {
        assert_str!(do_lisp("(zero? 0)"), "#t");
        assert_str!(do_lisp("(zero? 0.0)"), "#t");
        assert_str!(do_lisp("(zero? 0/3)"), "#t");
        assert_str!(do_lisp("(zero? 2)"), "#f");
        assert_str!(do_lisp("(zero? -3)"), "#f");
        assert_str!(do_lisp("(zero? 2.5)"), "#f");
        assert_str!(do_lisp("(zero? 1/3)"), "#f");
    }
    #[test]
    fn positive() {
        assert_str!(do_lisp("(positive? 0)"), "#f");
        assert_str!(do_lisp("(positive? 0.0)"), "#f");
        assert_str!(do_lisp("(positive? 0/3)"), "#f");
        assert_str!(do_lisp("(positive? 2)"), "#t");
        assert_str!(do_lisp("(positive? -3)"), "#f");
        assert_str!(do_lisp("(positive? 2.5)"), "#t");
        assert_str!(do_lisp("(positive? -1.5)"), "#f");
        assert_str!(do_lisp("(positive? 1/3)"), "#t");
        assert_str!(do_lisp("(positive? -1/3)"), "#f");
    }
    #[test]
    fn negative() {
        assert_str!(do_lisp("(negative? 0)"), "#f");
        assert_str!(do_lisp("(negative? 0.0)"), "#f");
        assert_str!(do_lisp("(negative? 0/3)"), "#f");
        assert_str!(do_lisp("(negative? 2)"), "#f");
        assert_str!(do_lisp("(negative? -3)"), "#t");
        assert_str!(do_lisp("(negative? 2.5)"), "#f");
        assert_str!(do_lisp("(negative? -1.5)"), "#t");
        assert_str!(do_lisp("(negative? 1/3)"), "#f");
        assert_str!(do_lisp("(negative? -1/3)"), "#t");
    }
    #[test]
    fn list_f() {
        assert_str!(do_lisp("(list? (list 1 2 3))"), "#t");
        assert_str!(do_lisp("(list? 90)"), "#f");
    }
    #[test]
    fn pair_f() {
        assert_str!(do_lisp("(pair? (cons 1 2))"), "#t");
        assert_str!(do_lisp("(pair? 110)"), "#f");
    }
    #[test]
    fn char_f() {
        assert_str!(do_lisp("(char? #\\a)"), "#t");
        assert_str!(do_lisp("(char? 100)"), "#f");
    }
    #[test]
    fn string_f() {
        assert_str!(do_lisp("(string? \"a\")"), "#t");
        assert_str!(do_lisp("(string? 100)"), "#f");
    }
    #[test]
    fn integer_f() {
        assert_str!(do_lisp("(integer? 10)"), "#t");
        assert_str!(do_lisp("(integer? \"a\")"), "#f");
    }
    #[test]
    fn number_f() {
        assert_str!(do_lisp("(number? 10)"), "#t");
        assert_str!(do_lisp("(number? 10.5)"), "#t");
        assert_str!(do_lisp("(number? 1/3)"), "#t");
        assert_str!(do_lisp("(number? \"a\")"), "#f");
    }
    #[test]
    fn procedure_f() {
        assert_str!(do_lisp("(procedure? (lambda (n)n))"), "#t");
        assert_str!(do_lisp("(procedure? +)"), "#t");
        assert_str!(do_lisp("(procedure? 10)"), "#f");
    }
    #[test]
    fn define() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_str!(do_lisp_env("a", &env), "100");
        do_lisp_env("(define a 10.5)", &env);
        assert_str!(do_lisp_env("a", &env), "10.5");
        do_lisp_env("(define a #t)", &env);
        assert_str!(do_lisp_env("a", &env), "#t");
        do_lisp_env("(define a #\\A)", &env);
        assert_str!(do_lisp_env("a", &env), "#\\A");

        do_lisp_env("(define (fuga a b)(* a b))", &env);
        assert_str!(do_lisp_env("(fuga 6 8)", &env), "48");
        do_lisp_env("(define (hoge a b) a)", &env);
        assert_str!(do_lisp_env("(hoge 6 8)", &env), "6");

        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b a)", &env);
        assert_str!(do_lisp_env("b", &env), "100");

        do_lisp_env("(define plus +)", &env);
        assert_str!(do_lisp_env("(plus 10 20)", &env), "30");

        do_lisp_env("(define (p-nashi)(* 10 20))", &env);
        assert_str!(do_lisp_env("(p-nashi)", &env), "200");

        do_lisp_env("(define (hoge a b)(define (alpha x)(+ x 10))(define (beta y)(+ y 10))(+ (alpha a)(beta b)))",&env);
        assert_str!(do_lisp_env("(hoge 1 2)", &env), "23");
        assert_str!(do_lisp_env("(hoge 3 4)", &env), "27");
    }
    #[test]
    fn lambda() {
        assert_str!(do_lisp("((lambda (a b)(+ a b)) 1 2)"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &env);
        assert_str!(do_lisp_env("(hoge 6 8)", &env), "14");
        do_lisp_env("(define hoge (lambda (a b) b))", &env);
        assert_str!(do_lisp_env("(hoge 6 8)", &env), "8");
    }
    #[test]
    fn if_f() {
        assert_str!(do_lisp("(if (= 10 10) #\\a)"), "#\\a");
        assert_str!(do_lisp("(if (= 10 11) #\\a)"), "nil");
        assert_str!(do_lisp("(if (<= 1 6) #\\a #\\b)"), "#\\a");
        assert_str!(do_lisp("(if (<= 9 6) #\\a #\\b)"), "#\\b");
    }
    #[test]
    fn cond() {
        assert_str!(do_lisp("(cond ((= 10 10)))"), "#t");
        assert_str!(do_lisp("(cond ((= 100 10)))"), "nil");
        assert_str!(do_lisp("(cond (else 10))"), "10");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 10)", &env);
        assert_str!(do_lisp_env("(cond (a 20))", &env), "20");
        assert_str!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 20)", &env);
        assert_str!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"B\""
        );
        do_lisp_env("(define a 30)", &env);
        assert_str!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"C\""
        );
        assert_str!(
            do_lisp_env(
                "(cond ((= a 10) \"A\")((= a 20) \"B\")(else (* a 10)))",
                &env
            ),
            "300"
        );
        do_lisp_env("(define a 100)", &env);
        assert_str!(do_lisp_env("(cond ((= a 10) 20)(else 30 40))", &env), "40");
        assert_str!(
            do_lisp_env("(cond ((= a 100) 20 30)(else 40 50))", &env),
            "30"
        );
    }
    #[test]
    fn eqv() {
        assert_str!(do_lisp("(eqv? 1.1 1.1)"), "#t");
        assert_str!(do_lisp("(eq? 1.1 1.1)"), "#t");
        assert_str!(do_lisp("(eqv? 1.1 1.2)"), "#f");
        assert_str!(do_lisp("(eqv? 10 (+ 2 8))"), "#t");
        assert_str!(do_lisp("(eqv? 1 2)"), "#f");
        assert_str!(do_lisp("(eqv? 5/3 5/3)"), "#t");
        assert_str!(do_lisp("(eqv? 5/3 4/3)"), "#f");
        assert_str!(do_lisp("(eqv? (+ 1 2) 9/3)"), "#t");
        assert_str!(do_lisp("(eqv? 8/2 (+ 1 3))"), "#t");
        assert_str!(do_lisp("(eqv? 1 1.0)"), "#f");
        assert_str!(do_lisp("(eqv? 1/1 1.0)"), "#f");
        assert_str!(do_lisp("(eqv? 1.0 1)"), "#f");

        assert_str!(do_lisp("(eq? 'a 'a)"), "#t");
        assert_str!(do_lisp("(eq? 'a 'b)"), "#f");
        assert_str!(do_lisp("(eq? 'a 10)"), "#f");
        assert_str!(do_lisp("(eq? #f #f)"), "#t");
        assert_str!(do_lisp("(eq? #t #f)"), "#f");
        assert_str!(do_lisp("(eq? #t 10)"), "#f");
        assert_str!(do_lisp("(eq? #\\a #\\a)"), "#t");
        assert_str!(do_lisp("(eq? #\\a #\\b)"), "#f");
        assert_str!(do_lisp("(eq? #\\space #\\space)"), "#t");
    }
    #[test]
    fn case() {
        assert_str!(do_lisp("(case 10)"), "nil");
        assert_str!(do_lisp("(case 10 ((1 2) \"A\"))"), "nil");
        assert_str!(do_lisp("(case 10 (else 20))"), "20");
        assert_str!(do_lisp("(case 10 (else))"), "nil");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_str!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 1)", &env);
        assert_str!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"B\""
        );
        do_lisp_env("(define a 200)", &env);
        assert_str!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 400)", &env);
        assert_str!(
            do_lisp_env(
                "(case a ((100 200) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"B\""
        );
        do_lisp_env("(define b 100)", &env);
        assert_str!(
            do_lisp_env(
                "(case a ((200 b) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"B\""
        );
        do_lisp_env("(define a 100)", &env);
        assert_str!(
            do_lisp_env(
                "(case a ((200 b) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"A\""
        );
        do_lisp_env("(define a 1000)", &env);
        assert_str!(
            do_lisp_env(
                "(case a ((b 200) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"C\""
        );
        do_lisp_env("(define a 100) ", &env);
        assert_str!(
            do_lisp_env("(case a ((100 200) \"A\" \"B\") (else \"C\"))", &env),
            "\"B\""
        );
    }
    #[test]
    fn apply() {
        assert_str!(do_lisp("(apply + (list 1 2 3))"), "6");
        assert_str!(do_lisp("(apply + (list (+ 1 1) 2 3))"), "7");
        assert_str!(do_lisp("(apply - (list 5 3 2))"), "0");
        assert_str!(do_lisp("(apply (lambda (a b) (+ a b)) (list 1 2))"), "3");
        assert_str!(do_lisp("(apply + (iota 10))"), "45");

        let env = lisp::Environment::new();
        do_lisp_env("(define (hoge x y)(* x y))", &env);
        assert_str!(do_lisp_env("(apply hoge (list 3 4))", &env), "12");
    }
    #[test]
    fn identity() {
        assert_str!(do_lisp("(identity (+ 1 2 3))"), "6");
        assert_str!(do_lisp("(identity ((lambda (a b) (+ a b)) 1 2))"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_str!(do_lisp_env("(identity a)", &env), "100");
    }
    #[test]
    fn modulo() {
        assert_str!(do_lisp("(modulo 11 3)"), "2");
        assert_str!(do_lisp("(modulo 11 (+ 1 2))"), "2");
        assert_str!(do_lisp("(modulo  3 5)"), "3");
    }
    #[test]
    fn quotient() {
        assert_str!(do_lisp("(quotient 11 3)"), "3");
        assert_str!(do_lisp("(quotient 11 (+ 1 2))"), "3");
        assert_str!(do_lisp("(quotient 3 5)"), "0");
    }
    #[test]
    fn expt() {
        assert_str!(do_lisp("(expt 2 3)"), "8");
        assert_str!(do_lisp("(expt 2 (+ 1 2))"), "8");
        assert_str!(do_lisp("(expt 2 -2)"), "1/4");
        assert_str!(do_lisp("(expt 2 0)"), "1");
        assert_str!(do_lisp("(expt 2.0 3.0)"), "8");
        assert_str!(do_lisp("(expt 2.0 3)"), "8");
        assert_str!(do_lisp("(expt 2 3.0)"), "8");
    }
    #[test]
    fn and() {
        assert_str!(do_lisp("(and (= 1 1)(= 2 2))"), "#t");
        assert_str!(do_lisp("(and (= 1 1)(= 2 3))"), "#f");
        assert_str!(do_lisp("(and (= 2 1)(= 2 2))"), "#f");
        assert_str!(do_lisp("(and (= 0 1)(= 2 3))"), "#f");
    }
    #[test]
    fn or() {
        assert_str!(do_lisp("(or (= 1 1)(= 2 2))"), "#t");
        assert_str!(do_lisp("(or (= 1 1)(= 2 3))"), "#t");
        assert_str!(do_lisp("(or (= 2 1)(= 2 2))"), "#t");
        assert_str!(do_lisp("(or (= 0 1)(= 2 3))"), "#f");
    }
    #[test]
    fn not() {
        assert_str!(do_lisp("(not (= 1 1))"), "#f");
        assert_str!(do_lisp("(not (= 2 1))"), "#t");
    }
    #[test]
    fn let_f() {
        assert_str!(do_lisp("(let ((a 10)(b 20)) (+ a b))"), "30");
        assert_str!(
            do_lisp("(let loop ((i 0)(j 0)) (if (<= 10 i) (+ i j) (loop (+ i 1)(+ j 2))))"),
            "30"
        );
        assert_str!(
            do_lisp("(let loop ((i 0)) (if (<= 10 i) i (+ 10 (loop (+ i 1)))))"),
            "110"
        );
    }
    #[test]
    fn tail_recurcieve_1() {
        // stack overflow check
        assert_str!(
            do_lisp("(let loop ((i 0)) (if (<= 10000 i) i (loop (+ i 1))))"),
            "10000"
        );
    }
    #[test]
    fn tail_recurcieve_2() {
        // stack overflow check
        assert_str!(
            do_lisp("(let loop ((i 0)) (cond ((<= 10000 i) i) (else (loop (+ i 1)))))"),
            "10000"
        );
    }
    #[test]
    fn tail_recurcieve_3() {
        // stack overflow check
        let env = lisp::Environment::new();
        do_lisp_env("(define (hoge i) (if (<= 10000 i) i (hoge (+ i 1))))", &env);
        assert_str!(do_lisp_env("(hoge 0)", &env), "10000");
    }
    #[test]
    fn time_f() {
        let env = lisp::Environment::new();
        assert_str!(do_lisp_env("(time (+ 10 20))", &env), "30");
    }
    #[test]
    fn set_f() {
        let env = lisp::Environment::new();
        do_lisp_env("(define c 0)", &env);
        do_lisp_env("(set! c 10)", &env);
        assert_str!(do_lisp_env("c", &env), "10");
        do_lisp_env("(set! c (+ c 1))", &env);
        assert_str!(do_lisp_env("c", &env), "11");
    }
    #[test]
    fn list() {
        assert_str!(do_lisp("(list 1 2)"), "(1 2)");
        assert_str!(do_lisp("(list 0.5 1)"), "(0.5 1)");
        assert_str!(do_lisp("(list #t #f)"), "(#t #f)");
        assert_str!(do_lisp("(list (list 1)(list 2))"), "((1)(2))");
        assert_str!(
            do_lisp("(list (list (list 1))(list 2)(list 3))"),
            "(((1))(2)(3))"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 10)", &env);
        do_lisp_env("(define b 20)", &env);
        assert_str!(do_lisp_env("(list a b)", &env), "(10 20)");
    }
    #[test]
    fn make_list() {
        assert_str!(do_lisp("(make-list 10 0)"), "(0 0 0 0 0 0 0 0 0 0)");
        assert_str!(
            do_lisp("(make-list 4 (list 1 2 3))"),
            "((1 2 3)(1 2 3)(1 2 3)(1 2 3))"
        );
        assert_str!(do_lisp("(make-list 8 'a)"), "(a a a a a a a a)");
    }
    #[test]
    fn null_f() {
        assert_str!(do_lisp("(null? (list))"), "#t");
        assert_str!(do_lisp("(null? (list 10))"), "#f");
        assert_str!(do_lisp("(null? 10)"), "#f");
    }
    #[test]
    fn length() {
        assert_str!(do_lisp("(length (list))"), "0");
        assert_str!(do_lisp("(length (list 3))"), "1");
        assert_str!(do_lisp("(length (iota 10))"), "10");
    }
    #[test]
    fn car() {
        assert_str!(do_lisp("(car (list 1))"), "1");
        assert_str!(do_lisp("(car (list (list 2)))"), "(2)");
        assert_str!(
            do_lisp("(car (list (list (list 1))(list 2)(list 3)))"),
            "((1))"
        );
        assert_str!(do_lisp("(car (cons 10 20))"), "10");
    }
    #[test]
    fn cdr() {
        assert_str!(do_lisp("(cdr (list 1 2))"), "(2)");
        assert_str!(do_lisp("(cdr (list 1 0.5))"), "(0.5)");
        assert_str!(do_lisp("(cdr (list 1 (list 3)))"), "((3))");
        assert_str!(do_lisp("(cdr (cons 1 2))"), "2");
        assert_str!(do_lisp("(cdr (list 1))"), "()");
    }
    #[test]
    fn cadr() {
        assert_str!(do_lisp("(cadr (list 1 2))"), "2");
        assert_str!(do_lisp("(cadr (list 1 2 3))"), "2");
    }
    #[test]
    fn cons() {
        assert_str!(do_lisp("(cons  1 2)"), "(1 . 2)");
        assert_str!(do_lisp("(cons 1.5 2.5)"), "(1.5 . 2.5)");
        assert_str!(do_lisp("(cons  1 1.5)"), "(1 . 1.5)");
        assert_str!(do_lisp("(cons 1 (list 2))"), "(1 2)");
        assert_str!(do_lisp("(cons (list 1)(list 2))"), "((1) 2)");
    }
    #[test]
    fn append() {
        assert_str!(do_lisp("(append (list 1)(list 2))"), "(1 2)");
        assert_str!(do_lisp("(append (list 1)(list 2)(list 3))"), "(1 2 3)");
        assert_str!(
            do_lisp("(append (list (list 10))(list 2)(list 3))"),
            "((10) 2 3)"
        );
        assert_str!(do_lisp("(append (iota 5) (list 100))"), "(0 1 2 3 4 100)");
    }
    #[test]
    fn take() {
        assert_str!(do_lisp("(take (iota 10) 0)"), "()");
        assert_str!(do_lisp("(take (iota 10) 1)"), "(0)");
        assert_str!(do_lisp("(take (iota 10) 3)"), "(0 1 2)");
        assert_str!(do_lisp("(take (iota 10) 9)"), "(0 1 2 3 4 5 6 7 8)");
        assert_str!(do_lisp("(take (iota 10) 10)"), "(0 1 2 3 4 5 6 7 8 9)");
    }
    #[test]
    fn drop() {
        assert_str!(do_lisp("(drop (iota 10) 0)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_str!(do_lisp("(drop (iota 10) 1)"), "(1 2 3 4 5 6 7 8 9)");
        assert_str!(do_lisp("(drop (iota 10) 3)"), "(3 4 5 6 7 8 9)");
        assert_str!(do_lisp("(drop (iota 10) 9)"), "(9)");
        assert_str!(do_lisp("(drop (iota 10) 10)"), "()");
    }
    #[test]
    fn delete() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a (list 10 10.5 3/5 \"ABC\" #\\a #t))", &env);
        assert_str!(
            do_lisp_env("(delete 10 a)", &env),
            "(10.5 3/5 \"ABC\" #\\a #t)"
        );
        assert_str!(
            do_lisp_env("(delete 10.5 a)", &env),
            "(10 3/5 \"ABC\" #\\a #t)"
        );
        assert_str!(
            do_lisp_env("(delete 3/5 a)", &env),
            "(10 10.5 \"ABC\" #\\a #t)"
        );
        assert_str!(
            do_lisp_env("(delete \"ABC\" a)", &env),
            "(10 10.5 3/5 #\\a #t)"
        );
        assert_str!(
            do_lisp_env("(delete #\\a a)", &env),
            "(10 10.5 3/5 \"ABC\" #t)"
        );
        assert_str!(
            do_lisp_env("(delete #t a)", &env),
            "(10 10.5 3/5 \"ABC\" #\\a)"
        );
    }
    #[test]
    fn last() {
        assert_str!(do_lisp("(last (list 1))"), "1");
        assert_str!(do_lisp("(last (list 1 2))"), "2");
        assert_str!(do_lisp("(last (cons 1 2))"), "1");
    }
    #[test]
    fn reverse() {
        assert_str!(do_lisp("(reverse (list 10))"), "(10)");
        assert_str!(do_lisp("(reverse (iota 10))"), "(9 8 7 6 5 4 3 2 1 0)");
        assert_str!(do_lisp("(reverse (list))"), "()");
    }
    #[test]
    fn iota() {
        assert_str!(do_lisp("(iota 10)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_str!(do_lisp("(iota 10 1)"), "(1 2 3 4 5 6 7 8 9 10)");
        assert_str!(do_lisp("(iota 1 10)"), "(10)");
        assert_str!(do_lisp("(iota 10 1 2)"), "(1 3 5 7 9 11 13 15 17 19)");
        assert_str!(do_lisp("(iota 10 1 -1)"), "(1 0 -1 -2 -3 -4 -5 -6 -7 -8)");
    }
    #[test]
    fn map() {
        assert_str!(
            do_lisp("(map (lambda (n) (* n 10)) (iota 10 1))"),
            "(10 20 30 40 50 60 70 80 90 100)"
        );
        assert_str!(do_lisp("(map (lambda (n) (car n)) (list))"), "()");

        assert_str!(
            do_lisp("(map (lambda (n) (car n)) (list (list 1)(list 2)(list 3)))"),
            "(1 2 3)"
        );
        assert_str!(
            do_lisp("(map (lambda (n) (car n)) (list (list (list 1))(list 2)(list 3)))"),
            "((1) 2 3)"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        do_lisp_env(
            "(define d (list (list (list 1))(list (list 2))(list (list 3))))",
            &env,
        );

        assert_str!(
            do_lisp_env(
                "(map (lambda (n)(map (lambda (m)(/ m 10)) n))(list (list 10 20 30)(list a b c)))",
                &env
            ),
            "((1 2 3)(10 20 30))"
        );
        assert_str!(
            do_lisp_env("(map (lambda (n) (car n)) d)", &env),
            "((1)(2)(3))"
        );
    }
    #[test]
    fn filter() {
        assert_str!(
            do_lisp("(filter (lambda (n) (= 0 (modulo n 2))) (iota 10 1))"),
            "(2 4 6 8 10)"
        );
        assert_str!(
            do_lisp("(filter (lambda (n) (not (= 0 (modulo n 2)))) (iota 10 1))"),
            "(1 3 5 7 9)"
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        assert_str!(
            do_lisp_env("(filter (lambda (n) (= n 100)) (list a b c))", &env),
            "(100)"
        );
        assert_str!(
            do_lisp_env("(filter (lambda (n) (not (= n 100))) (list a b c))", &env),
            "(200 300)"
        );
    }
    #[test]
    fn reduce() {
        assert_str!(do_lisp("(reduce (lambda (a b) (+ a b))0(list 1 2 3))"), "6");
        assert_str!(
            do_lisp("(reduce (lambda (a b) (append a b))(list)(list (list 1) (list 2) (list 3)))"),
            "(1 2 3)"
        );
        assert_str!(
            do_lisp("(reduce (lambda (a b) (+ a b))(* 10 10)(list))"),
            "100"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b 200)", &env);
        do_lisp_env("(define c 300)", &env);
        assert_str!(
            do_lisp_env("(reduce (lambda (a b) (+ a b))0(list a b c))", &env),
            "600"
        );
    }
    #[test]
    fn for_each() {
        let env = lisp::Environment::new();
        do_lisp_env("(define c 0)", &env);
        do_lisp_env("(for-each (lambda (n) (set! c (+ c n)))(iota 5))", &env);
        assert_str!(do_lisp_env("c", &env), "10");
    }
    #[test]
    fn list_ref() {
        assert_str!(do_lisp("(list-ref (iota 10) 0)"), "0");
        assert_str!(do_lisp("(list-ref (iota 10) 1)"), "1");
        assert_str!(do_lisp("(list-ref (iota 10) 8)"), "8");
        assert_str!(do_lisp("(list-ref (iota 10) 9)"), "9");
        assert_str!(do_lisp("(list-ref '(#\\a #\\b #\\c) 1)"), "#\\b");
        assert_str!(do_lisp("(list-ref (list (list 0 1) 1 2 3) 0)"), "(0 1)");
    }
    #[test]
    fn sqrt() {
        assert_str!(do_lisp("(sqrt 9)"), "3");
        assert_str!(do_lisp("(sqrt 25.0)"), "5");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 16)", &env);
        assert_str!(do_lisp_env("(sqrt a)", &env), "4");
    }
    #[test]
    fn sin() {
        assert_str!(
            do_lisp("(sin (/(* 30 (* 4 (atan 1))) 180))"),
            "0.49999999999999994"
        );
        assert_str!(
            do_lisp("(sin (/(* 30.025 (* 4 (atan 1))) 180))"),
            "0.5003778272590873"
        );
        assert_str!(
            do_lisp("(sin (/(* 60 (* 4 (atan 1))) 180))"),
            "0.8660254037844386"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 30 (* 4 (atan 1))) 180))", &env);
        assert_str!(do_lisp_env("(sin a)", &env), "0.49999999999999994");
    }
    #[test]
    fn cos() {
        assert_str!(
            do_lisp("(cos (/(* 30 (* 4 (atan 1))) 180))"),
            "0.8660254037844387"
        );
        assert_str!(
            do_lisp("(cos (/(* 60 (* 4 (atan 1))) 180))"),
            "0.5000000000000001"
        );
        assert_str!(
            do_lisp("(cos (/(* 59.725 (* 4 (atan 1))) 180))"),
            "0.5041508484218754"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 60 (* 4 (atan 1))) 180))", &env);
        assert_str!(do_lisp_env("(cos a)", &env), "0.5000000000000001");
    }
    #[test]
    fn tan() {
        assert_str!(
            do_lisp("(tan (/(* 45 (* 4 (atan 1))) 180))"),
            "0.9999999999999999"
        );
        assert_str!(
            do_lisp("(tan (/(* 45.5 (* 4 (atan 1))) 180))"),
            "1.0176073929721252"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* 45 (* 4 (atan 1))) 180))", &env);
        assert_str!(do_lisp_env("(tan a)", &env), "0.9999999999999999");
    }
    #[test]
    fn asin() {
        assert_str!(
            do_lisp("(round (/ (* (asin (/(sqrt 3) 2)) 180)(*(atan 1)4)))"),
            "60"
        );
        assert_str!(
            do_lisp("(sin (asin (/(* pi 30)180)))"),
            do_lisp("(/(* pi 30)180)")
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* pi 30)180))", &env);
        assert_str!(do_lisp_env("(sin (asin a))", &env), do_lisp_env("a", &env));
    }
    #[test]
    fn acos() {
        assert_str!(
            do_lisp("(round (/ (* (acos (/ 1 2)) 180)(*(atan 1)4)))"),
            "60"
        );
        assert_str!(
            do_lisp("(cos (acos (/(* pi 30)180)))"),
            do_lisp("(/(* pi 30)180)")
        );

        let env = lisp::Environment::new();
        do_lisp_env("(define a (/(* pi 30)180))", &env);
        assert_str!(do_lisp_env("(cos (acos a))", &env), do_lisp_env("a", &env));
    }
    #[test]
    fn atan() {
        assert_str!(do_lisp("(round (/(* (atan 1) 180)(*(atan 1)4)))"), "45");
        assert_str!(do_lisp("(* 4 (atan 1))"), "3.141592653589793");
        assert_str!(do_lisp("(* 4 (atan 1.0))"), "3.141592653589793");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 1)", &env);
        assert_str!(do_lisp_env("(* 4 (atan a))", &env), "3.141592653589793");
    }
    #[test]
    fn exp() {
        assert_str!(do_lisp("(exp 1)"), "2.718281828459045");
        assert_str!(do_lisp("(exp 1.025)"), "2.7870954605658507");
        assert_str!(do_lisp("(exp 2)"), "7.38905609893065");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 3)", &env);
        assert_str!(do_lisp_env("(exp a)", &env), "20.085536923187668");
    }
    #[test]
    fn log() {
        assert_str!(do_lisp("(/(log 8)(log 2))"), "3");
        assert_str!(do_lisp("(/(log 9.0)(log 3.0))"), "2");
        assert_str!(do_lisp("(exp (/(log 8) 3))"), "2");
        assert_str!(do_lisp("(round (exp (* (log 2) 3)))"), "8");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 9)", &env);
        do_lisp_env("(define b 3)", &env);
        assert_str!(do_lisp_env("(/(log a)(log b))", &env), "2");
    }
    #[test]
    fn truncate() {
        assert_str!(do_lisp("(truncate 3.7)"), "3");
        assert_str!(do_lisp("(truncate 3.1)"), "3");
        assert_str!(do_lisp("(truncate -3.1)"), "-3");
        assert_str!(do_lisp("(truncate -3.7)"), "-3");
    }
    #[test]
    fn floor() {
        assert_str!(do_lisp("(floor 3.7)"), "3");
        assert_str!(do_lisp("(floor 3.1)"), "3");
        assert_str!(do_lisp("(floor -3.1)"), "-4");
        assert_str!(do_lisp("(floor -3.7)"), "-4");
    }
    #[test]
    fn ceiling() {
        assert_str!(do_lisp("(ceiling 3.7)"), "4");
        assert_str!(do_lisp("(ceiling 3.1)"), "4");
        assert_str!(do_lisp("(ceiling -3.1)"), "-3");
        assert_str!(do_lisp("(ceiling -3.7)"), "-3");
    }
    #[test]
    fn round() {
        assert_str!(do_lisp("(round 3.7)"), "4");
        assert_str!(do_lisp("(round 3.1)"), "3");
        assert_str!(do_lisp("(round -3.1)"), "-3");
        assert_str!(do_lisp("(round -3.7)"), "-4");
    }
    #[test]
    fn abs() {
        assert_str!(do_lisp("(abs -20)"), "20");
        assert_str!(do_lisp("(abs  20)"), "20");
        assert_str!(do_lisp("(abs -1.5)"), "1.5");
        assert_str!(do_lisp("(abs  1.5)"), "1.5");
        assert_str!(do_lisp("(abs -1/3)"), "1/3");
        assert_str!(do_lisp("(abs  1/3)"), "1/3");

        let env = lisp::Environment::new();
        do_lisp_env("(define a -20)", &env);
        do_lisp_env("(define b -1.5)", &env);
        assert_str!(do_lisp_env("(+ (abs a)(abs b))", &env), "21.5");
    }
    #[test]
    fn rand_integer() {
        assert_str!(do_lisp("(integer? (rand-integer))"), "#t");
        assert_str!(do_lisp("(* 0 (rand-integer))"), "0");
    }
    #[test]
    fn rand_list() {
        assert_str!(do_lisp("(length (rand-list 4))"), "4");
        assert_str!(
            do_lisp("(map (lambda (n) (integer? n)) (rand-list 4))"),
            "(#t #t #t #t)"
        );
    }
    #[test]
    #[allow(unused_must_use)]
    fn load_file() {
        let test_dir = Path::new(&env::var("HOME").unwrap()).join("tmp");
        let test_file = test_dir.join("test.scm");

        std::fs::create_dir(test_dir);
        std::fs::remove_file(&test_file);

        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "(define foo 100)").unwrap();
        writeln!(file, "(define hoge 200)").unwrap();
        writeln!(file, "(define fuga (+ foo hoge))").unwrap();
        writeln!(file, "(define a 100)(define b 200)(define c 300)").unwrap();
        writeln!(file, "(define d 100)").unwrap();
        file.flush().unwrap();

        let env = lisp::Environment::new();
        let f = test_file.as_path().to_str().expect("die");
        do_lisp_env(format!("(load-file \"{}\")", f).as_str(), &env);
        assert_str!(do_lisp_env("foo", &env), "100");
        assert_str!(do_lisp_env("hoge", &env), "200");
        assert_str!(do_lisp_env("fuga", &env), "300");
        assert_str!(do_lisp_env("(+ a b c)", &env), "600");
    }
    #[test]
    fn delay_force() {
        assert_str!(do_lisp("(delay (+ 1 1))"), "Promise");
        assert_str!(do_lisp("(force (delay (+ 1 1)))"), "2");
        assert_str!(do_lisp("(force  (+ 1 2))"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define p (delay (+ 2 3)))", &env);
        assert_str!(do_lisp_env("(force p)", &env), "5");
    }
    #[test]
    fn test_add_rational() {
        assert_str!(do_lisp("(+ 1 1/2)"), "3/2");
        assert_str!(do_lisp("(+ 2.5 1/4)"), "2.75");
        assert_str!(do_lisp("(+ 3/4 1)"), "7/4");
        assert_str!(do_lisp("(+ 1/4 2.5)"), "2.75");
        assert_str!(do_lisp("(+ 3/4 1/3)"), "13/12");
        assert_str!(do_lisp("(+ -1/3 1/3)"), "0");
    }
    #[test]
    fn test_sub_rational() {
        assert_str!(do_lisp("(- 1 1/2)"), "1/2");
        assert_str!(do_lisp("(- 2.5 1/4)"), "2.25");
        assert_str!(do_lisp("(- 1/2 1)"), "-1/2");
        assert_str!(do_lisp("(- 3/4 0.5)"), "0.25");
        assert_str!(do_lisp("(- 3/4 1/2)"), "1/4");
    }
    #[test]
    fn test_mul_rational() {
        assert_str!(do_lisp("(* 3 1/2)"), "3/2");
        assert_str!(do_lisp("(* 2.5 1/4)"), "0.625");
        assert_str!(do_lisp("(* 1/2 3)"), "3/2");
        assert_str!(do_lisp("(* 3/4 0.5)"), "0.375");
        assert_str!(do_lisp("(* 3/4 1/2)"), "3/8");
    }
    #[test]
    fn test_div_rational() {
        assert_str!(do_lisp("(/ 3 1/2)"), "6");
        assert_str!(do_lisp("(/ 2.5 1/3)"), "7.5");
        assert_str!(do_lisp("(/ 1/2 3)"), "1/6");
        assert_str!(do_lisp("(/ 3/4 0.5)"), "1.5");
        assert_str!(do_lisp("(/ 3/4 1/2)"), "3/2");
    }
    #[test]
    fn test_eq_rational() {
        assert_str!(do_lisp("(= 3 6/2)"), "#t");
        assert_str!(do_lisp("(= 0.5 1/2)"), "#t");
        assert_str!(do_lisp("(= 6/2 3)"), "#t");
        assert_str!(do_lisp("(= 3/2 1.5)"), "#t");
        assert_str!(do_lisp("(= 4/8 2/4)"), "#t");
    }

    #[test]
    fn test_lt_rational() {
        assert_str!(do_lisp("(< 3 7/2)"), "#t");
        assert_str!(do_lisp("(< 0.3 1/2)"), "#t");
        assert_str!(do_lisp("(< 6/2 4)"), "#t");
        assert_str!(do_lisp("(< 4/8 3/4)"), "#t");
    }

    #[test]
    fn test_le_rational() {
        assert_str!(do_lisp("(<= 3 7/2)"), "#t");
        assert_str!(do_lisp("(<= 0.3 1/2)"), "#t");
        assert_str!(do_lisp("(<= 6/2 4)"), "#t");
        assert_str!(do_lisp("(<= 4/8 3/4)"), "#t");

        assert_str!(do_lisp("(<= 3 6/2)"), "#t");
        assert_str!(do_lisp("(<= 0.5 1/2)"), "#t");
        assert_str!(do_lisp("(<= 6/2 3)"), "#t");
        assert_str!(do_lisp("(<= 3/2 1.5)"), "#t");
        assert_str!(do_lisp("(<= 4/8 2/4)"), "#t");
    }

    #[test]
    fn test_gt_rational() {
        assert_str!(do_lisp("(> 7/2 3)"), "#t");
        assert_str!(do_lisp("(> 1/2 0.3)"), "#t");
        assert_str!(do_lisp("(> 4 6/2)"), "#t");
        assert_str!(do_lisp("(> 1.6 3/2)"), "#t");
        assert_str!(do_lisp("(> 3/4 4/8)"), "#t");
    }
    #[test]
    fn test_ge_rational() {
        assert_str!(do_lisp("(>= 7/2 3)"), "#t");
        assert_str!(do_lisp("(>= 1/2 0.3)"), "#t");
        assert_str!(do_lisp("(>= 4 6/2)"), "#t");
        assert_str!(do_lisp("(>= 1.6 3/2)"), "#t");
        assert_str!(do_lisp("(>= 3/4 4/8)"), "#t");

        assert_str!(do_lisp("(>= 3 6/2)"), "#t");
        assert_str!(do_lisp("(>= 0.5 1/2)"), "#t");
        assert_str!(do_lisp("(>= 6/2 3)"), "#t");
        assert_str!(do_lisp("(>= 3/2 1.5)"), "#t");
        assert_str!(do_lisp("(>= 4/8 2/4)"), "#t");
    }
    #[test]
    fn display() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_str!(do_lisp_env("(display a)", &env), "nil");
    }
    #[test]
    fn newline() {
        assert_str!(do_lisp("(newline)"), "nil");
    }
    #[test]
    fn begin() {
        assert_str!(do_lisp("(begin (list 1 2)(list 3 4)(list 5 6))"), "(5 6)");
    }
    #[test]
    fn sequence() {
        assert_str!(
            do_lisp("(let ((i 0)) (define i 100) (set! i (+ i 1)) i)"),
            "101"
        );
        assert_str!(
            do_lisp("((lambda (a) (define i 100) (set! i (+ i a)) i)10)"),
            "110"
        );
    }
    #[test]
    fn format_f() {
        assert_str!(do_lisp("(format \"~D\" 10)"), "\"10\"");
        assert_str!(do_lisp("(format \"~d\" 10)"), "\"10\"");
        assert_str!(do_lisp("(format \"~X\" 10)"), "\"A\"");
        assert_str!(do_lisp("(format \"~x\" 10)"), "\"a\"");
        assert_str!(do_lisp("(format \"~O\" 10)"), "\"12\"");
        assert_str!(do_lisp("(format \"~o\" 10)"), "\"12\"");
        assert_str!(do_lisp("(format \"~B\" 10)"), "\"1010\"");
        assert_str!(do_lisp("(format \"~b\" 10)"), "\"1010\"");

        let env = lisp::Environment::new();
        do_lisp_env("(define a \"~D\")", &env);
        do_lisp_env("(define b 100)", &env);
        assert_str!(do_lisp_env("(format a b)", &env), "\"100\"");
    }
    #[test]
    fn force_stop() {
        let env = lisp::Environment::new();
        assert!(env.is_force_stop() == false);
        do_lisp_env("( force-stop )", &env);
        assert!(env.is_force_stop() == true);
        assert_str!(do_lisp_env("a", &env), "E9000");
        env.set_force_stop(false);
        assert_str!(do_lisp_env("100", &env), "100");
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
    #[test]
    fn string_eq() {
        assert_str!(do_lisp("(string=? \"abc\" \"abc\")"), "#t");
        assert_str!(do_lisp("(string=? \"abc\" \"ABC\")"), "#f");
    }
    #[test]
    fn string_less() {
        assert_str!(do_lisp("(string<? \"1234\" \"9\")"), "#t");
        assert_str!(do_lisp("(string<? \"9\" \"1234\")"), "#f");
    }
    #[test]
    fn string_than() {
        assert_str!(do_lisp("(string>? \"9\" \"1234\")"), "#t");
        assert_str!(do_lisp("(string>? \"1234\" \"9\")"), "#f");
    }
    #[test]
    fn string_le() {
        assert_str!(do_lisp("(string<=? \"1234\" \"9\")"), "#t");
        assert_str!(do_lisp("(string<=? \"1234\" \"1234\")"), "#t");
        assert_str!(do_lisp("(string<=? \"9\" \"1234\")"), "#f");
    }
    #[test]
    fn string_ge() {
        assert_str!(do_lisp("(string>=?  \"9\" \"1234\")"), "#t");
        assert_str!(do_lisp("(string>=?  \"1234\" \"1234\")"), "#t");
        assert_str!(do_lisp("(string>=?  \"1234\" \"9\")"), "#f");
    }
    #[test]
    fn char_eq() {
        assert_str!(do_lisp("(char=? #\\a #\\a)"), "#t");
        assert_str!(do_lisp("(char=? #\\a #\\b)"), "#f");
    }
    #[test]
    fn char_less() {
        assert_str!(do_lisp("(char<? #\\a #\\b)"), "#t");
        assert_str!(do_lisp("(char<? #\\b #\\a)"), "#f");
    }
    #[test]
    fn char_than() {
        assert_str!(do_lisp("(char>? #\\b #\\a)"), "#t");
        assert_str!(do_lisp("(char>? #\\a #\\b)"), "#f");
    }
    #[test]
    fn char_le() {
        assert_str!(do_lisp("(char<=? #\\a #\\b)"), "#t");
        assert_str!(do_lisp("(char<=? #\\a #\\a)"), "#t");
        assert_str!(do_lisp("(char<=? #\\b #\\a)"), "#f");
    }
    #[test]
    fn char_ge() {
        assert_str!(do_lisp("(char>=? #\\b #\\a)"), "#t");
        assert_str!(do_lisp("(char>=? #\\a #\\a)"), "#t");
        assert_str!(do_lisp("(char>=? #\\a #\\b)"), "#f");
    }
    #[test]
    fn str_append() {
        assert_str!(do_lisp("(string-append \"ABC\" \"DEF\")"), "\"ABCDEF\"");
        assert_str!(
            do_lisp("(string-append \"ABC\" \"DEF\" \"123\")"),
            "\"ABCDEF123\""
        );
    }
    #[test]
    fn string_length() {
        assert_str!(do_lisp("(string-length \"\")"), "0");
        assert_str!(do_lisp("(string-length \"1234567890\")"), "10");
        assert_str!(do_lisp("(string-length \"山\")"), "1");
    }
    #[test]
    fn string_size() {
        assert_str!(do_lisp("(string-size \"\")"), "0");
        assert_str!(do_lisp("(string-size \"1234567890\")"), "10");
        assert_str!(do_lisp("(string-size \"山\")"), "3");
    }
    #[test]
    fn number_string() {
        assert_str!(do_lisp("(number->string 10)"), "\"10\"");
        assert_str!(do_lisp("(number->string 10.5)"), "\"10.5\"");
        assert_str!(do_lisp("(number->string 1/3)"), "\"1/3\"");
    }
    #[test]
    fn string_number() {
        assert_str!(do_lisp("(string->number \"123\")"), "123");
        assert_str!(do_lisp("(string->number \"10.5\")"), "10.5");
        assert_str!(do_lisp("(string->number \"1/3\")"), "1/3");
    }
    #[test]
    fn list_string() {
        assert_str!(do_lisp("(list->string (list))"), "\"\"");
        assert_str!(do_lisp("(list->string (list #\\a #\\b #\\c))"), "\"abc\"");
    }
    #[test]
    fn string_list() {
        assert_str!(do_lisp("(string->list \"\")"), "()");
        assert_str!(do_lisp("(string->list \"abc\")"), "(#\\a #\\b #\\c)");
        assert_str!(do_lisp("(string->list \"山田\")"), "(#\\山 #\\田)");
    }
    #[test]
    fn substring() {
        assert_str!(do_lisp("(substring \"1234567890\" 1 2)"), "\"2\"");
        assert_str!(do_lisp("(substring \"1234567890\" 1 3)"), "\"23\"");
        assert_str!(do_lisp("(substring \"1234567890\" 0 10)"), "\"1234567890\"");
        assert_str!(do_lisp("(substring \"山\" 0 1)"), "\"山\"");
        assert_str!(do_lisp("(substring \"山1\" 0 2)"), "\"山1\"");
    }
    #[test]
    fn symbol_string() {
        assert_str!(do_lisp("(symbol->string 'abc)"), "\"abc\"");
    }
    #[test]
    fn string_symbol() {
        assert_str!(do_lisp("(string->symbol \"abc\")"), "abc");
    }
    #[test]
    fn make_string() {
        assert_str!(do_lisp("(make-string 4 #\\a)"), "\"aaaa\"");
        assert_str!(do_lisp("(make-string 4 #\\山)"), "\"山山山山\"");
    }
    #[test]
    fn integer_char() {
        assert_str!(do_lisp("(integer->char 65)"), "#\\A");
        assert_str!(do_lisp("(integer->char 23665)"), "#\\山");
    }
    #[test]
    fn char_integer() {
        assert_str!(do_lisp("(char->integer #\\A)"), "65");
        assert_str!(do_lisp("(char->integer #\\山)"), "23665");
    }
    #[test]
    fn quote() {
        assert_str!(do_lisp("(quote 1)"), "1");
        assert_str!(do_lisp("(quote \"abc\")"), "\"abc\"");
        assert_str!(do_lisp("(quote a)"), "a");
        assert_str!(do_lisp("(quote (a b c))"), "(a b c)");

        assert_str!(do_lisp("' a"), "a");
        assert_str!(do_lisp("'abc"), "abc");
        assert_str!(do_lisp("'\"abc\""), "\"abc\"");
        assert_str!(do_lisp("'\"abc\" '\"def\""), "\"def\"");
        assert_str!(do_lisp("'(a b c)"), "(a b c)");
        assert_str!(
            do_lisp("'(a b c (d e f (g h i)))"),
            "(a b c (d e f (g h i)))"
        );
    }
    #[test]
    fn get_env() {
        assert_str!(
            do_lisp("(get-environment-variable \"HOME\")"),
            format!("\"{}\"", env::var("HOME").unwrap())
        );
    }
}
#[cfg(test)]
mod error_tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn syntax_error() {
        assert_str!(do_lisp("("), "E0001");
        assert_str!(do_lisp(")"), "E0003");
        assert_str!(do_lisp("(list (+ 1 2) 3"), "E0002");
    }
    #[test]
    fn atom() {
        assert_str!(do_lisp("\""), "E0004");
        assert_str!(do_lisp("\"a"), "E0004");
        assert_str!(do_lisp("a\""), "E0004");
        assert_str!(do_lisp("3/0"), "E1013");
    }
    #[test]
    fn atom_utf8() {
        assert_str!(do_lisp("\"山"), "E0004");
        assert_str!(do_lisp("山"), "E1008");
    }
    #[test]
    fn plus() {
        assert_str!(do_lisp("(+ 1 a)"), "E1008");
        assert_str!(do_lisp("(+ 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(+ 1)"), "E1007");
    }
    #[test]
    fn minus() {
        assert_str!(do_lisp("(- 6 a)"), "E1008");
        assert_str!(do_lisp("(- 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(- 1)"), "E1007");
    }
    #[test]
    fn multi() {
        assert_str!(do_lisp("(* 6 a)"), "E1008");
        assert_str!(do_lisp("(* 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(* 1)"), "E1007");
    }
    #[test]
    fn div() {
        assert_str!(do_lisp("(/ 9 a)"), "E1008");
        assert_str!(do_lisp("(/ 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(/ 1)"), "E1007");
    }
    #[test]
    fn max_f() {
        assert_str!(do_lisp("(max 10)"), "E1007");
        assert_str!(do_lisp("(max 9 a)"), "E1008");
        assert_str!(do_lisp("(max 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(max 1)"), "E1007");
    }
    #[test]
    fn min_f() {
        assert_str!(do_lisp("(min 10)"), "E1007");
        assert_str!(do_lisp("(min 9 a)"), "E1008");
        assert_str!(do_lisp("(min 1 3.4 #t)"), "E1003");
        assert_str!(do_lisp("(min 1)"), "E1007");
    }
    #[test]
    fn eq() {
        assert_str!(do_lisp("(= 5)"), "E1007");
        assert_str!(do_lisp("(= 5 a)"), "E1008");
        assert_str!(do_lisp("(= 5 #f)"), "E1003");
    }
    #[test]
    fn than() {
        assert_str!(do_lisp("(> 6)"), "E1007");
        assert_str!(do_lisp("(> 6 a)"), "E1008");
        assert_str!(do_lisp("(> 6 #f)"), "E1003");
    }
    #[test]
    fn less() {
        assert_str!(do_lisp("(< 5)"), "E1007");
        assert_str!(do_lisp("(< 5 a)"), "E1008");
        assert_str!(do_lisp("(< 5 #f)"), "E1003");
    }
    #[test]
    fn than_eq() {
        assert_str!(do_lisp("(>= 6)"), "E1007");
        assert_str!(do_lisp("(>= 6 a)"), "E1008");
        assert_str!(do_lisp("(>= 6 #t)"), "E1003");
    }
    #[test]
    fn less_eq() {
        assert_str!(do_lisp("(<= 6)"), "E1007");
        assert_str!(do_lisp("(<= 6 a)"), "E1008");
        assert_str!(do_lisp("(<= 6 #t)"), "E1003");
    }
    #[test]
    fn ash() {
        assert_str!(do_lisp("(ash)"), "E1007");
        assert_str!(do_lisp("(ash 10)"), "E1007");
        assert_str!(do_lisp("(ash 10 1 1)"), "E1007");
        assert_str!(do_lisp("(ash a 1)"), "E1008");
        assert_str!(do_lisp("(ash 10 a)"), "E1008");
        assert_str!(do_lisp("(ash 10.5 1)"), "E1002");
        assert_str!(do_lisp("(ash 10 1.5)"), "E1002");
    }
    #[test]
    fn logand() {
        assert_str!(do_lisp("(logand)"), "E1007");
        assert_str!(do_lisp("(logand 10)"), "E1007");
        assert_str!(do_lisp("(logand a 1)"), "E1008");
        assert_str!(do_lisp("(logand 10 a)"), "E1008");
        assert_str!(do_lisp("(logand 10.5 1)"), "E1002");
        assert_str!(do_lisp("(logand 10 1.5)"), "E1002");
    }
    #[test]
    fn logior() {
        assert_str!(do_lisp("(logior)"), "E1007");
        assert_str!(do_lisp("(logior 10)"), "E1007");
        assert_str!(do_lisp("(logior a 1)"), "E1008");
        assert_str!(do_lisp("(logior 10 a)"), "E1008");
        assert_str!(do_lisp("(logior 10.5 1)"), "E1002");
        assert_str!(do_lisp("(logior 10 1.5)"), "E1002");
    }
    #[test]
    fn logxor() {
        assert_str!(do_lisp("(logxor)"), "E1007");
        assert_str!(do_lisp("(logxor 10)"), "E1007");
        assert_str!(do_lisp("(logxor a 1)"), "E1008");
        assert_str!(do_lisp("(logxor 10 a)"), "E1008");
        assert_str!(do_lisp("(logxor 10.5 1)"), "E1002");
        assert_str!(do_lisp("(logxor 10 1.5)"), "E1002");
    }
    #[test]
    fn lognot() {
        assert_str!(do_lisp("(lognot)"), "E1007");
        assert_str!(do_lisp("(lognot 10 10)"), "E1007");
        assert_str!(do_lisp("(lognot a)"), "E1008");
        assert_str!(do_lisp("(lognot 1.5)"), "E1002");
    }
    #[test]
    fn even() {
        assert_str!(do_lisp("(even?)"), "E1007");
        assert_str!(do_lisp("(even? 1 2)"), "E1007");
        assert_str!(do_lisp("(even? 1/3)"), "E1002");
        assert_str!(do_lisp("(even? 10.5)"), "E1002");
        assert_str!(do_lisp("(even? a)"), "E1008");
    }
    #[test]
    fn odd() {
        assert_str!(do_lisp("(odd?)"), "E1007");
        assert_str!(do_lisp("(odd? 1 2)"), "E1007");
        assert_str!(do_lisp("(odd? 1/3)"), "E1002");
        assert_str!(do_lisp("(odd? 10.5)"), "E1002");
        assert_str!(do_lisp("(odd? a)"), "E1008");
    }
    #[test]
    fn zero() {
        assert_str!(do_lisp("(zero?)"), "E1007");
        assert_str!(do_lisp("(zero? 1 2)"), "E1007");
        assert_str!(do_lisp("(zero? #f)"), "E1003");
        assert_str!(do_lisp("(zero? a)"), "E1008");
    }
    #[test]
    fn positive() {
        assert_str!(do_lisp("(positive?)"), "E1007");
        assert_str!(do_lisp("(positive? 1 2)"), "E1007");
        assert_str!(do_lisp("(positive? #f)"), "E1003");
        assert_str!(do_lisp("(positive? a)"), "E1008");
    }
    #[test]
    fn negative() {
        assert_str!(do_lisp("(negative?)"), "E1007");
        assert_str!(do_lisp("(negative? 1 2)"), "E1007");
        assert_str!(do_lisp("(negative? #f)"), "E1003");
        assert_str!(do_lisp("(negative? a)"), "E1008");
    }
    #[test]
    fn list_f() {
        assert_str!(do_lisp("(list?)"), "E1007");
        assert_str!(do_lisp("(list? (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(list? a)"), "E1008");
    }
    #[test]
    fn pair_f() {
        assert_str!(do_lisp("(pair?)"), "E1007");
        assert_str!(do_lisp("(pair? (cons 1 2)(cons 3 4))"), "E1007");
        assert_str!(do_lisp("(pair? a)"), "E1008");
    }
    #[test]
    fn char_f() {
        assert_str!(do_lisp("(char?)"), "E1007");
        assert_str!(do_lisp("(char? #\\a #\\b)"), "E1007");
        assert_str!(do_lisp("(char? a)"), "E1008");
    }
    #[test]
    fn string_f() {
        assert_str!(do_lisp("(string?)"), "E1007");
        assert_str!(do_lisp("(string? \"a\" \"b\")"), "E1007");
        assert_str!(do_lisp("(string? a)"), "E1008");
    }
    #[test]
    fn integer_f() {
        assert_str!(do_lisp("(integer?)"), "E1007");
        assert_str!(do_lisp("(integer? 10 20)"), "E1007");
        assert_str!(do_lisp("(integer? a)"), "E1008");
    }
    #[test]
    fn number_f() {
        assert_str!(do_lisp("(number?)"), "E1007");
        assert_str!(do_lisp("(number? 10 20)"), "E1007");
        assert_str!(do_lisp("(number? a)"), "E1008");
    }
    #[test]
    fn procedure_f() {
        assert_str!(do_lisp("(procedure?)"), "E1007");
        assert_str!(
            do_lisp("(procedure? (lambda (n) n)(lambda (n) n))"),
            "E1007"
        );
        assert_str!(do_lisp("(procedure? a)"), "E1008");
    }
    #[test]
    fn define() {
        let env = lisp::Environment::new();
        assert_str!(do_lisp_env("(define)", &env), "E1007");
        assert_str!(do_lisp_env("(define a)", &env), "E1007");
        assert_str!(do_lisp_env("(define a 11 12)", &env), "E1007");
        assert_str!(do_lisp_env("(define 1 10)", &env), "E1004");
        assert_str!(do_lisp_env("(define (hoge a 1) (+ 100 a))", &env), "E1004");
        assert_str!(do_lisp_env("(define (hoge 1 a) (+ 100 a))", &env), "E1004");
        assert_str!(do_lisp_env("(define (100 a b) (+ 100 a))", &env), "E1004");
        assert_str!(do_lisp_env("(define () (+ 100 a))", &env), "E1007");

        assert_str!(do_lisp_env("(define a ga)", &env), "E1008");
    }
    #[test]
    fn lambda() {
        let env = lisp::Environment::new();
        assert_str!(do_lisp_env("(lambda)", &env), "E1007");
        assert_str!(do_lisp_env("(lambda (a b))", &env), "E1007");
        assert_str!(do_lisp_env("(lambda  a (+ a b))", &env), "E1005");
        assert_str!(do_lisp_env("(lambda (a 1) (+ a 10))", &env), "E1004");
        assert_str!(do_lisp_env("((list 1) 10)", &env), "E1006");

        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &env);
        assert_str!(do_lisp_env("(hoge 10 ga)", &env), "E1008");

        do_lisp_env("(define hoge (lambda (a b) (+ ga b)))", &env);
        assert_str!(do_lisp_env("(hoge 10 20)", &env), "E1008");
    }
    #[test]
    fn if_f() {
        assert_str!(do_lisp("(if (<= 1 6))"), "E1007");
        assert_str!(do_lisp("(if (<= 1 6) a #\\b)"), "E1008");
        assert_str!(do_lisp("(if (<= 9 6) #\\a b)"), "E1008");
        assert_str!(do_lisp("(if 9 #\\a b)"), "E1001");
    }
    #[test]
    fn cond() {
        assert_str!(do_lisp("(cond)"), "E1007");
        assert_str!(do_lisp("(cond 10)"), "E1005");
        assert_str!(do_lisp("(cond (b 10))"), "E1008");
        assert_str!(do_lisp("(cond ((= 10 10) b))"), "E1008");
        assert_str!(do_lisp("(cond ())"), "E1012");
    }
    #[test]
    fn eqv() {
        assert_str!(do_lisp("(eqv?)"), "E1007");
        assert_str!(do_lisp("(eqv? 10 10 10)"), "E1007");
        assert_str!(do_lisp("(eq? 10 10 10)"), "E1007");
        assert_str!(do_lisp("(eq? 10 a)"), "E1008");
        assert_str!(do_lisp("(eq? a 10)"), "E1008");
    }
    #[test]
    fn case() {
        assert_str!(do_lisp("(case)"), "E1007");
        assert_str!(do_lisp("(case 10 (hoge 20))"), "E1017");
        assert_str!(do_lisp("(case 10 10)"), "E1005");
        assert_str!(do_lisp("(case 10 (20))"), "E1017");
        assert_str!(do_lisp("(case a)"), "E1008");
        assert_str!(do_lisp("(case 10 ((10 20) a))"), "E1008");
        assert_str!(do_lisp("(case 10 ((20 30) 1)(else a))"), "E1008");
    }
    #[test]
    fn apply() {
        assert_str!(do_lisp("(apply)"), "E1007");
        assert_str!(do_lisp("(apply -)"), "E1007");
        assert_str!(do_lisp("(apply + (list 1 2)(lis 3 4))"), "E1007");
        assert_str!(do_lisp("(apply + 10)"), "E1005");
        assert_str!(do_lisp("(apply hoge (list 1 2))"), "E1008");
    }
    #[test]
    fn identity() {
        assert_str!(do_lisp("(identity)"), "E1007");
        assert_str!(do_lisp("(identity 10 20)"), "E1007");
        assert_str!(do_lisp("(identity a)"), "E1008");
    }
    #[test]
    fn modulo() {
        assert_str!(do_lisp("(modulo 10)"), "E1007");
        assert_str!(do_lisp("(modulo 10 0)"), "E1013");
        assert_str!(do_lisp("(modulo 13 5.5)"), "E1002");
        assert_str!(do_lisp("(modulo 10 a)"), "E1008");
    }
    #[test]
    fn quotient() {
        assert_str!(do_lisp("(quotient 10)"), "E1007");
        assert_str!(do_lisp("(quotient 10 0)"), "E1013");
        assert_str!(do_lisp("(quotient 13 5.5)"), "E1002");
        assert_str!(do_lisp("(quotient 10 a)"), "E1008");
    }
    #[test]
    fn expt() {
        assert_str!(do_lisp("(expt 10)"), "E1007");
        assert_str!(do_lisp("(expt a 2)"), "E1008");
        assert_str!(do_lisp("(expt 10 #f)"), "E1003");
        assert_str!(do_lisp("(expt 10.5 #f)"), "E1003");
        assert_str!(do_lisp("(expt #t 10)"), "E1003");
    }
    #[test]
    fn and() {
        assert_str!(do_lisp("(and (= 1 1))"), "E1007");
        assert_str!(do_lisp("(and (= 1 1) 10)"), "E1001");
        assert_str!(do_lisp("(and a (= 1 1))"), "E1008");
    }
    #[test]
    fn or() {
        assert_str!(do_lisp("(or (= 1 1))"), "E1007");
        assert_str!(do_lisp("(or (= 1 2) 10)"), "E1001");
        assert_str!(do_lisp("(or a (= 1 2) 10)"), "E1008");
    }
    #[test]
    fn not() {
        assert_str!(do_lisp("(not)"), "E1007");
        assert_str!(do_lisp("(not 10)"), "E1001");
        assert_str!(do_lisp("(not a)"), "E1008");
    }
    #[test]
    fn let_f() {
        assert_str!(do_lisp("(let loop)"), "E1007");
        assert_str!(do_lisp("(let ((i 0 10)) (+ i 10))"), "E1007");
        assert_str!(do_lisp("(let ((100 10)) (+ i 10))"), "E1004");
        assert_str!(do_lisp("(let ((i a)) (+ i 10))"), "E1008");
        assert_str!(do_lisp("(let (10) (+ i 10))"), "E1005");
        assert_str!(do_lisp("(let 100 (+ i 10))"), "E1005");
        assert_str!(
            do_lisp("(let loop ((i 0)) (if (<= 10 i) i (loop (+ i 1)(+ i 1))))"),
            "E1007"
        );
    }
    #[test]
    fn time_f() {
        let env = lisp::Environment::new();
        assert_str!(do_lisp_env("(time)", &env), "E1007");
        assert_str!(do_lisp_env("(time 10 10)", &env), "E1007");
        assert_str!(do_lisp_env("(time c)", &env), "E1008");
    }
    #[test]
    fn set_f() {
        let env = lisp::Environment::new();
        assert_str!(do_lisp_env("(set!)", &env), "E1007");
        assert_str!(do_lisp_env("(set! c)", &env), "E1007");
        assert_str!(do_lisp_env("(set! 10 10)", &env), "E1004");
        assert_str!(do_lisp_env("(set! c 10)", &env), "E1008");
    }
    #[test]
    fn list() {
        assert_str!(do_lisp("(list c 10)"), "E1008");
    }
    #[test]
    fn make_list() {
        assert_str!(do_lisp("(make-list)"), "E1007");
        assert_str!(do_lisp("(make-list 10)"), "E1007");
        assert_str!(do_lisp("(make-list 10 0 1)"), "E1007");
        assert_str!(do_lisp("(make-list #t 0)"), "E1002");
        assert_str!(do_lisp("(make-list -1 0)"), "E1011");
        assert_str!(do_lisp("(make-list 10 c)"), "E1008");
    }
    #[test]
    fn null_f() {
        assert_str!(do_lisp("(null?)"), "E1007");
        assert_str!(do_lisp("(null? (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(null? c)"), "E1008");
    }
    #[test]
    fn length() {
        assert_str!(do_lisp("(length)"), "E1007");
        assert_str!(do_lisp("(length (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(length (cons 1 2))"), "E1005");
        assert_str!(do_lisp("(length a)"), "E1008");
    }
    #[test]
    fn car() {
        assert_str!(do_lisp("(car)"), "E1007");
        assert_str!(do_lisp("(car (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(car l)"), "E1008");
        assert_str!(do_lisp("(car (list))"), "E1011");
        assert_str!(do_lisp("(car 10)"), "E1005");
    }
    #[test]
    fn cdr() {
        assert_str!(do_lisp("(cdr)"), "E1007");
        assert_str!(do_lisp("(cdr (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(cdr (list c))"), "E1008");
        assert_str!(do_lisp("(cdr (list))"), "E1011");
        assert_str!(do_lisp("(cdr 200)"), "E1005");
    }
    #[test]
    fn cadr() {
        assert_str!(do_lisp("(cadr)"), "E1007");
        assert_str!(do_lisp("(cadr (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(cadr c)"), "E1008");
        assert_str!(do_lisp("(cadr (list 1))"), "E1011");
        assert_str!(do_lisp("(cadr 991)"), "E1005");
    }
    #[test]
    fn cons() {
        assert_str!(do_lisp("(cons)"), "E1007");
        assert_str!(do_lisp("(cons (list 1)(list 2)(list 3))"), "E1007");
        assert_str!(do_lisp("(cons a 10)"), "E1008");
    }
    #[test]
    fn append() {
        assert_str!(do_lisp("(append)"), "E1007");
        assert_str!(do_lisp("(append (list 1))"), "E1007");
        assert_str!(do_lisp("(append (list 1) 105)"), "E1005");
        assert_str!(do_lisp("(append (list 1) a)"), "E1008");
    }
    #[test]
    fn take() {
        assert_str!(do_lisp("(take)"), "E1007");
        assert_str!(do_lisp("(take (list 10 20))"), "E1007");
        assert_str!(do_lisp("(take (list 10 20) 1 2)"), "E1007");
        assert_str!(do_lisp("(take 1 (list 1 2))"), "E1005");
        assert_str!(do_lisp("(take (list 1 2) 10.5)"), "E1002");
        assert_str!(do_lisp("(take (list 1 2) 3)"), "E1011");
        assert_str!(do_lisp("(take (list 1 2) -1)"), "E1011");
        assert_str!(do_lisp("(take a 1)"), "E1008");
    }
    #[test]
    fn drop() {
        assert_str!(do_lisp("(drop)"), "E1007");
        assert_str!(do_lisp("(drop (list 10 20))"), "E1007");
        assert_str!(do_lisp("(drop (list 10 20) 1 2)"), "E1007");
        assert_str!(do_lisp("(drop 1 (list 1 2))"), "E1005");
        assert_str!(do_lisp("(drop (list 1 2) 10.5)"), "E1002");
        assert_str!(do_lisp("(drop (list 1 2) 3)"), "E1011");
        assert_str!(do_lisp("(drop (list 1 2) -1)"), "E1011");
        assert_str!(do_lisp("(drop a 1)"), "E1008");
    }
    #[test]
    fn delete() {
        assert_str!(do_lisp("(delete)"), "E1007");
        assert_str!(do_lisp("(delete 10)"), "E1007");
        assert_str!(do_lisp("(delete 10 (list 10 20) 3)"), "E1007");
        assert_str!(do_lisp("(delete 10 20)"), "E1005");
        assert_str!(do_lisp("(delete 10 a)"), "E1008");
    }
    #[test]
    fn last() {
        assert_str!(do_lisp("(last)"), "E1007");
        assert_str!(do_lisp("(last (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(last (list))"), "E1011");
        assert_str!(do_lisp("(last 29)"), "E1005");
        assert_str!(do_lisp("(last a)"), "E1008");
    }
    #[test]
    fn reverse() {
        assert_str!(do_lisp("(reverse)"), "E1007");
        assert_str!(do_lisp("(reverse (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(reverse 29)"), "E1005");
        assert_str!(do_lisp("(reverse a)"), "E1008");
    }
    #[test]
    fn iota() {
        assert_str!(do_lisp("(iota)"), "E1007");
        assert_str!(do_lisp("(iota 1 2 3 4)"), "E1007");
        assert_str!(do_lisp("(iota 1.5 2)"), "E1002");
        assert_str!(do_lisp("(iota 1 10.5)"), "E1002");
        assert_str!(do_lisp("(iota 10 1 10.5)"), "E1002");
        assert_str!(do_lisp("(iota a)"), "E1008");
    }
    #[test]
    fn map() {
        assert_str!(do_lisp("(map)"), "E1007");
        assert_str!(do_lisp("(map (lambda (n) n))"), "E1007");
        assert_str!(
            do_lisp("(map (lambda (a b) (* 10 a)) (list 1 2 3))"),
            "E1007"
        );
        assert_str!(do_lisp("(map 1 2 3)"), "E1007");
        assert_str!(do_lisp("(map (iota 10) (lambda (n) n))"), "E1006");
        assert_str!(do_lisp("(map  (lambda (n) n) 10)"), "E1005");
    }
    #[test]
    fn filter() {
        assert_str!(do_lisp("(filter)"), "E1007");
        assert_str!(do_lisp("(filter (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(filter 1 2 3)"), "E1007");
        assert_str!(
            do_lisp("(filter (lambda (a b) (= 0 a))(iota 10 1))"),
            "E1007"
        );
        assert_str!(do_lisp("(filter (iota 10) (lambda (n) n))"), "E1006");
        assert_str!(do_lisp("(filter (lambda (n) n) 10)"), "E1005");
        assert_str!(do_lisp("(filter (lambda (n) n) (iota 4))"), "E1001");
    }
    #[test]
    fn reduce() {
        assert_str!(do_lisp("(reduce)"), "E1007");
        assert_str!(do_lisp("(reduce (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(reduce 1 2 3 4)"), "E1007");
        assert_str!(do_lisp("(reduce 0 (list) (list))"), "E1006");
        assert_str!(do_lisp("(reduce (lambda (n) n) 10 10)"), "E1005");
        assert_str!(do_lisp("(reduce (lambda (n) n) 0 (iota 4))"), "E1007");
    }
    #[test]
    fn for_each() {
        assert_str!(do_lisp("(for-each)"), "E1007");
        assert_str!(do_lisp("(for-each (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(for-each 1 2 3)"), "E1007");
        assert_str!(do_lisp("(for-each (list) (list))"), "E1006");
        assert_str!(do_lisp("(for-each (lambda (n) n) 10)"), "E1005");
    }
    #[test]
    fn list_ref() {
        assert_str!(do_lisp("(list-ref)"), "E1007");
        assert_str!(do_lisp("(list-ref (iota 10))"), "E1007");
        assert_str!(do_lisp("(list-ref (iota 10) 1 2)"), "E1007");
        assert_str!(do_lisp("(list-ref 10 -1)"), "E1005");
        assert_str!(do_lisp("(list-ref (iota 10) #t)"), "E1002");

        assert_str!(do_lisp("(list-ref a #t)"), "E1008");
        assert_str!(do_lisp("(list-ref (iota 10) a)"), "E1008");

        assert_str!(do_lisp("(list-ref (iota 10) -1)"), "E1011");
        assert_str!(do_lisp("(list-ref (iota 10) 10)"), "E1011");
    }
    #[test]
    fn sqrt() {
        assert_str!(do_lisp("(sqrt)"), "E1007");
        assert_str!(do_lisp("(sqrt 10 2.5)"), "E1007");
        assert_str!(do_lisp("(sqrt #t)"), "E1003");
        assert_str!(do_lisp("(sqrt a)"), "E1008");
    }
    #[test]
    fn sin() {
        assert_str!(do_lisp("(sin)"), "E1007");
        assert_str!(do_lisp("(sin 10 2.5)"), "E1007");
        assert_str!(do_lisp("(sin #t)"), "E1003");
        assert_str!(do_lisp("(sin a)"), "E1008");
    }
    #[test]
    fn cos() {
        assert_str!(do_lisp("(cos)"), "E1007");
        assert_str!(do_lisp("(cos 10 2.5)"), "E1007");
        assert_str!(do_lisp("(cos #t)"), "E1003");
        assert_str!(do_lisp("(cos a)"), "E1008");
    }
    #[test]
    fn tan() {
        assert_str!(do_lisp("(tan)"), "E1007");
        assert_str!(do_lisp("(tan 10 2.5)"), "E1007");
        assert_str!(do_lisp("(tan #t)"), "E1003");
        assert_str!(do_lisp("(tan a)"), "E1008");
    }
    #[test]
    fn asin() {
        assert_str!(do_lisp("(asin)"), "E1007");
        assert_str!(do_lisp("(asin 10 2.5)"), "E1007");
        assert_str!(do_lisp("(asin #t)"), "E1003");
        assert_str!(do_lisp("(asin a)"), "E1008");
    }
    #[test]
    fn acos() {
        assert_str!(do_lisp("(acos)"), "E1007");
        assert_str!(do_lisp("(acos 10 2.5)"), "E1007");
        assert_str!(do_lisp("(acos #t)"), "E1003");
        assert_str!(do_lisp("(acos a)"), "E1008");
    }
    #[test]
    fn atan() {
        assert_str!(do_lisp("(atan)"), "E1007");
        assert_str!(do_lisp("(atan 10 2.5)"), "E1007");
        assert_str!(do_lisp("(atan #t)"), "E1003");
        assert_str!(do_lisp("(atan a)"), "E1008");
    }
    #[test]
    fn exp() {
        assert_str!(do_lisp("(exp)"), "E1007");
        assert_str!(do_lisp("(exp 10 2.5)"), "E1007");
        assert_str!(do_lisp("(exp #t)"), "E1003");
        assert_str!(do_lisp("(exp a)"), "E1008");
    }
    #[test]
    fn log() {
        assert_str!(do_lisp("(log)"), "E1007");
        assert_str!(do_lisp("(log 10 2.5)"), "E1007");
        assert_str!(do_lisp("(log #t)"), "E1003");
        assert_str!(do_lisp("(log a)"), "E1008");
    }
    #[test]
    fn truncate() {
        assert_str!(do_lisp("(truncate)"), "E1007");
        assert_str!(do_lisp("(truncate 10 2.5)"), "E1007");
        assert_str!(do_lisp("(truncate #t)"), "E1003");
        assert_str!(do_lisp("(truncate a)"), "E1008");
    }
    #[test]
    fn floor() {
        assert_str!(do_lisp("(floor)"), "E1007");
        assert_str!(do_lisp("(floor 10 2.5)"), "E1007");
        assert_str!(do_lisp("(floor #t)"), "E1003");
        assert_str!(do_lisp("(floor a)"), "E1008");
    }
    #[test]
    fn ceiling() {
        assert_str!(do_lisp("(ceiling)"), "E1007");
        assert_str!(do_lisp("(ceiling 10 2.5)"), "E1007");
        assert_str!(do_lisp("(ceiling #t)"), "E1003");
        assert_str!(do_lisp("(ceiling a)"), "E1008");
    }
    #[test]
    fn round() {
        assert_str!(do_lisp("(round)"), "E1007");
        assert_str!(do_lisp("(round 10 2.5)"), "E1007");
        assert_str!(do_lisp("(round #t)"), "E1003");
        assert_str!(do_lisp("(round a)"), "E1008");
    }
    #[test]
    fn abs() {
        assert_str!(do_lisp("(abs)"), "E1007");
        assert_str!(do_lisp("(abs 10 2.5)"), "E1007");
        assert_str!(do_lisp("(abs #t)"), "E1003");
        assert_str!(do_lisp("(abs a)"), "E1008");
    }
    #[test]
    fn rand_integer() {
        assert_str!(do_lisp("(rand-integer 10)"), "E1007");
    }
    #[test]
    fn rand_list() {
        assert_str!(do_lisp("(rand-list)"), "E1007");
        assert_str!(do_lisp("(rand-list 1 2)"), "E1007");
        assert_str!(do_lisp("(rand-list 10.5)"), "E1002");
    }
    #[test]
    fn load_file() {
        assert_str!(do_lisp("(load-file)"), "E1007");
        assert_str!(do_lisp("(load-file 1 2)"), "E1007");
        assert_str!(do_lisp("(load-file hoge)"), "E1008");
        assert_str!(do_lisp("(load-file #t)"), "E1015");
        assert_str!(do_lisp("(load-file \"/etc/test.scm\")"), "E1014");
        assert_str!(do_lisp("(load-file \"/tmp\")"), "E1016");
        assert_str!(do_lisp("(load-file \"/bin/cp\")"), "E9999");
    }
    #[test]
    fn delay_force() {
        assert_str!(do_lisp("(delay)"), "E1007");
        assert_str!(do_lisp("(delay 1 2)"), "E1007");
        assert_str!(do_lisp("(force)"), "E1007");
        assert_str!(do_lisp("(force 1 2)"), "E1007");
        assert_str!(do_lisp("(force hoge)"), "E1008");
    }
    #[test]
    fn display() {
        assert_str!(do_lisp("(display)"), "E1007");
        assert_str!(do_lisp("(display a)"), "E1008");
    }
    #[test]
    fn newline() {
        assert_str!(do_lisp("(newline 123)"), "E1007");
    }
    #[test]
    fn begin() {
        assert_str!(do_lisp("(begin)"), "E1007");
        assert_str!(do_lisp("(begin a)"), "E1008");
    }
    #[test]
    fn sequence() {
        assert_str!(
            do_lisp("(let ((i 0)) (define i 100) (set! i (+ i b)) i)"),
            "E1008"
        );
        assert_str!(
            do_lisp("((lambda (a) (define i 100) (set! i (+ i b)) i)10)"),
            "E1008"
        );
    }
    #[test]
    fn format_f() {
        assert_str!(do_lisp("(format)"), "E1007");
        assert_str!(do_lisp("(format \"~B\")"), "E1007");
        assert_str!(do_lisp("(format \"~B\" 10 12)"), "E1007");
        assert_str!(do_lisp("(format 10 12)"), "E1015");
        assert_str!(do_lisp("(format \"~A\" #f)"), "E1002");
        assert_str!(do_lisp("(format \"~A\" 10)"), "E1018");
    }
    #[test]
    fn force_stop() {
        assert_str!(do_lisp("(force-stop 10)"), "E1008");
    }
    #[test]
    fn set_tail_recursion() {
        assert_str!(do_lisp("(tail-recursion-off 20)"), "E1008");
        assert_str!(do_lisp("(tail-recursion-on 30)"), "E1008");
    }
    #[test]
    fn string_eq() {
        assert_str!(do_lisp("(string=?)"), "E1007");
        assert_str!(do_lisp("(string=? \"abc\")"), "E1007");
        assert_str!(do_lisp("(string=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_str!(do_lisp("(string=? \"abc\" 10)"), "E1015");
        assert_str!(do_lisp("(string=? 10 \"abc\")"), "E1015");
        assert_str!(do_lisp("(string=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_less() {
        assert_str!(do_lisp("(string<?)"), "E1007");
        assert_str!(do_lisp("(string<? \"abc\")"), "E1007");
        assert_str!(do_lisp("(string<? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_str!(do_lisp("(string<? \"abc\" 10)"), "E1015");
        assert_str!(do_lisp("(string<? 10 \"abc\")"), "E1015");
        assert_str!(do_lisp("(string<? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_than() {
        assert_str!(do_lisp("(string>?)"), "E1007");
        assert_str!(do_lisp("(string>? \"abc\")"), "E1007");
        assert_str!(do_lisp("(string>? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_str!(do_lisp("(string>? \"abc\" 10)"), "E1015");
        assert_str!(do_lisp("(string>? 10 \"abc\")"), "E1015");
        assert_str!(do_lisp("(string>? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_le() {
        assert_str!(do_lisp("(string<=?)"), "E1007");
        assert_str!(do_lisp("(string<=? \"abc\")"), "E1007");
        assert_str!(do_lisp("(string<=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_str!(do_lisp("(string<=? \"abc\" 10)"), "E1015");
        assert_str!(do_lisp("(string<=? 10 \"abc\")"), "E1015");
        assert_str!(do_lisp("(string<=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn string_ge() {
        assert_str!(do_lisp("(string>=?)"), "E1007");
        assert_str!(do_lisp("(string>=? \"abc\")"), "E1007");
        assert_str!(do_lisp("(string>=? \"abc\" \"ABC\" \"DEF\")"), "E1007");
        assert_str!(do_lisp("(string>=? \"abc\" 10)"), "E1015");
        assert_str!(do_lisp("(string>=? 10 \"abc\")"), "E1015");
        assert_str!(do_lisp("(string>=? \"abc\" a)"), "E1008");
    }
    #[test]
    fn char_eq() {
        assert_str!(do_lisp("(char=?)"), "E1007");
        assert_str!(do_lisp("(char=? #\\a)"), "E1007");
        assert_str!(do_lisp("(char=? #\\a #\\b #\\c)"), "E1007");
        assert_str!(do_lisp("(char=? #\\a 10)"), "E1019");
        assert_str!(do_lisp("(char=? 10 #\\a)"), "E1019");
        assert_str!(do_lisp("(char=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_less() {
        assert_str!(do_lisp("(char<?)"), "E1007");
        assert_str!(do_lisp("(char<? #\\a)"), "E1007");
        assert_str!(do_lisp("(char<? #\\a #\\b #\\c)"), "E1007");
        assert_str!(do_lisp("(char<? #\\a 10)"), "E1019");
        assert_str!(do_lisp("(char<? 10 #\\a)"), "E1019");
        assert_str!(do_lisp("(char<? #\\a a)"), "E1008");
    }
    #[test]
    fn char_than() {
        assert_str!(do_lisp("(char>?)"), "E1007");
        assert_str!(do_lisp("(char>? #\\a)"), "E1007");
        assert_str!(do_lisp("(char>? #\\a #\\b #\\c)"), "E1007");
        assert_str!(do_lisp("(char>? #\\a 10)"), "E1019");
        assert_str!(do_lisp("(char>? 10 #\\a)"), "E1019");
        assert_str!(do_lisp("(char>? #\\a a)"), "E1008");
    }
    #[test]
    fn char_le() {
        assert_str!(do_lisp("(char<=?)"), "E1007");
        assert_str!(do_lisp("(char<=? #\\a)"), "E1007");
        assert_str!(do_lisp("(char<=? #\\a #\\b #\\c)"), "E1007");
        assert_str!(do_lisp("(char<=? #\\a 10)"), "E1019");
        assert_str!(do_lisp("(char<=? 10 #\\a)"), "E1019");
        assert_str!(do_lisp("(char<=? #\\a a)"), "E1008");
    }
    #[test]
    fn char_ge() {
        assert_str!(do_lisp("(char>=?)"), "E1007");
        assert_str!(do_lisp("(char>=? #\\a)"), "E1007");
        assert_str!(do_lisp("(char>=? #\\a #\\b #\\c)"), "E1007");
        assert_str!(do_lisp("(char>=? #\\a 10)"), "E1019");
        assert_str!(do_lisp("(char>=? 10 #\\a)"), "E1019");
        assert_str!(do_lisp("(char>=? #\\a a)"), "E1008");
    }
    #[test]
    fn str_append() {
        assert_str!(do_lisp("(string-append)"), "E1007");
        assert_str!(do_lisp("(string-append \"a\")"), "E1007");
        assert_str!(do_lisp("(string-append \"a\" 10)"), "E1015");
        assert_str!(do_lisp("(string-append \"a\" a)"), "E1008");
    }
    #[test]
    fn string_length() {
        assert_str!(do_lisp("(string-length)"), "E1007");
        assert_str!(do_lisp("(string-length \"1234\" \"12345\")"), "E1007");
        assert_str!(do_lisp("(string-length 1000)"), "E1015");
    }
    #[test]
    fn string_size() {
        assert_str!(do_lisp("(string-size)"), "E1007");
        assert_str!(do_lisp("(string-size \"1234\" \"12345\")"), "E1007");
        assert_str!(do_lisp("(string-size 1000)"), "E1015");
    }
    #[test]
    fn number_string() {
        assert_str!(do_lisp("(number->string)"), "E1007");
        assert_str!(do_lisp("(number->string 10 20)"), "E1007");
        assert_str!(do_lisp("(number->string #f)"), "E1003");
        assert_str!(do_lisp("(number->string a)"), "E1008");
    }
    #[test]
    fn string_number() {
        assert_str!(do_lisp("(string->number)"), "E1007");
        assert_str!(do_lisp("(string->number \"123\" \"10.5\")"), "E1007");
        assert_str!(do_lisp("(string->number 100)"), "E1015");
        assert_str!(do_lisp("(string->number \"/1\")"), "E1003");
        assert_str!(do_lisp("(string->number \"1/3/2\")"), "E1003");
        assert_str!(do_lisp("(string->number \"1/0\")"), "E1013");
        assert_str!(do_lisp("(string->number a)"), "E1008");
    }
    #[test]
    fn list_string() {
        assert_str!(do_lisp("(list->string)"), "E1007");
        assert_str!(
            do_lisp("(list->string (list #\\a #\\b)(list #\\a #\\b))"),
            "E1007"
        );
        assert_str!(do_lisp("(list->string 10)"), "E1005");
        assert_str!(do_lisp("(list->string (list #\\a 10))"), "E1019");
        assert_str!(do_lisp("(list->string a)"), "E1008");
    }
    #[test]
    fn substring() {
        assert_str!(do_lisp("(substring)"), "E1007");
        assert_str!(do_lisp("(substring \"1234567890\" 1)"), "E1007");
        assert_str!(do_lisp("(substring \"1234567890\" 1 2 3)"), "E1007");
        assert_str!(do_lisp("(substring  1 2 3)"), "E1015");
        assert_str!(do_lisp("(substring \"1234567890\" #t 2)"), "E1002");
        assert_str!(do_lisp("(substring \"1234567890\" 0 #t)"), "E1002");
        assert_str!(do_lisp("(substring \"1234567890\" a 2)"), "E1008");
        assert_str!(do_lisp("(substring \"1234567890\" 0 a)"), "E1008");

        assert_str!(do_lisp("(substring \"1234567890\" -1 2)"), "E1021");
        assert_str!(do_lisp("(substring \"1234567890\" 0 -2)"), "E1021");
        assert_str!(do_lisp("(substring \"1234567890\" 0 11)"), "E1021");
        assert_str!(do_lisp("(substring \"1234567890\" 6 5)"), "E1021");

        assert_str!(do_lisp("(substring \"山\" 0 2)"), "E1021");
    }
    #[test]
    fn symbol_string() {
        assert_str!(do_lisp("(symbol->string)"), "E1007");
        assert_str!(do_lisp("(symbol->string 'a 'b)"), "E1007");
        assert_str!(do_lisp("(symbol->string #t)"), "E1004");
    }
    #[test]
    fn string_symbol() {
        assert_str!(do_lisp("(string->symbol)"), "E1007");
        assert_str!(do_lisp("(string->symbol \"abc\"  \"def\")"), "E1007");
        assert_str!(do_lisp("(string->symbol #t)"), "E1015");
    }

    #[test]
    fn string_list() {
        assert_str!(do_lisp("(string->list)"), "E1007");
        assert_str!(do_lisp("(string->list \"a\" \"b\")"), "E1007");
        assert_str!(do_lisp("(string->list #\\a)"), "E1015");
        assert_str!(do_lisp("(string->list a)"), "E1008");
    }
    #[test]
    fn make_string() {
        assert_str!(do_lisp("(make-string)"), "E1007");
        assert_str!(do_lisp("(make-string a)"), "E1007");
        assert_str!(do_lisp("(make-string a a a)"), "E1007");

        assert_str!(do_lisp("(make-string #t #\\a)"), "E1002");
        assert_str!(do_lisp("(make-string -1 #\\a)"), "E1021");
        assert_str!(do_lisp("(make-string 4 a)"), "E1008");
        assert_str!(do_lisp("(make-string 4 #t)"), "E1019");
    }
    #[test]
    fn integer_char() {
        assert_str!(do_lisp("(integer->char)"), "E1007");
        assert_str!(do_lisp("(integer->char 23 665)"), "E1007");
        assert_str!(do_lisp("(integer->char #\\a)"), "E1002");
        assert_str!(do_lisp("(integer->char -999)"), "E1019");
        assert_str!(do_lisp("(integer->char a)"), "E1008");
    }
    #[test]
    fn char_integer() {
        assert_str!(do_lisp("(char->integer)"), "E1007");
        assert_str!(do_lisp("(char->integer #\\a #\\b)"), "E1007");
        assert_str!(do_lisp("(char->integer 999)"), "E1019");
        assert_str!(do_lisp("(char->integer a)"), "E1008");
    }
    #[test]
    fn quote() {
        assert_str!(do_lisp("(quote)"), "E1007");
        assert_str!(do_lisp("(quote 1 2)"), "E1007");
    }
    #[test]
    fn get_env() {
        assert_str!(do_lisp("(get-environment-variable)"), "E1007");
        assert_str!(
            do_lisp("(get-environment-variable  \"HOME\"  \"HOME\")"),
            "E1007"
        );
        assert_str!(do_lisp("(get-environment-variable a)"), "E1008");
        assert_str!(do_lisp("(get-environment-variable #t)"), "E1015");
    }
}
