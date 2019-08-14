/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
pub mod draw;
pub mod fractal;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate elisp;
    use draw::{create_image_table, scheme_gtk};
    use elisp::lisp;
    use elisp::lisp::Environment;

    use std::fs::File;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    macro_rules! assert_str {
        ($a: expr,
         $b: expr) => {
            assert!($a == $b.to_string())
        };
    }

    fn do_lisp_env(program: &str, env: &Environment) -> String {
        match lisp::do_core_logic(&program.into(), env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }
    #[test]
    fn test_error_check() {
        let png = format!(
            "/tmp/hoge_{}.png",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let env = Environment::new();
        let image_table = create_image_table();
        scheme_gtk(&env, &image_table);

        // draw-clear check
        assert_str!(do_lisp_env("(draw-clear 10)", &env), "E1007");

        // draw-line check
        assert_str!(do_lisp_env("(draw-line)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_str!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");

        // create-image-from-png check
        assert_str!(do_lisp_env("(create-image-from-png)", &env), "E1007");
        assert_str!(
            do_lisp_env("(create-image-from-png \"sample\")", &env),
            "E1007"
        );
        assert_str!(
            do_lisp_env("(create-image-from-png 10 \"/tmp/hoge.png\")", &env),
            "E1015"
        );
        assert_str!(
            do_lisp_env("(create-image-from-png \"sample\" 20)", &env),
            "E1015"
        );
        assert_str!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env
            ),
            "E9999"
        );
        let mut file = File::create(&png).unwrap();
        assert_str!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env
            ),
            "E9999"
        );
        let png_data: Vec<u8> = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xd7, 0x63, 0xd0, 0xd2, 0xd2, 0x02, 0x00, 0x01, 0x00, 0x00, 0x7f, 0x09, 0xa9, 0x5a,
            0x4d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        file.write_all(&png_data).unwrap();
        file.flush().unwrap();
        do_lisp_env(
            format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
            &env,
        );

        assert_str!(do_lisp_env("(draw-image)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-image 10)", &env), "E1007");
        assert_str!(
            do_lisp_env("(draw-image \"sample\" (list 1 2 3) 10)", &env),
            "E1007"
        );
        assert_str!(
            do_lisp_env("(draw-image 10 (list 0.0 0.0 1.0 1.0))", &env),
            "E1015"
        );
        assert_str!(
            do_lisp_env("(draw-image \"sample1\" (list 0.0 0.0 1.0 1.0))", &env),
            "E1008"
        );
        assert_str!(
            do_lisp_env("(draw-image \"sample\" (list 0.0 0.0 1.0 1.0))", &env),
            "E1007"
        );
        assert_str!(
            do_lisp_env(
                "(draw-image \"sample\" (list 0.0 0.0 1.0 1.0 1.0 10))",
                &env
            ),
            "E1003"
        );
        assert_str!(do_lisp_env("(draw-image \"sample\" 10)", &env), "E1005");

        std::fs::remove_file(png).unwrap();

        assert_str!(do_lisp_env("(draw-koch)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-koch 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-koch 10.5)", &env), "E1002");
        assert_str!(do_lisp_env("(draw-tree)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-tree 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-tree 10.5)", &env), "E1002");
        assert_str!(do_lisp_env("(draw-sierpinski)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-sierpinski 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-sierpinski 10.5)", &env), "E1002");
    }
}
