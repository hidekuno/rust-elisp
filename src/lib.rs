// cargo test --lib
#[macro_use]
extern crate lazy_static;

pub mod dyn_lisp;
pub mod lisp;
use crate::lisp::EvalResult;

#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::path::Path;
//        assert_str!(do_lisp(""), "");
//        let mut env = lisp::SimpleEnv::new();
//        assert_str!(do_lisp_env("", &mut env), "");
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
#[allow(unused_macros)]
macro_rules! assert_str {
    ($a: expr,
     $b: expr) => {
        assert!($a == $b.to_string())
    };
}
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn atom() {
        assert_str!(do_lisp("10"), "10");
        assert_str!(do_lisp("10.5"), "10.5");
        assert_str!(do_lisp("#t"), "#t");
        assert_str!(do_lisp("#\\a"), "a");
        assert_str!(do_lisp("\"abc\""), "\"abc\"");
    }
    #[test]
    fn plus() {
        assert_str!(do_lisp("(+ 1 2)"), "3");
        assert_str!(do_lisp("(+ 1.25 2.25)"), "3.5");
        assert_str!(do_lisp("(+ 1 2.5)"), "3.5");
        assert_str!(do_lisp("(+ 3 1.5)"), "4.5");
        assert_str!(do_lisp("(+ (* 1 2)(* 3 4))"), "14");
    }
    #[test]
    fn minus() {
        assert_str!(do_lisp("(- 6 1)"), "5");
        assert_str!(do_lisp("(- 5.75 1.5)"), "4.25");
        assert_str!(do_lisp("(- 6 1.5)"), "4.5");
        assert_str!(do_lisp("(- 6.5 3)"), "3.5");
        assert_str!(do_lisp("(- (* 3 4)(* 1 2))"), "10");
    }
    #[test]
    fn multi() {
        assert_str!(do_lisp("(* 3 6)"), "18");
        assert_str!(do_lisp("(* 0.5 5.75)"), "2.875");
        assert_str!(do_lisp("(* 3.5 6)"), "21");
        assert_str!(do_lisp("(* 6 3.5)"), "21");
        assert_str!(do_lisp("(* (+ 3 4)(+ 1 2))"), "21");
    }
    #[test]
    fn div() {
        assert_str!(do_lisp("(/ 9 3)"), "3");
        assert_str!(do_lisp("(/ 0.75 0.25)"), "3");
        assert_str!(do_lisp("(/ 9.5 5)"), "1.9");
        assert_str!(do_lisp("(/ 6 2.5)"), "2.4");
        assert_str!(do_lisp("(/ 0 0)"), "NaN");
        assert_str!(do_lisp("(/ 9 0)"), "inf");
        assert_str!(do_lisp("(/ 10 0.0)"), "inf");
        assert_str!(do_lisp("(/ 0 9)"), "0");
        assert_str!(do_lisp("(/ 0.0 9)"), "0");
        assert_str!(do_lisp("(/ (+ 4 4)(+ 2 2))"), "2");
    }
    #[test]
    fn eq() {
        assert_str!(do_lisp("(= 5 5)"), "#t");
        assert_str!(do_lisp("(= 5.5 5.5)"), "#t");
        assert_str!(do_lisp("(= 5 5.0)"), "#t");
        assert_str!(do_lisp("(= 5 6)"), "#f");
        assert_str!(do_lisp("(= 5.5 6.6)"), "#f");
        assert_str!(do_lisp("(= (+ 1 1)(+ 0 2))"), "#t");
    }
    #[test]
    fn than() {
        assert_str!(do_lisp("(> 6 5)"), "#t");
        assert_str!(do_lisp("(> 6 6)"), "#f");
        assert_str!(do_lisp("(> 6.5 5.5)"), "#t");
        assert_str!(do_lisp("(> 4.5 5.5)"), "#f");
        assert_str!(do_lisp("(> (+ 3 3) 5)"), "#t");
    }
    #[test]
    fn less() {
        assert_str!(do_lisp("(< 5 6)"), "#t");
        assert_str!(do_lisp("(< 5.6 6.5)"), "#t");
        assert_str!(do_lisp("(> 6 6)"), "#f");
        assert_str!(do_lisp("(> 6.5 6.6)"), "#f");
        assert_str!(do_lisp("(< 5 (+ 3 3))"), "#t");
    }
    #[test]
    fn than_eq() {
        assert_str!(do_lisp("(>= 6 6)"), "#t");
        assert_str!(do_lisp("(>= 7.6 7.6)"), "#t");
        assert_str!(do_lisp("(>= 6 5)"), "#t");
        assert_str!(do_lisp("(>= 6.3 5.2)"), "#t");
        assert_str!(do_lisp("(>= 5 6)"), "#f");
        assert_str!(do_lisp("(>= 5.1 6.2)"), "#f");
        assert_str!(do_lisp("(>= (+ 2 3 1) 6)"), "#t");
    }
    #[test]
    fn less_eq() {
        assert_str!(do_lisp("(<= 6 6)"), "#t");
        assert_str!(do_lisp("(<= 6.1 6.1)"), "#t");
        assert_str!(do_lisp("(<= 5 6)"), "#t");
        assert_str!(do_lisp("(<= 5.2 6.9)"), "#t");
        assert_str!(do_lisp("(<= 6 5)"), "#f");
        assert_str!(do_lisp("(<= 8.6 5.4)"), "#f");
        assert_str!(do_lisp("(<= (+ 3 3) 6)"), "#t");
    }
    #[test]
    fn define() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        assert_str!(do_lisp_env("a", &mut env), "100");
        do_lisp_env("(define a 10.5)", &mut env);
        assert_str!(do_lisp_env("a", &mut env), "10.5");
        do_lisp_env("(define a #t)", &mut env);
        assert_str!(do_lisp_env("a", &mut env), "#t");
        do_lisp_env("(define a #\\A)", &mut env);
        assert_str!(do_lisp_env("a", &mut env), "A");

        do_lisp_env("(define (fuga a b)(* a b))", &mut env);
        assert_str!(do_lisp_env("(fuga 6 8)", &mut env), "48");
        do_lisp_env("(define (hoge a b) a)", &mut env);
        assert_str!(do_lisp_env("(hoge 6 8)", &mut env), "6");

        do_lisp_env("(define a 100)", &mut env);
        do_lisp_env("(define b a)", &mut env);
        assert_str!(do_lisp_env("b", &mut env), "100");
    }
    #[test]
    fn lambda() {
        assert_str!(do_lisp("((lambda (a b)(+ a b)) 1 2)"), "3");

        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &mut env);
        assert_str!(do_lisp_env("(hoge 6 8)", &mut env), "14");
        do_lisp_env("(define hoge (lambda (a b) b))", &mut env);
        assert_str!(do_lisp_env("(hoge 6 8)", &mut env), "8");
    }
    #[test]
    fn if_f() {
        assert_str!(do_lisp("(if (= 10 10) #\\a)"), "a");
        assert_str!(do_lisp("(if (= 10 11) #\\a)"), "nil");
        assert_str!(do_lisp("(if (<= 1 6) #\\a #\\b)"), "a");
        assert_str!(do_lisp("(if (<= 9 6) #\\a #\\b)"), "b");
    }
    #[test]
    fn modulo() {
        assert_str!(do_lisp("(modulo 11 3)"), "2");
        assert_str!(do_lisp("(modulo 11 (+ 1 2))"), "2");
        assert_str!(do_lisp("(modulo  3 5)"), "3");
    }
    #[test]
    fn expt() {
        assert_str!(do_lisp("(expt 2 3)"), "8");
        assert_str!(do_lisp("(expt 2 (+ 1 2))"), "8");
        assert_str!(do_lisp("(expt 2 -2)"), "0.25");
        assert_str!(do_lisp("(expt 2 0)"), "1");
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
            do_lisp("(let loop ((i 0)) (if (<= 10 i) i (+ 10 (loop (+ i 1)))))"),
            "110"
        );
        // stack overflow
        assert_str!(
            do_lisp("(let loop ((i 0)) (if (<= 10000 i) i (loop (+ i 1))))"),
            "10000"
        );
    }
    #[test]
    fn set_f() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define c 0)", &mut env);
        do_lisp_env("(set! c 10)", &mut env);
        assert_str!(do_lisp_env("c", &mut env), "10");
        do_lisp_env("(set! c (+ c 1))", &mut env);
        assert_str!(do_lisp_env("c", &mut env), "11");
    }
    #[test]
    fn closure() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env(
            "(define (counter) (let ((c 0)) (lambda () (set! c (+ 1 c)) c)))",
            &mut env,
        );
        do_lisp_env("(define a (counter))", &mut env);
        do_lisp_env("(define b (counter))", &mut env);
        for _i in 0..10 {
            do_lisp_env("(a)", &mut env);
        }
        for _i in 0..5 {
            do_lisp_env("(b)", &mut env);
        }
        assert_str!(do_lisp_env("(a)", &mut env), "11");
        assert_str!(do_lisp_env("(b)", &mut env), "6");

        do_lisp_env(
            "(define (scounter step) (let ((c 0)) (lambda () (set! c (+ step c)) c)))",
            &mut env,
        );
        do_lisp_env("(define x (scounter 10))", &mut env);
        do_lisp_env("(define y (scounter 100))", &mut env);
        for _i in 0..2 {
            do_lisp_env("(x)", &mut env);
            do_lisp_env("(y)", &mut env);
        }
        assert_str!(do_lisp_env("(x)", &mut env), "30");
        assert_str!(do_lisp_env("(y)", &mut env), "300");
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
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 10)", &mut env);
        do_lisp_env("(define b 20)", &mut env);
        assert_str!(do_lisp_env("(list a b)", &mut env), "(10 20)");
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
    fn last() {
        assert_str!(do_lisp("(last (list 1))"), "1");
        assert_str!(do_lisp("(last (list 1 2))"), "2");
        assert_str!(do_lisp("(last (cons 1 2))"), "1");
    }
    #[test]
    fn reverse() {
        assert_str!(do_lisp("(reverse (list 10))"), "(10)");
        //        assert_str!(do_lisp(""), "");
        //        assert_str!(do_lisp(""), "");
        assert_str!(do_lisp("(reverse (iota 10))"), "(9 8 7 6 5 4 3 2 1 0)");
        assert_str!(do_lisp("(reverse (list))"), "()");
    }
    #[test]
    fn iota() {
        assert_str!(do_lisp("(iota 10)"), "(0 1 2 3 4 5 6 7 8 9)");
        assert_str!(do_lisp("(iota 10 1)"), "(1 2 3 4 5 6 7 8 9 10)");
        assert_str!(do_lisp("(iota 1 10)"), "(10)");
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
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        do_lisp_env("(define b 200)", &mut env);
        do_lisp_env("(define c 300)", &mut env);
        do_lisp_env(
            "(define d (list (list (list 1))(list (list 2))(list (list 3))))",
            &mut env,
        );

        assert_str!(
            do_lisp_env(
                "(map (lambda (n)(map (lambda (m)(/ m 10)) n))(list (list 10 20 30)(list a b c)))",
                &mut env
            ),
            "((1 2 3)(10 20 30))"
        );
        assert_str!(
            do_lisp_env("(map (lambda (n) (car n)) d)", &mut env),
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

        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        do_lisp_env("(define b 200)", &mut env);
        do_lisp_env("(define c 300)", &mut env);
        assert_str!(
            do_lisp_env("(filter (lambda (n) (= n 100)) (list a b c))", &mut env),
            "(100)"
        );
        assert_str!(
            do_lisp_env(
                "(filter (lambda (n) (not (= n 100))) (list a b c))",
                &mut env
            ),
            "(200 300)"
        );
    }
    #[test]
    fn reduce() {
        assert_str!(do_lisp("(reduce (lambda (a b) (+ a b))(list 1 2 3))"), "6");
        assert_str!(
            do_lisp("(reduce (lambda (a b) (append a b))(list (list 1) (list 2) (list 3)))"),
            "(1 2 3)"
        );
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        do_lisp_env("(define b 200)", &mut env);
        do_lisp_env("(define c 300)", &mut env);
        assert_str!(
            do_lisp_env("(reduce (lambda (a b) (+ a b))(list a b c))", &mut env),
            "600"
        );
    }
    #[test]
    fn for_each() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define c 0)", &mut env);
        do_lisp_env("(for-each (lambda (n) (set! c (+ c n)))(iota 5))", &mut env);
        assert_str!(do_lisp_env("c", &mut env), "10");
    }
    #[test]
    fn sqrt() {
        assert_str!(do_lisp("(sqrt 9)"), "3");
        assert_str!(do_lisp("(sqrt 25.0)"), "5");

        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 16)", &mut env);
        assert_str!(do_lisp_env("(sqrt a)", &mut env), "4");
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
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a (/(* 30 (* 4 (atan 1))) 180))", &mut env);
        assert_str!(do_lisp_env("(sin a)", &mut env), "0.49999999999999994");
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
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a (/(* 60 (* 4 (atan 1))) 180))", &mut env);
        assert_str!(do_lisp_env("(cos a)", &mut env), "0.5000000000000001");
    }
    #[test]
    fn tan() {
        assert_str!(
            do_lisp("(tan (/(* 45 (* 4 (atan 1))) 180))"),
            "0.9999999999999999"
        );
        assert_str!(
            do_lisp("(tan (/(* 45.025 (* 4 (atan 1))) 180))"),
            "1.0008730456194168"
        );
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a (/(* 45 (* 4 (atan 1))) 180))", &mut env);
        assert_str!(do_lisp_env("(tan a)", &mut env), "0.9999999999999999");
    }
    #[test]
    fn atan() {
        assert_str!(do_lisp("(* 4 (atan 1))"), "3.141592653589793");
        assert_str!(do_lisp("(* 4 (atan 1.0))"), "3.141592653589793");
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 1)", &mut env);
        assert_str!(do_lisp_env("(* 4 (atan a))", &mut env), "3.141592653589793");
    }
    #[test]
    fn exp() {
        assert_str!(do_lisp("(exp 1)"), "2.718281828459045");
        assert_str!(do_lisp("(exp 1.025)"), "2.7870954605658507");
        assert_str!(do_lisp("(exp 2)"), "7.38905609893065");

        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 3)", &mut env);
        assert_str!(do_lisp_env("(exp a)", &mut env), "20.085536923187668");
    }
    #[test]
    fn log() {
        assert_str!(do_lisp("(/(log 8)(log 2))"), "3");
        assert_str!(do_lisp("(/(log 9.0)(log 3.0))"), "2");
        assert_str!(do_lisp("(exp (/(log 8) 3))"), "2");
        assert_str!(do_lisp("(exp (* (log 2) 3))"), "7.999999999999998");

        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 9)", &mut env);
        do_lisp_env("(define b 3)", &mut env);
        assert_str!(do_lisp_env("(/(log a)(log b))", &mut env), "2");
    }
    #[test]
    fn load_file() {
        let test_file = Path::new(&env::var("HOME").unwrap())
            .join("tmp")
            .join("test.scm");

        match std::fs::remove_file(&test_file) {
            Err(_) => (),
            _ => (),
        }
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "(define foo 100)").unwrap();
        writeln!(file, "(define hoge 200)").unwrap();
        writeln!(file, "(define fuga (+ foo hoge))").unwrap();
        file.flush().unwrap();

        let mut env = lisp::SimpleEnv::new();
        let f = test_file.as_path().to_str().expect("die");
        do_lisp_env(
            format!("(load-file \"{}\")", f.to_string()).as_str(),
            &mut env,
        );
        assert_str!(do_lisp_env("foo", &mut env), "100");
        assert_str!(do_lisp_env("hoge", &mut env), "200");
        assert_str!(do_lisp_env("fuga", &mut env), "300");
    }
    #[test]
    fn sample_program() {
        let program = ["(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (gcm m mod))))",
                       "(define (bad-gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (+ 0 (bad-gcm m mod)))))",
                       "(define (lcm n m) (/(* n m)(gcm n m)))",
                       "(define prime (lambda (l) (if (> (car l)(sqrt (last l))) l (cons (car l)(prime (filter (lambda (n) (not (= 0 (modulo n (car l))))) (cdr l)))))))",
                       "(define qsort (lambda (l pred) (if (null? l) l (append (qsort (filter (lambda (n) (pred n (car l))) (cdr l)) pred) (cons (car l) (qsort (filter (lambda (n) (not (pred n (car l))))(cdr l)) pred))))))",
                       "(define comb (lambda (l n) (if (null? l) l (if (= n 1) (map (lambda (n) (list n)) l) (append (map (lambda (p) (cons (car l) p)) (comb (cdr l)(- n 1))) (comb (cdr l) n))))))",
                       "(define delete (lambda (x l) (filter (lambda (n) (not (= x n))) l)))",
                       "(define perm (lambda (l n)(if (>= 0 n) (list (list))(reduce (lambda (a b)(append a b))(map (lambda (x) (map (lambda (p) (cons x p)) (perm (delete x l)(- n 1)))) l)))))",
                       "(define bubble-iter (lambda (x l)(if (or (null? l)(< x (car l)))(cons x l)(cons (car l)(bubble-iter x (cdr l))))))",
                       "(define bsort (lambda (l)(if (null? l) l (bubble-iter (car l)(bsort (cdr l))))))",
                       "(define take (lambda (l n)(if (>= 0 n) (list)(cons (car l)(take (cdr l)(- n 1))))))",
                       "(define drop (lambda (l n)(if (>= 0 n) l (drop (cdr l)(- n 1)))))",
                       "(define merge (lambda (a b)(if (or (null? a)(null? b)) (append a b) (if (< (car a)(car b))(cons (car a)(merge (cdr a) b))(cons (car b) (merge a (cdr b)))))))",
                       "(define msort (lambda (l)(let ((n (length l)))(if (>= 1 n ) l (if (= n 2) (if (< (car l)(cadr l)) l (reverse l))(let ((mid (quotient n 2)))(merge (msort (take l mid))(msort (drop l mid)))))))))",
                       "(define test-list (list 36 27 14 19 2 8 7 6 0 9 3))"];

        let mut env = lisp::SimpleEnv::new();
        for p in &program {
            do_lisp_env(p, &mut env);
        }
        assert_str!(do_lisp_env("(gcm 36 27)", &mut env), "9");
        assert_str!(do_lisp_env("(lcm 36 27)", &mut env), "108");
        assert_str!(do_lisp_env("(bad-gcm 36 27)", &mut env), "9");
        assert_str!(
            do_lisp_env("(prime (iota 30 2))", &mut env),
            "(2 3 5 7 11 13 17 19 23 29 31)"
        );
        assert_str!(
            do_lisp_env("(perm (list 1 2 3) 2)", &mut env),
            "((1 2)(1 3)(2 1)(2 3)(3 1)(3 2))"
        );
        assert_str!(
            do_lisp_env("(comb (list 1 2 3) 2)", &mut env),
            "((1 2)(1 3)(2 3))"
        );
        assert_str!(
            do_lisp_env("(merge (list 1 3 5 7 9)(list 2 4 6 8 10))", &mut env),
            "(1 2 3 4 5 6 7 8 9 10)"
        );
        assert_str!(
            do_lisp_env("(take (list 2 4 6 8 10) 3)", &mut env),
            "(2 4 6)"
        );
        assert_str!(
            do_lisp_env("(drop (list 2 4 6 8 10) 3)", &mut env),
            "(8 10)"
        );
        assert_str!(
            do_lisp_env("(qsort test-list (lambda (a b)(< a b)))", &mut env),
            "(0 2 3 6 7 8 9 14 19 27 36)"
        );
        assert_str!(
            do_lisp_env("(qsort test-list (lambda (a b)(> a b)))", &mut env),
            "(36 27 19 14 9 8 7 6 3 2 0)"
        );
        assert_str!(
            do_lisp_env("(bsort test-list)", &mut env),
            "(0 2 3 6 7 8 9 14 19 27 36)"
        );
        assert_str!(
            do_lisp_env("(msort test-list)", &mut env),
            "(0 2 3 6 7 8 9 14 19 27 36)"
        );
    }
}
mod error_tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn atom() {
        assert_str!(do_lisp("\"a"), "E0001");
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
    fn define() {
        let mut env = lisp::SimpleEnv::new();
        assert_str!(do_lisp_env("(define)", &mut env), "E1007");
        assert_str!(do_lisp_env("(define a)", &mut env), "E1007");
        assert_str!(do_lisp_env("(define 1 10)", &mut env), "E1004");
        assert_str!(
            do_lisp_env("(define (hoge a 1) (+ 100 a))", &mut env),
            "E1004"
        );
        assert_str!(
            do_lisp_env("(define (hoge 1 a) (+ 100 a))", &mut env),
            "E1004"
        );
        assert_str!(
            do_lisp_env("(define (100 a b) (+ 100 a))", &mut env),
            "E1004"
        );
        assert_str!(do_lisp_env("(define () (+ 100 a))", &mut env), "E1007");

        assert_str!(do_lisp_env("(define a ga)", &mut env), "E1008");
    }
    #[test]
    fn lambda() {
        let mut env = lisp::SimpleEnv::new();
        assert_str!(do_lisp_env("(lambda)", &mut env), "E1007");
        assert_str!(do_lisp_env("(lambda (a b))", &mut env), "E1007");
        assert_str!(do_lisp_env("(lambda  a (+ a b))", &mut env), "E1005");
        assert_str!(do_lisp_env("(lambda (a 1) (+ a 10))", &mut env), "E1004");
        assert_str!(do_lisp_env("((list 1) 10)", &mut env), "E1006");

        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &mut env);
        assert_str!(do_lisp_env("(hoge 10 ga)", &mut env), "E1008");

        do_lisp_env("(define hoge (lambda (a b) (+ ga b)))", &mut env);
        assert_str!(do_lisp_env("(hoge 10 20)", &mut env), "E1008");
    }
    #[test]
    fn if_f() {
        assert_str!(do_lisp("(if (<= 1 6))"), "E1007");
        assert_str!(do_lisp("(if (<= 1 6) a #\\b)"), "E1008");
        assert_str!(do_lisp("(if (<= 9 6) #\\a b)"), "E1008");
        assert_str!(do_lisp("(if 9 #\\a b)"), "E1001");
    }
    #[test]
    fn modulo() {
        assert_str!(do_lisp("(modulo 10)"), "E1007");
        assert_str!(do_lisp("(modulo 10 0)"), "E1013");
        assert_str!(do_lisp("(modulo 13 5.5)"), "E1002");
        assert_str!(do_lisp("(modulo 10 a)"), "E1008");
    }
    #[test]
    fn expt() {
        assert_str!(do_lisp("(expt 10)"), "E1007");
        assert_str!(do_lisp("(expt a 2)"), "E1008");
        assert_str!(do_lisp("(expt 10 #f)"), "E1002");
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
    fn set_f() {
        let mut env = lisp::SimpleEnv::new();
        assert_str!(do_lisp_env("(set! c)", &mut env), "E1007");
        assert_str!(do_lisp_env("(set! 10 10)", &mut env), "E1004");
        assert_str!(do_lisp_env("(set! c 10)", &mut env), "E1008");
    }
    #[test]
    fn list() {
        assert_str!(do_lisp("(list c 10)"), "E1008");
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
    }
    #[test]
    fn append() {
        assert_str!(do_lisp("(append)"), "E1007");
        assert_str!(do_lisp("(append (list 1))"), "E1007");
        assert_str!(do_lisp("(append (list 1) 105)"), "E1005");
    }
    #[test]
    fn last() {
        assert_str!(do_lisp("(last)"), "E1007");
        assert_str!(do_lisp("(last (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(last (list))"), "E1011");
        assert_str!(do_lisp("(last 29)"), "E1005");
    }
    #[test]
    fn reverse() {
        assert_str!(do_lisp("(reverse)"), "E1007");
        assert_str!(do_lisp("(reverse (list 1)(list 2))"), "E1007");
        assert_str!(do_lisp("(reverse 29)"), "E1005");
    }
    #[test]
    fn iota() {
        assert_str!(do_lisp("(iota)"), "E1007");
        assert_str!(do_lisp("(iota 1 2 3)"), "E1007");
        assert_str!(do_lisp("(iota 1.5 2)"), "E1002");
        assert_str!(do_lisp("(iota 1 10.5)"), "E1002");
    }
    #[test]
    fn map() {
        assert_str!(do_lisp("(map)"), "E1007");
        assert_str!(do_lisp("(map (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(map 1 2 3)"), "E1007");
        assert_str!(do_lisp("(map (iota 10) (lambda (n) n))"), "E1006");
        assert_str!(do_lisp("(map  (lambda (n) n) 10)"), "E1005");
    }
    #[test]
    fn filter() {
        assert_str!(do_lisp("(filter)"), "E1007");
        assert_str!(do_lisp("(filter (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(filter 1 2 3)"), "E1007");
        assert_str!(do_lisp("(filter (iota 10) (lambda (n) n))"), "E1006");
        assert_str!(do_lisp("(filter (lambda (n) n) 10)"), "E1005");
        assert_str!(do_lisp("(filter (lambda (n) n) (iota 4))"), "E1001");
    }
    #[test]
    fn reduce() {
        assert_str!(do_lisp("(reduce)"), "E1007");
        assert_str!(do_lisp("(reduce (lambda (n) n))"), "E1007");
        assert_str!(do_lisp("(reduce 1 2 3)"), "E1007");
        assert_str!(do_lisp("(reduce (lambda (n) n) (list))"), "E1011");
        assert_str!(do_lisp("(reduce (list) (list))"), "E1006");
        assert_str!(do_lisp("(reduce (lambda (n) n) 10)"), "E1005");
        assert_str!(do_lisp("(reduce (lambda (n) n) (iota 4))"), "E1007");
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
    fn sample_program() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env(
            "(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (gcm f mod))))",
            &mut env,
        );
        assert_str!(do_lisp_env("(gcm 36 27)", &mut env), "E1008");
    }
    #[test]
    fn load_file() {
        assert_str!(do_lisp("(load-file)"), "E1007");
        assert_str!(do_lisp("(load-file 1 2)"), "E1007");
        assert_str!(do_lisp("(load-file hoge)"), "E1008");
        assert_str!(do_lisp("(load-file #t)"), "E1015");
        assert_str!(do_lisp("(load-file \"/etc/test.scm\")"), "E1014");
    }
}
