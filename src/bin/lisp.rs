/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
use crate::lisp::EvalResult;
use elisp::lisp;

extern crate env_logger;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    env_logger::init();

    if args.len() < 2 {
        lisp::do_interactive();
    } else {
        let filename = &args[1];
        let mut program: Vec<String> = Vec::new();

        let mut env = lisp::SimpleEnv::new();
        for result in BufReader::new(File::open(filename)?).lines() {
            let l = result?;
            program.push(l);
        }
        match lisp::do_core_logic(program.join(" "), &mut env) {
            Ok(r) => println!("{}", r.value_string()),
            Err(e) => println!("{}", e.get_code()),
        }
    }
    Ok(())
}
