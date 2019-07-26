/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
use elisp::lisp;

extern crate env_logger;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use elisp::print_error;
fn main() -> Result<(), Box<Error>> {
    let args: Vec<String> = env::args().collect();
    env_logger::init();

    if args.len() < 2 {
        lisp::do_interactive();
    } else if args[1] == "--profile" {
        let mut env = lisp::Environment::new();
        match lisp::do_core_logic(
            &String::from("(let loop ((i 0)) (if (<= 1000000 i) i (loop (+ i 1))))"),
            &mut env,
        ) {
            Ok(r) => println!("{}", r.to_string()),
            Err(e) => print_error!(e),
        }
    } else {
        let filename = &args[1];
        let mut program: Vec<String> = Vec::new();
        let mut env = lisp::Environment::new();

        for result in BufReader::new(File::open(filename)?).lines() {
            let l = result?;
            program.push(l);
        }
        match lisp::do_core_logic(&program.join(" "), &mut env) {
            Ok(r) => println!("{}", r.to_string()),
            Err(e) => print_error!(e),
        }
    }
    Ok(())
}
