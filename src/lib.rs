// cargo test --lib
#[macro_use]
extern crate lazy_static;

pub mod lisp;
mod tests {
    use super::*;

    fn calc_int(program: &str) -> String {
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
    #[test]
    fn plus() {
        assert!(calc_int("(+ 1 2)") == "3".to_string());
    }
    #[test]
    fn minus() {
        assert!(calc_int("(- 6 1)") == "5".to_string());
    }
    #[test]
    fn multi() {
        assert!(calc_int("(* 3 6)") == "18".to_string());
    }
    #[test]
    fn div() {
        assert!(calc_int("(/ 9 3)") == "3".to_string());
    }
    #[test]
    fn eq() {
        assert!(calc_int("(= 5 5)") == "#t".to_string());
    }
    #[test]
    fn not_eq() {
        assert!(calc_int("(= 5 6)") == "#f".to_string());
    }
}
