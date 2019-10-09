/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
// cargo test --test integration_test
// cargo test

extern crate elisp;
use elisp::lisp;

macro_rules! assert_str {
    ($a: expr,
     $b: expr) => {
        assert!($a == $b.to_string())
    };
}
fn do_lisp_env(program: &str, env: &lisp::Environment) -> String {
    match lisp::do_core_logic(&program.into(), env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[test]
fn gcm() {
    let program = [
        "(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (gcm m mod))))",
        "(define (effect/gcm n m) (if (= 0 (modulo n m)) m (effect/gcm m (modulo n m))))",
        "(define (bad-gcm n m) (let ((mod (modulo n m))) (if (= 0 mod)  m (+ 0 (bad-gcm m mod)))))",
        "(define (lcm n m) (/(* n m)(gcm n m)))",
    ];
    let env = lisp::Environment::new();
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(do_lisp_env("(gcm 36 27)", &env), "9");
    assert_str!(do_lisp_env("(effect/gcm 36 15)", &env), "3");
    assert_str!(do_lisp_env("(lcm 36 27)", &env), "108");
    assert_str!(do_lisp_env("(bad-gcm 36 27)", &env), "9");
}
#[test]
fn fact() {
    let env = lisp::Environment::new();

    let program = [
        "(define (fact-iter n m)(if (= n 1)m(fact-iter (- n 1)(* n m))))",
        "(define (fact n)(if (>= 1 n) 1 (* n (fact (- n 1)))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(do_lisp_env("(fact 5)", &env), "120");
    assert_str!(do_lisp_env("(fact-iter 4 1)", &env), "24");
}
#[test]
fn hanoi() {
    let env = lisp::Environment::new();
    do_lisp_env(
        "(define hanoi (lambda (from to work n) \
         (if (>= 0 n) (list) \
         (append (hanoi from work to (- n 1)) \
         (list (list (cons from to) n)) (hanoi work to from (- n 1))))))",
        &env,
    );
    assert_str!(
        do_lisp_env("(hanoi (quote a)(quote b)(quote c) 3)", &env),
        "(((a . b) 1)((a . c) 2)((b . c) 1)((a . b) 3)((c . a) 1)((c . b) 2)((a . b) 1))"
    );
    assert_str!(
        do_lisp_env("(hanoi 'a 'b 'c 3)", &env),
        "(((a . b) 1)((a . c) 2)((b . c) 1)((a . b) 3)((c . a) 1)((c . b) 2)((a . b) 1))"
    );
}
#[test]
fn prime() {
    let env = lisp::Environment::new();
    do_lisp_env(
        "(define (prime l) \
         (if (> (car l)(sqrt (last l))) l \
         (cons (car l)(prime (filter (lambda (n) (not (= 0 (modulo n (car l))))) (cdr l))))))",
        &env,
    );

    assert_str!(
        do_lisp_env("(prime (iota 30 2))", &env),
        "(2 3 5 7 11 13 17 19 23 29 31)"
    );
}
#[test]
fn perm() {
    let env = lisp::Environment::new();
    do_lisp_env(
        "(define (perm l n)\
         (if (>= 0 n) (list (list))\
         (reduce (lambda (a b)(append a b))(list)\
         (map (lambda (x) (map (lambda (p) (cons x p)) (perm (delete x l)(- n 1)))) l))))",
        &env,
    );

    assert_str!(
        do_lisp_env("(perm (list 1 2 3) 2)", &env),
        "((1 2)(1 3)(2 1)(2 3)(3 1)(3 2))"
    );
    assert_str!(
        do_lisp_env("(perm '(a b c) 2)", &env),
        "((a b)(a c)(b a)(b c)(c a)(c b))"
    );
}
#[test]
fn comb() {
    let env = lisp::Environment::new();
    do_lisp_env(
        "(define (comb l n)(if (null? l) l \
         (if (= n 1) (map (lambda (n) (list n)) l) \
         (append (map (lambda (p) (cons (car l) p)) (comb (cdr l)(- n 1))) (comb (cdr l) n)))))",
        &env,
    );

    assert_str!(
        do_lisp_env("(comb (list 1 2 3) 2)", &env),
        "((1 2)(1 3)(2 3))"
    );
    assert_str!(do_lisp_env("(comb '(a b c) 2)", &env), "((a b)(a c)(b c))");
}
#[test]
fn quick_sort() {
    let env = lisp::Environment::new();
    let program = [
        "(define test-list (list 36 27 14 19 2 8 7 6 0 9 3))",
        "(define (qsort l pred)(if (null? l) l \
         (append (qsort (filter (lambda (n) (pred n (car l))) (cdr l)) pred) \
         (cons (car l) (qsort (filter (lambda (n) (not (pred n (car l))))(cdr l)) pred)))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(
        do_lisp_env("(qsort test-list (lambda (a b)(< a b)))", &env),
        "(0 2 3 6 7 8 9 14 19 27 36)"
    );
    assert_str!(
        do_lisp_env("(qsort test-list (lambda (a b)(> a b)))", &env),
        "(36 27 19 14 9 8 7 6 3 2 0)"
    );
}
#[test]
fn bubble_sort() {
    let env = lisp::Environment::new();
    let program = [
        "(define test-list (list 36 27 14 19 2 8 7 6 0 9 3))",
        "(define bubble-iter (lambda (x l)(if (or (null? l)(< x (car l)))\
         (cons x l)(cons (car l)(bubble-iter x (cdr l))))))",
        "(define bsort (lambda (l)(if (null? l) l (bubble-iter (car l)(bsort (cdr l))))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(
        do_lisp_env("(bsort test-list)", &env),
        "(0 2 3 6 7 8 9 14 19 27 36)"
    );
}
#[test]
fn merge_sort() {
    let env = lisp::Environment::new();

    let program = [
        "(define test-list (list 36 27 14 19 2 8 7 6 0 9 3))",
        "(define (merge a b)(if (or (null? a)(null? b)) (append a b) \
         (if (< (car a)(car b))(cons (car a)(merge (cdr a) b)) \
         (cons (car b) (merge a (cdr b))))))",
        "(define (msort l)(let ((n (length l)))(if (>= 1 n ) l \
         (if (= n 2) (if (< (car l)(cadr l)) l \
         (reverse l))(let ((mid (quotient n 2)))(merge (msort (take l mid))(msort (drop l mid))))))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(
        do_lisp_env("(merge (list 1 3 5 7 9)(list 2 4 6 8 10))", &env),
        "(1 2 3 4 5 6 7 8 9 10)"
    );
    assert_str!(
        do_lisp_env("(msort test-list)", &env),
        "(0 2 3 6 7 8 9 14 19 27 36)"
    );
}
#[test]
fn inf_list() {
    let env = lisp::Environment::new();
    let program = [
        "(define stream-car (lambda (l)(car l)))",
        "(define stream-cdr (lambda (l)(force (cdr l))))",
        "(define (make-generator generator inits)(cons (car inits)(delay (make-generator generator (generator inits)))))",
        "(define (inf-list generator inits limit)\
         (let loop ((l (make-generator generator inits))(c limit)) \
         (if (>= 0 c) (list)(cons (stream-car l)(loop (stream-cdr l)(- c 1))))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(
        do_lisp_env(
            "(inf-list (lambda (n) (list (+ 1 (car n)))) (list 0) 10)",
            &env
        ),
        "(0 1 2 3 4 5 6 7 8 9)"
    );
    assert_str!(
        do_lisp_env(
            "(inf-list (lambda (n) (list (cadr n)(+ (cadr n) (car n)))) (list 0 1) 10)",
            &env
        ),
        "(0 1 1 2 3 5 8 13 21 34)"
    );
}
#[test]
fn cps() {
    // https://practical-scheme.net/wiliki/wiliki.cgi?Scheme%3A使いたい人のための継続入門
    let env = lisp::Environment::new();
    do_lisp_env("(tail-recursion-off)", &env);

    let program = [
        "(define fact-cps (lambda (n cont)(if (= n 0)(cont 1)(fact-cps (- n 1) (lambda (a) (cont (* n a)))))))",
        "(define (fact/cps n cont)(if (= n 0)(cont 1)(fact/cps (- n 1) (lambda (a) (cont (* n a))))))",
        "(define (fact/cps-ng n cont)(if (= n 0)(cont 1)(fact/cps-ng (- n 1) (lambda (a b) (cont (* n a))))))",
    ];
    for p in &program {
        do_lisp_env(p, &env);
    }
    assert_str!(do_lisp_env("(fact-cps 4 (lambda (a) a))", &env), "24");
    assert_str!(do_lisp_env("(fact-cps 4 (lambda (a) (* 2 a)))", &env), "48");
    assert_str!(do_lisp_env("(fact/cps 5 (lambda (a) a))", &env), "120");
    assert_str!(
        do_lisp_env("(fact/cps 5 (lambda (a) (* 2 a)))", &env),
        "240"
    );
    assert_str!(
        do_lisp_env("(fact/cps-ng 3 (lambda (a) (* 2 a)))", &env),
        "E1007"
    );
    assert_str!(do_lisp_env("(fact/cps 5 (lambda (a b) a))", &env), "E1007");
    assert_str!(
        do_lisp_env("(fact/cps 5 (lambda (a) (+ ng a)))", &env),
        "E1008"
    );
}
#[test]
fn closure() {
    let env = lisp::Environment::new();
    do_lisp_env(
        "(define (counter) (let ((c 0)) (lambda () (set! c (+ 1 c)) c)))",
        &env,
    );
    do_lisp_env("(define a (counter))", &env);
    do_lisp_env("(define b (counter))", &env);
    for _i in 0..10 {
        do_lisp_env("(a)", &env);
    }
    for _i in 0..5 {
        do_lisp_env("(b)", &env);
    }
    assert_str!(do_lisp_env("(a)", &env), "11");
    assert_str!(do_lisp_env("(b)", &env), "6");

    do_lisp_env(
        "(define (scounter step) (let ((c 0)) (lambda () (set! c (+ step c)) c)))",
        &env,
    );
    do_lisp_env("(define x (scounter 10))", &env);
    do_lisp_env("(define y (scounter 100))", &env);
    for _i in 0..2 {
        do_lisp_env("(x)", &env);
        do_lisp_env("(y)", &env);
    }
    assert_str!(do_lisp_env("(x)", &env), "30");
    assert_str!(do_lisp_env("(y)", &env), "300");
}
#[test]
fn closure_nest() {
    let env = lisp::Environment::new();

    do_lisp_env("(define (testf x) (lambda () (* x 10)))", &env);
    do_lisp_env("(define (foo x) (testf (* 2 x)))", &env);
    assert_str!(do_lisp_env("((foo 2))", &env), "40");

    do_lisp_env(
        "(define (counter x) (let ((c 0)) (lambda () (set! c (+ x c)) c)))",
        &env,
    );
    do_lisp_env("(define (make-counter c) (counter c))", &env);
    do_lisp_env("(define c (make-counter 10))", &env);
    assert_str!(do_lisp_env("(c)", &env), "10");
    assert_str!(do_lisp_env("(c)", &env), "20");
}
