/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
pub mod buildin;
pub mod draw;
pub mod fractal;
pub mod helper;
pub mod ui;

extern crate elisp;

#[cfg(test)]
use elisp::lisp::Environment;

#[cfg(test)]
fn do_lisp_env(program: &str, env: &Environment) -> String {
    match elisp::lisp::do_core_logic(program, env) {
        Ok(v) => v.to_string(),
        Err(e) => e.get_code(),
    }
}
#[cfg(test)]
fn init() -> Environment {
    use buildin::{build_demo_function, build_lisp_function};
    use draw::create_draw_table;

    let env = Environment::new();
    let draw_table = create_draw_table();
    build_lisp_function(&env, &draw_table);
    build_demo_function(&env, &draw_table);
    env
}
#[cfg(test)]
fn create_png_file(kind: &str) -> String {
    use std::fs::File;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xd0,
        0xd2, 0xd2, 0x02, 0x00, 0x01, 0x00, 0x00, 0x7f, 0x09, 0xa9, 0x5a, 0x4d, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    file.write_all(&png_data).unwrap();
    file.flush().unwrap();
    png
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn draw_clear() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-clear)", &env), "nil");
    }
    #[test]
    fn draw_line() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-line 0.0 1.0 0.2 0.3)", &env), "nil");
        assert_eq!(
            do_lisp_env("(draw-line (cons 0.0 1.0) (cons 0.2 0.3))", &env),
            "nil"
        );
    }
    #[test]
    fn draw_koch() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-koch 2)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-koch 12)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-koch 0)", &env), "nil");
    }
    #[test]
    fn draw_tree() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-tree 2)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-tree 22)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-tree 0)", &env), "nil");
    }
    #[test]
    fn draw_sierpinski() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-sierpinski 2)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-sierpinski 15)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-sierpinski 0)", &env), "nil");
    }
    #[test]
    fn draw_dragon() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-dragon 2)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-dragon 20)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-dragon 0)", &env), "nil");
    }
    #[test]
    fn draw_hilbert() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-hilbert 2)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-hilbert 9)", &env), "nil");
        assert_eq!(do_lisp_env("(draw-hilbert 0)", &env), "nil");
    }
    #[test]
    fn set_background() {
        let env = init();
        assert_eq!(do_lisp_env("(set-background 0.0 0.0 0.0)", &env), "nil");
    }
    #[test]
    fn set_foreground() {
        let env = init();
        assert_eq!(do_lisp_env("(set-foreground 0.0 1.0 0.0)", &env), "nil");
    }
    #[test]
    fn set_line_width() {
        let env = init();
        assert_eq!(do_lisp_env("(set-line-width 0.001)", &env), "nil");
    }
    #[test]
    fn draw_string() {
        let env = init();
        assert_eq!(
            do_lisp_env("(draw-string 0.04 0.50 0.15 \"日本語\")", &env),
            "nil"
        );
    }
    #[test]
    fn draw_arc() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-arc 0.27 0.65 0.02 0.0)", &env), "nil");
    }
    #[test]
    fn create_image_from_png() {
        let env = init();
        let png = create_png_file("1");
        assert_eq!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env,
            ),
            "nil"
        );
        std::fs::remove_file(png).unwrap();

        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "nil"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-image \"sample\" (cons 0.0 0.0) (cons 1.0 0.0) (cons 0.0 1.0))",
                &env
            ),
            "nil"
        );
        assert_eq!(do_lisp_env("(image-width \"sample\")", &env), "1");
        assert_eq!(do_lisp_env("(image-height \"sample\")", &env), "1");
    }
    #[test]
    fn load_image() {
        let env = init();
        let png = create_png_file("3");
        assert_eq!(
            do_lisp_env(
                format!("(load-image \"sample\" \"{}\")", png).as_str(),
                &env,
            ),
            "nil"
        );
        std::fs::remove_file(png).unwrap();

        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "nil"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-image \"sample\" (cons 0.0 0.0) (cons 1.0 0.0)(cons 0.0 1.0))",
                &env
            ),
            "nil"
        );
        assert_eq!(do_lisp_env("(image-width \"sample\")", &env), "1");
        assert_eq!(do_lisp_env("(image-height \"sample\")", &env), "1");
    }
    #[test]
    fn screen_width() {
        let env = init();
        assert_eq!(do_lisp_env("(screen-width)", &env), "560");
    }
    #[test]
    fn screen_height() {
        let env = init();
        assert_eq!(do_lisp_env("(screen-height)", &env), "560");
    }
    #[test]
    fn draw_eval() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-eval (iota 10))", &env), "nil");
    }
    #[test]
    fn gtk_version() {
        let env = init();

        let output = Command::new("pkg-config")
            .arg("--modversion")
            .arg("gtk+-3.0")
            .output()
            .expect("gtk-xx-version falut");
        let version = String::from_utf8(output.stdout).unwrap();
        let mut iter = version.split('.');
        if let Some(v) = iter.next() {
            assert_eq!(do_lisp_env("(gtk-major-version)", &env), v);
        }
        if let Some(v) = iter.next() {
            assert_eq!(do_lisp_env("(gtk-minor-version)", &env), v);
        }
        if let Some(v) = iter.next() {
            assert_eq!(do_lisp_env("(gtk-micro-version)", &env), v.trim_end());
        }
    }
}
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn draw_clear() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-clear 10)", &env), "E1007");
    }
    #[test]
    fn draw_line() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-line)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");
        assert_eq!(do_lisp_env("(draw-line 0.0 1.0 2.0 3)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-line a b 2.0 3)", &env), "E1008");

        assert_eq!(do_lisp_env("(draw-line (cons 1.0 0.2) 1.0)", &env), "E1005");
        assert_eq!(do_lisp_env("(draw-line (cons 0.1 0.2) a)", &env), "E1008");
        assert_eq!(do_lisp_env("(draw-line (cons 1 0.2) a)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-line (cons 0.1 2) a)", &env), "E1003");
        assert_eq!(
            do_lisp_env("(draw-line (cons 0.1 0.1)(cons 1 0.2))", &env),
            "E1003"
        );
        assert_eq!(
            do_lisp_env("(draw-line (cons 0.1 0.1)(cons 0.1 2))", &env),
            "E1003"
        );
    }
    #[test]
    fn draw_string() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-string)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-string 0.04 0.50 0.15 \"日本語\" 0.04)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env("(draw-string 0.04 0.50 \"日本語\" 0.04)", &env),
            "E1003"
        );
        assert_eq!(
            do_lisp_env("(draw-string 0.04 0.50  0.15 0.01)", &env),
            "E1015"
        );
    }
    #[test]
    fn draw_arc() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-arc)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-arc 0.27 0.65 0.02 0.0 1.0)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env("(draw-arc 0.04 0.50 \"日本語\" 0.04)", &env),
            "E1003"
        );
        assert_eq!(do_lisp_env("(draw-arc #t 0.65 0.02 0.0)", &env), "E1003");
        assert_eq!(do_lisp_env("(draw-arc 0.27 0.65 0.02 #t)", &env), "E1003");
    }
    #[test]
    fn set_background() {
        let env = init();
        assert_eq!(do_lisp_env("(set-background)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-background 1.0)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(set-background 0.0 1.0 2.0 1.0)", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(set-background 0.0 1.0 2)", &env), "E1003");
        assert_eq!(do_lisp_env("(set-background 0.0 1.0 a)", &env), "E1008");
    }
    #[test]
    fn set_foreground() {
        let env = init();
        assert_eq!(do_lisp_env("(set-foreground)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-foreground 1.0)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(set-foreground 0.0 1.0 2.0 1.0)", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(set-foreground 0.0 1.0 2)", &env), "E1003");
        assert_eq!(do_lisp_env("(set-foreground 0.0 1.0 a)", &env), "E1008");
    }
    #[test]
    fn create_image_from_png() {
        use std::fs::File;
        use std::time::{SystemTime, UNIX_EPOCH};

        let env = init();
        let png = format!(
            "/tmp/hoge_{}_2.png",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        // create-image-from-png check
        assert_eq!(do_lisp_env("(create-image-from-png)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(create-image-from-png \"sample\")", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env("(create-image-from-png 10 \"/tmp/hoge.png\")", &env),
            "E1015"
        );
        assert_eq!(
            do_lisp_env("(create-image-from-png \"sample\" 20)", &env),
            "E1015"
        );
        assert_eq!(
            do_lisp_env(
                format!("(create-image-from-png \"sample\" \"{}\")", png).as_str(),
                &env
            ),
            "E9999"
        );
        File::create(&png).unwrap();
        assert_eq!(
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

        assert_eq!(do_lisp_env("(draw-image)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-image 10)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 1 2 3 10)", &env),
            "E1007"
        );
        assert_eq!(
            do_lisp_env("(draw-image 10 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "E1015"
        );
        assert_eq!(
            do_lisp_env("(draw-image \"sample1\" 0.0 0.0 1.0 0.0 0.0 1.0)", &env),
            "E1008"
        );
        assert_eq!(
            do_lisp_env("(draw-image \"sample\" 0.0 0.0 1.0 1.0 1.0 10)", &env),
            "E1003"
        );
        assert_eq!(
            do_lisp_env(
                "(draw-image \"sample\" (cons 0.0 0.0) (cons 1.0 1.0)(cons 1.0 10)))",
                &env
            ),
            "E1003"
        );
        assert_eq!(do_lisp_env("(image-width)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(image-width \"sample\" \"sample1\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(image-width 10)", &env), "E1015");
        assert_eq!(do_lisp_env("(image-width a)", &env), "E1008");

        assert_eq!(do_lisp_env("(image-height)", &env), "E1007");
        assert_eq!(
            do_lisp_env("(image-height \"sample\" \"sample1\")", &env),
            "E1007"
        );
        assert_eq!(do_lisp_env("(image-height 10)", &env), "E1015");
        assert_eq!(do_lisp_env("(image-height a)", &env), "E1008");
        std::fs::remove_file(png).unwrap();
    }
    #[test]
    fn load_image() {
        let env = init();
        // create-image-from-png check
        assert_eq!(do_lisp_env("(load-image)", &env), "E1007");
        assert_eq!(do_lisp_env("(load-image \"sample\")", &env), "E1007");
        assert_eq!(
            do_lisp_env("(load-image 10 \"/tmp/hoge.png\")", &env),
            "E1015"
        );
        assert_eq!(do_lisp_env("(load-image \"sample\" 20)", &env), "E1015");
        let png = "/etc/shadow";
        assert_eq!(
            do_lisp_env(
                format!("(load-image \"sample\" \"{}\")", png).as_str(),
                &env
            ),
            "E9999"
        );
    }
    #[test]
    fn draw_koch() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-koch)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-koch 10 20)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-koch 10.5)", &env), "E1002");
        assert_eq!(do_lisp_env("(draw-koch 13)", &env), "E1021");
        assert_eq!(do_lisp_env("(draw-koch -1)", &env), "E1021");
    }
    #[test]
    fn draw_tree() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-tree)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-tree 10 20)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-tree 10.5)", &env), "E1002");
        assert_eq!(do_lisp_env("(draw-tree 23)", &env), "E1021");
        assert_eq!(do_lisp_env("(draw-tree -1)", &env), "E1021");
    }
    #[test]
    fn draw_sierpinski() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-sierpinski)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-sierpinski 10 20)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-sierpinski 10.5)", &env), "E1002");
        assert_eq!(do_lisp_env("(draw-sierpinski 16)", &env), "E1021");
        assert_eq!(do_lisp_env("(draw-sierpinski -1)", &env), "E1021");
    }
    #[test]
    fn draw_dragon() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-dragon)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-dragon 10 20)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-dragon 10.5)", &env), "E1002");
        assert_eq!(do_lisp_env("(draw-dragon 21)", &env), "E1021");
        assert_eq!(do_lisp_env("(draw-dragon -1)", &env), "E1021");
    }
    #[test]
    fn draw_hilbert() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-hilbert)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-hilbert 10 20)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-hilbert 10.5)", &env), "E1002");
        assert_eq!(do_lisp_env("(draw-hilbert 10)", &env), "E1021");
        assert_eq!(do_lisp_env("(draw-hilbert -1)", &env), "E1021");
    }
    #[test]
    fn set_line_width() {
        let env = init();
        assert_eq!(do_lisp_env("(set-line-width)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-line-width  0.001 0.001)", &env), "E1007");
        assert_eq!(do_lisp_env("(set-line-width 2)", &env), "E1003");
        assert_eq!(do_lisp_env("(set-line-width a)", &env), "E1008");
    }
    #[test]
    fn screen_width() {
        let env = init();
        assert_eq!(do_lisp_env("(screen-width a)", &env), "E1007");
    }
    #[test]
    fn screen_height() {
        let env = init();
        assert_eq!(do_lisp_env("(screen-height b)", &env), "E1007");
    }
    #[test]
    fn draw_eval() {
        let env = init();
        assert_eq!(do_lisp_env("(draw-eval)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-eval a b)", &env), "E1007");
        assert_eq!(do_lisp_env("(draw-eval a)", &env), "E1008");
    }
    #[test]
    fn gtk_major_version() {
        let env = init();
        assert_eq!(do_lisp_env("(gtk-major-version a)", &env), "E1007");
    }
    #[test]
    fn gtk_minor_version() {
        let env = init();
        assert_eq!(do_lisp_env("(gtk-minor-version a)", &env), "E1007");
    }
    #[test]
    fn gtk_micro_version() {
        let env = init();
        assert_eq!(do_lisp_env("(gtk-micro-version a)", &env), "E1007");
    }
}
