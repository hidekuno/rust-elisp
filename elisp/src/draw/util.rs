/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   this is library for glis,wasmlisp.

   hidekuno@gmail.com
*/
use crate::create_error;
use crate::create_error_value;

use crate::draw::DrawArc;
use crate::draw::DrawImage;
use crate::draw::DrawLine;
use crate::draw::Fractal;
use crate::lisp::eval;
use crate::lisp::Environment;
use crate::lisp::ErrCode;
use crate::lisp::Error;
use crate::lisp::Expression;

// ----------------------------------------------------------------
// set up for draw_line
// ----------------------------------------------------------------
pub fn regist_draw_line(fname: &'static str, env: &Environment, draw_line: DrawLine) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 5 && exp.len() != 3 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        const N: usize = 4;
        let mut loc: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut loc, (1, N))?;
        draw_line(loc[0], loc[1], loc[2], loc[3])?;
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// set up for draw_image
// ----------------------------------------------------------------
pub fn regist_draw_image(fname: &'static str, env: &Environment, draw_image: DrawImage) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 8 && exp.len() != 5 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let symbol = match eval(&exp[1], env)? {
            Expression::String(s) => s,
            e => return Err(create_error_value!(ErrCode::E1015, e)),
        };
        const N: usize = 6;
        let mut ctm: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut ctm, (2, N))?;

        // Fix panic in a function that cannot unwind.
        //     Invalid cairo state: InvalidMatrix.
        //
        // validate the vector values that make up the image shape.
        if (ctm[2] * ctm[5]) == (ctm[3] * ctm[4]) {
            return Err(create_error!(ErrCode::E1021));
        }
        draw_image(ctm[2], ctm[3], ctm[4], ctm[5], ctm[0], ctm[1], &symbol)?;
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// set up for draw_arc
// ----------------------------------------------------------------
pub fn regist_draw_arc(fname: &'static str, env: &Environment, draw_arc: DrawArc) {
    env.add_builtin_ext_func(fname, move |exp, env| {
        if exp.len() != 5 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        const N: usize = 4;
        let mut prm: [f64; N] = [0.0; N];
        set_loc(exp, env, &mut prm, (1, N))?;
        draw_arc(prm[0], prm[1], prm[2], prm[3]);
        Ok(Expression::Nil())
    });
}
// ----------------------------------------------------------------
// thisi is function for draw-line, draw-image
// ----------------------------------------------------------------
fn set_loc(
    exp: &[Expression],
    env: &Environment,
    loc: &mut [f64],
    param: (usize, usize),
) -> Result<(), Error> {
    let mut iter = exp[param.0..].iter();

    if exp.len() == (param.1 + param.0) {
        for l in loc.iter_mut().take(param.1) {
            if let Some(e) = iter.next() {
                if let Expression::Float(f) = eval(e, env)? {
                    *l = f;
                } else {
                    return Err(create_error_value!(ErrCode::E1003, e));
                }
            }
        }
    } else if exp.len() == (param.1 / 2 + param.0) {
        for i in (0..param.1).step_by(2) {
            if let Some(e) = iter.next() {
                if let Expression::Pair(x, y) = eval(e, env)? {
                    if let Expression::Float(f) = eval(&x, env)? {
                        loc[i] = f;
                    } else {
                        return Err(create_error_value!(ErrCode::E1003, e));
                    }
                    if let Expression::Float(f) = eval(&y, env)? {
                        loc[i + 1] = f;
                    } else {
                        return Err(create_error_value!(ErrCode::E1003, e));
                    }
                } else {
                    return Err(create_error_value!(ErrCode::E1005, e));
                }
            }
        }
    }
    Ok(())
}
// ----------------------------------------------------------------
// create new lisp interface
// ----------------------------------------------------------------
pub fn make_lisp_function(fractal: Box<dyn Fractal>, env: &Environment) {
    env.add_builtin_ext_func(fractal.get_func_name(), move |exp, env| {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E1007, exp.len()));
        }
        let c = match eval(&exp[1], env)? {
            Expression::Integer(c) => c,
            e => return Err(create_error_value!(ErrCode::E1002, e)),
        };
        let c = c as i32;
        if 0 > c || fractal.get_max() < c {
            return Err(create_error!(ErrCode::E1021));
        }
        fractal.do_demo(c as i32)?;
        Ok(Expression::Nil())
    });
}
#[test]
fn test_draw_util() {
    use crate::do_lisp_env;

    let env = Environment::new();
    regist_draw_line("draw-line", &env, Box::new(move |_, _, _, _| Ok(())));
    regist_draw_image(
        "draw-image",
        &env,
        Box::new(move |_, _, _, _, _, _, _| Ok(())),
    );
    regist_draw_arc("draw-arc", &env, Box::new(move |_, _, _, _| ()));

    assert_eq!(do_lisp_env("(draw-line 0.1 0.1 0.1 0.1)", &env), "nil");
    assert_eq!(do_lisp_env("(draw-line 0.1 0.1 0.1)", &env), "E1007");
    assert_eq!(do_lisp_env("(draw-line 0.1 0.1 0.1 1)", &env), "E1003");

    assert_eq!(
        do_lisp_env("(draw-image \"image\" 0.0 0.0 0.1 0.0 0.0 0.1)", &env),
        "nil"
    );
    assert_eq!(do_lisp_env("(draw-image 0.1 0.1 0.1)", &env), "E1007");
    assert_eq!(
        do_lisp_env("(draw-image #\\a 0.1 0.1 0.1 0.1 0.1 0.1)", &env),
        "E1015"
    );
    assert_eq!(
        do_lisp_env("(draw-image \"image\" 0.0 0.0 0.0 0.0 0.1 0.1)", &env),
        "E1021"
    );
    assert_eq!(
        do_lisp_env("(draw-image \"image\" 0.0 0.0 0.0 0.0 1.0 0.1)", &env),
        "E1021"
    );
    assert_eq!(
        do_lisp_env("(draw-image \"image\" 0.0 0.0 1.0 1.0 0.0 0.0)", &env),
        "E1021"
    );

    assert_eq!(do_lisp_env("(draw-arc 0.1 0.1 0.1 0.1)", &env), "nil");
    assert_eq!(do_lisp_env("(draw-arc 0.1 0.1 0.1)", &env), "E1007");

    assert_eq!(
        do_lisp_env("(draw-line (cons 0.1 0.1) (cons 0.1 0.1))", &env),
        "nil"
    );
    assert_eq!(
        do_lisp_env("(draw-line (cons 1 0.1) (cons 0.1 0.1))", &env),
        "E1003"
    );
    assert_eq!(
        do_lisp_env("(draw-line (cons 0.1 1) (cons 0.1 0.1))", &env),
        "E1003"
    );
    assert_eq!(do_lisp_env("(draw-line 10 (cons 0.1 0.1))", &env), "E1005");

    struct TestFractal {
        _draw_line: DrawLine,
    }

    impl TestFractal {
        pub fn new(_draw_line: DrawLine) -> Self {
            TestFractal { _draw_line }
        }
    }
    impl Fractal for TestFractal {
        fn get_func_name(&self) -> &'static str {
            "test-fractal"
        }
        fn get_max(&self) -> i32 {
            10
        }
        fn do_demo(&self, _: i32) -> Result<(), Error> {
            Ok(())
        }
    }
    make_lisp_function(
        Box::new(TestFractal::new(Box::new(move |_, _, _, _| Ok(())))),
        &env,
    );
    assert_eq!(do_lisp_env("(test-fractal 4)", &env), "nil");
    assert_eq!(do_lisp_env("(test-fractal)", &env), "E1007");
    assert_eq!(do_lisp_env("(test-fractal #\\a)", &env), "E1002");
    assert_eq!(do_lisp_env("(test-fractal 11)", &env), "E1021");
}
