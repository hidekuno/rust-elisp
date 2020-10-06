/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::vec::Vec;

use crate::buildin::BuildInTable;
use crate::create_error;
use crate::create_error_value;
use crate::lisp::eval;
use crate::lisp::{Environment, Expression, ResultExpression};
use crate::lisp::{ErrCode, Error, Function};
use crate::referlence_list;
use crate::util::eqv;

pub fn create_function<T>(b: &mut T)
where
    T: BuildInTable + ?Sized,
{
    b.regist("define", define);
    b.regist("lambda", lambda);
    b.regist("let", let_f);
    b.regist("set!", set_f);

    b.regist("if", if_f);
    b.regist("and", and);
    b.regist("or", or);
    b.regist("not", not);
    b.regist("cond", cond);
    b.regist("case", case);
    b.regist("begin", begin);

    b.regist("apply", apply);
    b.regist("delay", delay);
    b.regist("force", force);
    b.regist("quote", |exp, _env| {
        if exp.len() != 2 {
            Err(create_error_value!(ErrCode::E1007, exp.len()))
        } else {
            Ok(exp[1].clone())
        }
    });
}
fn define(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::Symbol(v) = &exp[1] {
        if exp.len() != 3 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let se = eval(&exp[2], env)?;
        env.regist(v.to_string(), se);

        return Ok(Expression::Symbol(v.to_string()));
    }
    if let Expression::List(l) = &exp[1] {
        let l = &*(referlence_list!(l));
        if l.len() < 1 {
            return Err(create_error_value!(ErrCode::E1007, l.len()));
        }
        if let Expression::Symbol(s) = &l[0] {
            let mut param: Vec<Expression> = Vec::new();
            for n in &l[1..] {
                match n {
                    Expression::Symbol(_) => {
                        param.push(n.clone());
                    }
                    _ => return Err(create_error!(ErrCode::E1004)),
                }
            }

            let mut f = exp.to_vec();
            f[1] = Environment::create_list(param);
            let mut func = Function::new(&f, s.to_string(), env.clone());
            if env.is_tail_recursion() == true {
                func.set_tail_recurcieve();
            }
            env.regist(s.to_string(), Environment::create_func(func));

            Ok(Expression::Symbol(s.to_string()))
        } else {
            Err(create_error!(ErrCode::E1004))
        }
    } else {
        Err(create_error!(ErrCode::E1004))
    }
}
fn lambda(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::List(l) = &exp[1] {
        let l = &*(referlence_list!(l));
        for e in l {
            match e {
                Expression::Symbol(_) => {}
                _ => return Err(create_error!(ErrCode::E1004)),
            }
        }
    } else {
        return Err(create_error!(ErrCode::E1005));
    }
    Ok(Environment::create_func(Function::new(
        exp,
        String::from("lambda"),
        env.clone(),
    )))
}
fn let_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    // @@@ env.create();
    let mut param = Environment::with_parent(env);
    let mut idx = 1;
    let mut name = String::from("lambda");

    if let Expression::Symbol(s) = &exp[idx] {
        name = s.to_string();
        idx += 1;
    }
    // Parameter Setup
    let mut param_list: Vec<Expression> = Vec::new();
    let mut param_value_list: Vec<Expression> = Vec::new();
    param_value_list.push(Expression::String(String::from("dummy")));

    if let Expression::List(l) = &exp[idx] {
        let l = &*(referlence_list!(l));
        for plist in l {
            if let Expression::List(p) = plist {
                let p = &*(referlence_list!(p));
                if p.len() != 2 {
                    return Err(create_error_value!(ErrCode::E1007, p.len()));
                }
                if let Expression::Symbol(s) = &p[0] {
                    param_list.push(Expression::Symbol(s.clone()));
                    param_value_list.push(p[1].clone());
                } else {
                    return Err(create_error!(ErrCode::E1004));
                }
            } else {
                return Err(create_error!(ErrCode::E1005));
            }
        }
        idx += 1;
    } else {
        return Err(create_error!(ErrCode::E1005));
    }

    // Setup Function
    let mut vec = Vec::new();
    vec.push(Expression::String(name.to_string()));
    vec.push(Environment::create_list(param_list));
    vec.extend_from_slice(&exp[idx as usize..]);
    let mut f = Function::new(&vec[..], name.to_string(), param.clone());

    // Setup label name let
    if let Expression::Symbol(s) = &exp[1] {
        if env.is_tail_recursion() == true {
            f.set_tail_recurcieve();
            if f.get_tail_recurcieve() == false {
                param.regist(s.to_string(), Environment::create_func(f.clone()));
            }
        } else {
            param.regist(s.to_string(), Environment::create_func(f.clone()));
        }
    }
    f.execute(&param_value_list, &mut param)
}
fn set_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::Symbol(s) = &exp[1] {
        if let Some(_) = env.find(s) {
            let v = eval(&exp[2], env)?;
            env.update(s, v);
        } else {
            return Err(create_error_value!(ErrCode::E1008, s));
        }
        Ok(Expression::Symbol(s.to_string()))
    } else {
        Err(create_error!(ErrCode::E1004))
    }
}
fn if_f(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::Boolean(b) = eval(&exp[1], env)? {
        if b == true {
            eval(&exp[2], env)
        } else if 4 <= exp.len() {
            eval(&exp[3], env)
        } else {
            Ok(Expression::Nil())
        }
    } else {
        Err(create_error!(ErrCode::E1001))
    }
}
fn and(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == false {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!(ErrCode::E1001));
        }
    }
    Ok(Expression::Boolean(true))
}
fn or(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::Boolean(b) = eval(e, env)? {
            if b == true {
                return Ok(Expression::Boolean(b));
            }
        } else {
            return Err(create_error!(ErrCode::E1001));
        }
    }
    Ok(Expression::Boolean(false))
}
fn not(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    match eval(&exp[1], env)? {
        Expression::Boolean(b) => Ok(Expression::Boolean(!b)),
        _ => Err(create_error!(ErrCode::E1001)),
    }
}
fn cond(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    for e in &exp[1 as usize..] {
        if let Expression::List(l) = e {
            let l = &*(referlence_list!(l));
            let mut iter = l.iter();

            if let Some(e) = iter.next() {
                if let Expression::Symbol(s) = e {
                    if s != "else" {
                        eval(&e, env)?;
                    }
                } else {
                    let v = eval(&e, env)?;
                    if let Expression::Boolean(b) = v {
                        if b == false {
                            continue;
                        }
                        if l.len() == 1 {
                            return Ok(v);
                        }
                    }
                }
            } else {
                return Err(create_error!(ErrCode::E1012));
            }
            return begin(&l, env);
        } else {
            return Err(create_error!(ErrCode::E1005));
        }
    }
    Ok(Expression::Nil())
}
fn case(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut param: Vec<Expression> = Vec::new();
    param.push(Expression::Nil());
    param.push(eval(&exp[1], env)?);
    param.push(Expression::Nil());

    if 3 <= exp.len() {
        for e in &exp[2 as usize..] {
            if let Expression::List(l) = e {
                let l = &*(referlence_list!(l));
                if l.len() == 0 {
                    continue;
                }
                match &l[0] {
                    Expression::Symbol(s) => {
                        if s != "else" {
                            return Err(create_error!(ErrCode::E1017));
                        }
                        if 1 < l.len() {
                            return begin(&l, env);
                        } else {
                            return Ok(Expression::Integer(0));
                        }
                    }
                    Expression::List(r) => {
                        let c = &*(referlence_list!(r));
                        for e in c {
                            param[2] = eval(&e, env)?;
                            if let Expression::Boolean(b) = eqv(&param, env)? {
                                if b == true {
                                    if 1 < l.len() {
                                        return begin(&l, env);
                                    } else {
                                        return Ok(Expression::List(r.clone()));
                                    }
                                }
                            }
                        }
                    }
                    _ => return Err(create_error!(ErrCode::E1017)),
                }
            } else {
                return Err(create_error!(ErrCode::E1005));
            }
        }
    }
    Ok(Expression::Nil())
}
fn begin(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() < 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let mut ret = Expression::Nil();
    for e in &exp[1 as usize..] {
        ret = eval(e, env)?;
    }
    return Ok(ret);
}
fn apply(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 3 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    if let Expression::List(l) = eval(&exp[2], env)? {
        let l = &*(referlence_list!(l));
        let quote = Expression::BuildInFunction(
            "quote".to_string(),
            env.get_builtin_func("quote").unwrap(),
        );
        let mut se: Vec<Expression> = Vec::new();
        se.push(exp[1].clone());

        for e in l {
            match e {
                Expression::List(l) => {
                    let mut ql: Vec<Expression> = Vec::new();
                    ql.push(quote.clone());
                    ql.push(Expression::List(l.clone()));
                    se.push(Environment::create_list(ql));
                }
                _ => se.push(e.clone()),
            }
        }
        eval(&Environment::create_list(se), env)
    } else {
        Err(create_error_value!(ErrCode::E1005, exp.len()))
    }
}
fn delay(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    Ok(Expression::Promise(Box::new(exp[1].clone()), env.clone()))
}
fn force(exp: &[Expression], env: &Environment) -> ResultExpression {
    if exp.len() != 2 {
        return Err(create_error_value!(ErrCode::E1007, exp.len()));
    }
    let v = eval(&exp[1], env)?;
    if let Expression::Promise(p, pe) = v {
        eval(&(*p), &pe)
    } else {
        Ok(v)
    }
}
#[cfg(test)]
mod tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn define() {
        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_eq!(do_lisp_env("a", &env), "100");
        do_lisp_env("(define a 10.5)", &env);
        assert_eq!(do_lisp_env("a", &env), "10.5");
        do_lisp_env("(define a #t)", &env);
        assert_eq!(do_lisp_env("a", &env), "#t");
        do_lisp_env("(define a #\\A)", &env);
        assert_eq!(do_lisp_env("a", &env), "#\\A");

        do_lisp_env("(define (fuga a b)(* a b))", &env);
        assert_eq!(do_lisp_env("(fuga 6 8)", &env), "48");
        do_lisp_env("(define (hoge a b) a)", &env);
        assert_eq!(do_lisp_env("(hoge 6 8)", &env), "6");

        do_lisp_env("(define a 100)", &env);
        do_lisp_env("(define b a)", &env);
        assert_eq!(do_lisp_env("b", &env), "100");

        do_lisp_env("(define plus +)", &env);
        assert_eq!(do_lisp_env("(plus 10 20)", &env), "30");

        do_lisp_env("(define (p-nashi)(* 10 20))", &env);
        assert_eq!(do_lisp_env("(p-nashi)", &env), "200");

        do_lisp_env("(define (hoge a b)(define (alpha x)(+ x 10))(define (beta y)(+ y 10))(+ (alpha a)(beta b)))",&env);
        assert_eq!(do_lisp_env("(hoge 1 2)", &env), "23");
        assert_eq!(do_lisp_env("(hoge 3 4)", &env), "27");
    }
    #[test]
    fn lambda() {
        assert_eq!(do_lisp("((lambda (a b)(+ a b)) 1 2)"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &env);
        assert_eq!(do_lisp_env("(hoge 6 8)", &env), "14");
        do_lisp_env("(define hoge (lambda (a b) b))", &env);
        assert_eq!(do_lisp_env("(hoge 6 8)", &env), "8");
    }
    #[test]
    fn let_f() {
        assert_eq!(do_lisp("(let ((a 10)(b 20)) (+ a b))"), "30");
        assert_eq!(
            do_lisp("(let loop ((i 0)(j 0)) (if (<= 10 i) (+ i j) (loop (+ i 1)(+ j 2))))"),
            "30"
        );
        assert_eq!(
            do_lisp("(let loop ((i 0)) (if (<= 10 i) i (+ 10 (loop (+ i 1)))))"),
            "110"
        );
    }
    #[test]
    fn set_f() {
        let env = lisp::Environment::new();
        do_lisp_env("(define c 0)", &env);
        do_lisp_env("(set! c 10)", &env);
        assert_eq!(do_lisp_env("c", &env), "10");
        do_lisp_env("(set! c (+ c 1))", &env);
        assert_eq!(do_lisp_env("c", &env), "11");
    }

    #[test]
    fn if_f() {
        assert_eq!(do_lisp("(if (= 10 10) #\\a)"), "#\\a");
        assert_eq!(do_lisp("(if (= 10 11) #\\a)"), "nil");
        assert_eq!(do_lisp("(if (<= 1 6) #\\a #\\b)"), "#\\a");
        assert_eq!(do_lisp("(if (<= 9 6) #\\a #\\b)"), "#\\b");
    }
    #[test]
    fn and() {
        assert_eq!(do_lisp("(and (= 1 1)(= 2 2))"), "#t");
        assert_eq!(do_lisp("(and (= 1 1)(= 2 3))"), "#f");
        assert_eq!(do_lisp("(and (= 2 1)(= 2 2))"), "#f");
        assert_eq!(do_lisp("(and (= 0 1)(= 2 3))"), "#f");
    }
    #[test]
    fn or() {
        assert_eq!(do_lisp("(or (= 1 1)(= 2 2))"), "#t");
        assert_eq!(do_lisp("(or (= 1 1)(= 2 3))"), "#t");
        assert_eq!(do_lisp("(or (= 2 1)(= 2 2))"), "#t");
        assert_eq!(do_lisp("(or (= 0 1)(= 2 3))"), "#f");
    }
    #[test]
    fn not() {
        assert_eq!(do_lisp("(not (= 1 1))"), "#f");
        assert_eq!(do_lisp("(not (= 2 1))"), "#t");
    }
    #[test]
    fn cond() {
        assert_eq!(do_lisp("(cond ((= 10 10)))"), "#t");
        assert_eq!(do_lisp("(cond ((= 100 10)))"), "nil");
        assert_eq!(do_lisp("(cond (else 10))"), "10");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 10)", &env);
        assert_eq!(do_lisp_env("(cond (a 20))", &env), "20");
        assert_eq!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 20)", &env);
        assert_eq!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"B\""
        );
        do_lisp_env("(define a 30)", &env);
        assert_eq!(
            do_lisp_env("(cond ((= a 10) \"A\")((= a 20) \"B\")(else \"C\"))", &env),
            "\"C\""
        );
        assert_eq!(
            do_lisp_env(
                "(cond ((= a 10) \"A\")((= a 20) \"B\")(else (* a 10)))",
                &env
            ),
            "300"
        );
        do_lisp_env("(define a 100)", &env);
        assert_eq!(do_lisp_env("(cond ((= a 10) 20)(else 30 40))", &env), "40");
        assert_eq!(
            do_lisp_env("(cond ((= a 100) 20 30)(else 40 50))", &env),
            "30"
        );
    }
    #[test]
    fn case() {
        assert_eq!(do_lisp("(case 10)"), "nil");
        assert_eq!(do_lisp("(case 10 ((1 2) \"A\"))"), "nil");
        assert_eq!(do_lisp("(case 10 (else 20))"), "20");
        assert_eq!(do_lisp("(case 10 (else))"), "0");
        assert_eq!(do_lisp("(case 1 ((1 2)))"), "(1 2)");

        let env = lisp::Environment::new();
        do_lisp_env("(define a 100)", &env);
        assert_eq!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 1)", &env);
        assert_eq!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"B\""
        );
        do_lisp_env("(define a 200)", &env);
        assert_eq!(
            do_lisp_env("(case a ((100 200) \"A\")(else \"B\"))", &env),
            "\"A\""
        );
        do_lisp_env("(define a 400)", &env);
        assert_eq!(
            do_lisp_env(
                "(case a ((100 200) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"B\""
        );
        do_lisp_env("(define b 100)", &env);
        assert_eq!(
            do_lisp_env(
                "(case a ((200 b) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"B\""
        );
        do_lisp_env("(define a 100)", &env);
        assert_eq!(
            do_lisp_env(
                "(case a ((200 b) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"A\""
        );
        do_lisp_env("(define a 1000)", &env);
        assert_eq!(
            do_lisp_env(
                "(case a ((b 200) \"A\")((300 400) \"B\")(else \"C\"))",
                &env
            ),
            "\"C\""
        );
        do_lisp_env("(define a 100) ", &env);
        assert_eq!(
            do_lisp_env("(case a ((100 200) \"A\" \"B\") (else \"C\"))", &env),
            "\"B\""
        );
    }
    #[test]
    fn begin() {
        assert_eq!(do_lisp("(begin (list 1 2)(list 3 4)(list 5 6))"), "(5 6)");
    }
    #[test]
    fn apply() {
        assert_eq!(do_lisp("(apply + (list 1 2 3))"), "6");
        assert_eq!(do_lisp("(apply + (list (+ 1 1) 2 3))"), "7");
        assert_eq!(do_lisp("(apply - (list 5 3 2))"), "0");
        assert_eq!(do_lisp("(apply (lambda (a b) (+ a b)) (list 1 2))"), "3");
        assert_eq!(do_lisp("(apply + (iota 10))"), "45");
        assert_eq!(
            do_lisp("(apply append (list (list 1 2 3)(list 4 5 6)))"),
            "(1 2 3 4 5 6)"
        );
        assert_eq!(
            do_lisp("(apply (lambda (a) (map (lambda (n) (* n n)) a)) (list (list 1 2 3)))"),
            "(1 4 9)"
        );
        let env = lisp::Environment::new();
        do_lisp_env("(define (hoge x y)(* x y))", &env);
        assert_eq!(do_lisp_env("(apply hoge (list 3 4))", &env), "12");
    }
    #[test]
    fn delay_force() {
        assert_eq!(do_lisp("(delay (+ 1 1))"), "Promise");
        assert_eq!(do_lisp("(force (delay (+ 1 1)))"), "2");
        assert_eq!(do_lisp("(force  (+ 1 2))"), "3");

        let env = lisp::Environment::new();
        do_lisp_env("(define p (delay (+ 2 3)))", &env);
        assert_eq!(do_lisp_env("(force p)", &env), "5");
    }
    #[test]
    fn quote() {
        assert_eq!(do_lisp("(quote 1)"), "1");
        assert_eq!(do_lisp("(quote \"abc\")"), "\"abc\"");
        assert_eq!(do_lisp("(quote a)"), "a");
        assert_eq!(do_lisp("(quote (a b c))"), "(a b c)");

        assert_eq!(do_lisp("' a"), "a");
        assert_eq!(do_lisp("'abc"), "abc");
        assert_eq!(do_lisp("'\"abc\""), "\"abc\"");
        assert_eq!(do_lisp("'\"abc\" '\"def\""), "\"def\"");
        assert_eq!(do_lisp("'(a b c)"), "(a b c)");
        assert_eq!(
            do_lisp("'(a b c (d e f (g h i)))"),
            "(a b c (d e f (g h i)))"
        );
    }
}
#[cfg(test)]
mod error_tests {
    use crate::lisp;
    use crate::{do_lisp, do_lisp_env};

    #[test]
    fn define() {
        let env = lisp::Environment::new();
        assert_eq!(do_lisp_env("(define)", &env), "E1007");
        assert_eq!(do_lisp_env("(define a)", &env), "E1007");
        assert_eq!(do_lisp_env("(define a 11 12)", &env), "E1007");
        assert_eq!(do_lisp_env("(define 1 10)", &env), "E1004");
        assert_eq!(do_lisp_env("(define (hoge a 1) (+ 100 a))", &env), "E1004");
        assert_eq!(do_lisp_env("(define (hoge 1 a) (+ 100 a))", &env), "E1004");
        assert_eq!(do_lisp_env("(define (100 a b) (+ 100 a))", &env), "E1004");
        assert_eq!(do_lisp_env("(define () (+ 100 a))", &env), "E1007");

        assert_eq!(do_lisp_env("(define a ga)", &env), "E1008");
    }
    #[test]
    fn lambda() {
        let env = lisp::Environment::new();
        assert_eq!(do_lisp_env("(lambda)", &env), "E1007");
        assert_eq!(do_lisp_env("(lambda (a b))", &env), "E1007");
        assert_eq!(do_lisp_env("(lambda  a (+ a b))", &env), "E1005");
        assert_eq!(do_lisp_env("(lambda (a 1) (+ a 10))", &env), "E1004");
        assert_eq!(do_lisp_env("((list 1) 10)", &env), "E1006");

        do_lisp_env("(define hoge (lambda (a b) (+ a b)))", &env);
        assert_eq!(do_lisp_env("(hoge 10 ga)", &env), "E1008");

        do_lisp_env("(define hoge (lambda (a b) (+ ga b)))", &env);
        assert_eq!(do_lisp_env("(hoge 10 20)", &env), "E1008");
    }
    #[test]
    fn let_f() {
        assert_eq!(do_lisp("(let loop)"), "E1007");
        assert_eq!(do_lisp("(let ((i 0 10)) (+ i 10))"), "E1007");
        assert_eq!(do_lisp("(let ((100 10)) (+ i 10))"), "E1004");
        assert_eq!(do_lisp("(let ((i a)) (+ i 10))"), "E1008");
        assert_eq!(do_lisp("(let (10) (+ i 10))"), "E1005");
        assert_eq!(do_lisp("(let 100 (+ i 10))"), "E1005");
        assert_eq!(
            do_lisp("(let loop ((i 0)) (if (<= 10 i) i (loop (+ i 1)(+ i 1))))"),
            "E1007"
        );
    }
    #[test]
    fn set_f() {
        let env = lisp::Environment::new();
        assert_eq!(do_lisp_env("(set!)", &env), "E1007");
        assert_eq!(do_lisp_env("(set! c)", &env), "E1007");
        assert_eq!(do_lisp_env("(set! 10 10)", &env), "E1004");
        assert_eq!(do_lisp_env("(set! c 10)", &env), "E1008");
    }
    #[test]
    fn if_f() {
        assert_eq!(do_lisp("(if (<= 1 6))"), "E1007");
        assert_eq!(do_lisp("(if (<= 1 6) a #\\b)"), "E1008");
        assert_eq!(do_lisp("(if (<= 9 6) #\\a b)"), "E1008");
        assert_eq!(do_lisp("(if 9 #\\a b)"), "E1001");
    }
    #[test]
    fn and() {
        assert_eq!(do_lisp("(and (= 1 1))"), "E1007");
        assert_eq!(do_lisp("(and (= 1 1) 10)"), "E1001");
        assert_eq!(do_lisp("(and a (= 1 1))"), "E1008");
    }
    #[test]
    fn or() {
        assert_eq!(do_lisp("(or (= 1 1))"), "E1007");
        assert_eq!(do_lisp("(or (= 1 2) 10)"), "E1001");
        assert_eq!(do_lisp("(or a (= 1 2) 10)"), "E1008");
    }
    #[test]
    fn not() {
        assert_eq!(do_lisp("(not)"), "E1007");
        assert_eq!(do_lisp("(not 10)"), "E1001");
        assert_eq!(do_lisp("(not a)"), "E1008");
    }
    #[test]
    fn cond() {
        assert_eq!(do_lisp("(cond)"), "E1007");
        assert_eq!(do_lisp("(cond 10)"), "E1005");
        assert_eq!(do_lisp("(cond (b 10))"), "E1008");
        assert_eq!(do_lisp("(cond ((= 10 10) b))"), "E1008");
        assert_eq!(do_lisp("(cond ())"), "E1012");
    }
    #[test]
    fn case() {
        assert_eq!(do_lisp("(case)"), "E1007");
        assert_eq!(do_lisp("(case 10 (hoge 20))"), "E1017");
        assert_eq!(do_lisp("(case 10 10)"), "E1005");
        assert_eq!(do_lisp("(case 10 (20))"), "E1017");
        assert_eq!(do_lisp("(case a)"), "E1008");
        assert_eq!(do_lisp("(case 10 ((10 20) a))"), "E1008");
        assert_eq!(do_lisp("(case 10 ((20 30) 1)(else a))"), "E1008");
    }
    #[test]
    fn begin() {
        assert_eq!(do_lisp("(begin)"), "E1007");
        assert_eq!(do_lisp("(begin a)"), "E1008");
    }
    #[test]
    fn apply() {
        assert_eq!(do_lisp("(apply)"), "E1007");
        assert_eq!(do_lisp("(apply -)"), "E1007");
        assert_eq!(do_lisp("(apply + (list 1 2)(lis 3 4))"), "E1007");
        assert_eq!(do_lisp("(apply + 10)"), "E1005");
        assert_eq!(do_lisp("(apply hoge (list 1 2))"), "E1008");
    }
    #[test]
    fn delay_force() {
        assert_eq!(do_lisp("(delay)"), "E1007");
        assert_eq!(do_lisp("(delay 1 2)"), "E1007");
        assert_eq!(do_lisp("(force)"), "E1007");
        assert_eq!(do_lisp("(force 1 2)"), "E1007");
        assert_eq!(do_lisp("(force hoge)"), "E1008");
    }
    #[test]
    fn quote() {
        assert_eq!(do_lisp("(quote)"), "E1007");
        assert_eq!(do_lisp("(quote 1 2)"), "E1007");
    }
}
