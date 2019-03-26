// cargo test --lib
#[macro_use]
extern crate lazy_static;

pub mod lisp;
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn do_lisp(program: &str) -> String {
        let mut env = lisp::SimpleEnv::new();
        match lisp::do_core_logic(program.to_string(), &mut env) {
            Ok(v) => {
                return v.value_string();
            }
            _ => {
                return String::from("");
            }
        }
    }
    #[allow(dead_code)]
    fn do_lisp_env(program: &str, env: &mut lisp::SimpleEnv) -> String {
        match lisp::do_core_logic(program.to_string(), env) {
            Ok(v) => {
                return v.value_string();
            }
            _ => {
                return String::from("");
            }
        }
    }
    #[test]
    fn plus() {
        assert!(do_lisp("(+ 1 2)") == "3".to_string());
    }
    #[test]
    fn minus() {
        assert!(do_lisp("(- 6 1)") == "5".to_string());
    }
    #[test]
    fn multi() {
        assert!(do_lisp("(* 3 6)") == "18".to_string());
    }
    #[test]
    fn div() {
        assert!(do_lisp("(/ 9 3)") == "3".to_string());
    }
    #[test]
    fn eq() {
        assert!(do_lisp("(= 5 5)") == "#t".to_string());
    }
    #[test]
    fn not_eq() {
        assert!(do_lisp("(= 5 6)") == "#f".to_string());
    }
    #[test]
    fn than() {
        assert!(do_lisp("(> 6 5)") == "#t".to_string());
    }
    #[test]
    fn not_than() {
        assert!(do_lisp("(> 6 6)") == "#f".to_string());
    }
    #[test]
    fn less() {
        assert!(do_lisp("(< 5 6)") == "#t".to_string());
    }
    #[test]
    fn not_less() {
        assert!(do_lisp("(> 6 6)") == "#f".to_string());
    }
    #[test]
    fn than_eq() {
        assert!(do_lisp("(>= 6 6)") == "#t".to_string());
        assert!(do_lisp("(>= 6 5)") == "#t".to_string());
    }
    #[test]
    fn not_than_eq() {
        assert!(do_lisp("(>= 5 6)") == "#f".to_string());
    }
    #[test]
    fn less_eq() {
        assert!(do_lisp("(<= 6 6)") == "#t".to_string());
        assert!(do_lisp("(<= 5 6)") == "#t".to_string());
    }
    #[test]
    fn not_less_eq() {
        assert!(do_lisp("(<= 6 5)") == "#f".to_string());
    }
    #[test]
    fn define() {
        let mut env = lisp::SimpleEnv::new();
        do_lisp_env("(define a 100)", &mut env);
        assert!(do_lisp_env("a", &mut env) == "100".to_string());
    }
}
