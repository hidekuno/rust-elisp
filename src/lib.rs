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
            return lisp::value_string(&v);
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
    }
    #[test]
    fn sample_program() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env(
            "(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (gcm m mod))))",
            &mut env,
        );
        do_lisp_env("(define (lcm n m ) (/ (* n m)(gcm n m)))", &mut env);
        assert_str!(do_lisp_env("(gcm 36 27)", &mut env), "9");
        assert_str!(do_lisp_env("(lcm 36 27)", &mut env), "108");

        // No tail recursion
        do_lisp_env(
            "(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (+ 0 (gcm m mod)))))",
            &mut env,
        );
        assert_str!(do_lisp_env("(gcm 36 27)", &mut env), "9");
    }
}
mod error_tests {
    #[allow(unused_imports)]
    use super::*;

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
}
