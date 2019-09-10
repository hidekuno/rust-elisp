/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
pub mod buildin;
pub mod draw;
pub mod fractal;
pub mod ui;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate elisp;
    use buildin::{build_demo_function, build_lisp_function};
    use draw::create_image_table;

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
    fn create_png_file(kind: &str) -> String {
        let png = format!(
            "/tmp/hoge_{}_{}.png",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            kind
        );
        let mut file = File::create(&png).unwrap();
        let png_data: Vec<u8> = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xd7, 0x63, 0xd0, 0xd2, 0xd2, 0x02, 0x00, 0x01, 0x00, 0x00, 0x7f, 0x09, 0xa9, 0x5a,
            0x4d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        file.write_all(&png_data).unwrap();
        file.flush().unwrap();
        png
    }
    fn init() -> Environment {
        let env = Environment::new();
        let image_table = create_image_table();
        build_lisp_function(&env, &image_table);
        build_demo_function(&env, &image_table);
        env
    }
    #[test]
    fn test_01_normal_check() {
        let env = init();
        // draw-clear check
        assert_str!(do_lisp_env("(draw-clear)", &env), "nil");
        assert_str!(do_lisp_env("(draw-line 0.0 1.0 0.2 0.3)", &env), "nil");
        assert_str!(do_lisp_env("(draw-koch 2)", &env), "nil");
        assert_str!(do_lisp_env("(draw-tree 2)", &env), "nil");
        assert_str!(do_lisp_env("(draw-sierpinski 2)", &env), "nil");
        assert_str!(do_lisp_env("(draw-dragon 2)", &env), "nil");
        assert_str!(do_lisp_env("(draw-hilvert 2)", &env), "nil");

        assert_str!(do_lisp_env("(set-background 0.0 0.0 0.0)", &env), "nil");
        assert_str!(do_lisp_env("(set-foreground 0.0 1.0 0.0)", &env), "nil");
        assert_str!(
            do_lisp_env("(draw-string 0.04 0.50 0.15 \"日本語\")", &env),
            "nil"
        );
        assert_str!(do_lisp_env("(draw-arc 0.27 0.65 0.02 0.0)", &env), "nil");

        let png = create_png_file("1");
        assert_str!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env,
            ),
            "nil"
        );
        assert_str!(
            do_lisp_env(
                "(draw-image \"sample\" (list 0.0 0.0 1.0 1.0 1.0 1.0))",
                &env
            ),
            "nil"
        );
        std::fs::remove_file(png).unwrap();
    }
    #[test]
    fn test_02_error_check() {
        let env = init();
        // draw-clear check
        assert_str!(do_lisp_env("(draw-clear 10)", &env), "E1007");
    }
    #[test]
    fn test_03_error_check() {
        let env = init();
        // draw-line check
        assert_str!(do_lisp_env("(draw-line)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_str!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");
        assert_str!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_str!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");
    }
    #[test]
    fn test_04_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-string)", &env), "E1007");
        assert_str!(
            do_lisp_env("(draw-string 0.04 0.50 0.15 \"日本語\" 0.04)", &env),
            "E1007"
        );
        assert_str!(
            do_lisp_env("(draw-string 0.04 0.50 \"日本語\" 0.04)", &env),
            "E1003"
        );
        assert_str!(
            do_lisp_env("(draw-string 0.04 0.50  0.15 0.01)", &env),
            "E1015"
        );
    }
    #[test]
    fn test_05_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-arc)", &env), "E1007");
        assert_str!(
            do_lisp_env("(draw-arc 0.27 0.65 0.02 0.0 1.0)", &env),
            "E1007"
        );
        assert_str!(
            do_lisp_env("(draw-arc 0.04 0.50 \"日本語\" 0.04)", &env),
            "E1003"
        );
    }
    #[test]
    fn test_06_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(set-background)", &env), "E1007");
        assert_str!(do_lisp_env("(set-background 1.0)", &env), "E1007");
        assert_str!(
            do_lisp_env("(set-background 0.0 1.0 2.0 1.0)", &env),
            "E1007"
        );
        assert_str!(do_lisp_env("(set-background 0.0 1.0 2)", &env), "E1003");
        assert_str!(do_lisp_env("(set-background 0.0 1.0 a)", &env), "E1008");
    }
    #[test]
    fn test_07_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(set-foreground)", &env), "E1007");
        assert_str!(do_lisp_env("(set-foreground 1.0)", &env), "E1007");
        assert_str!(
            do_lisp_env("(set-foreground 0.0 1.0 2.0 1.0)", &env),
            "E1007"
        );
        assert_str!(do_lisp_env("(set-foreground 0.0 1.0 2)", &env), "E1003");
        assert_str!(do_lisp_env("(set-foreground 0.0 1.0 a)", &env), "E1008");
    }
    #[test]
    fn test_08_error_check() {
        let env = init();
        let png = format!(
            "/tmp/hoge_{}_2.png",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
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
        File::create(&png).unwrap();
        assert_str!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env
            ),
            "E9999"
        );
        let png = create_png_file("2");
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
            do_lisp_env(
                "(draw-image \"sample1\" (list 0.0 0.0 1.0 1.0 1.0 1.0))",
                &env
            ),
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
    }
    #[test]
    fn test_09_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-koch)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-koch 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-koch 10.5)", &env), "E1002");
    }
    #[test]
    fn test_10_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-tree)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-tree 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-tree 10.5)", &env), "E1002");
    }
    #[test]
    fn test_11_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-sierpinski)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-sierpinski 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-sierpinski 10.5)", &env), "E1002");
    }
    #[test]
    fn test_12_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-dragon)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-dragon 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-dragon 10.5)", &env), "E1002");
    }
    #[test]
    fn test_13_error_check() {
        let env = init();
        assert_str!(do_lisp_env("(draw-hilvert)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-hilvert 10 20)", &env), "E1007");
        assert_str!(do_lisp_env("(draw-hilvert 10.5)", &env), "E1002");
    }
}
