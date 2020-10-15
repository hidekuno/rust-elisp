/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate env_logger;

#[allow(improper_ctypes)]
#[cfg(not(feature = "thread"))]
extern "C" {
    fn signal(sig: u32, cb: extern "C" fn(u32)) -> fn(u32);
}

use elisp::lisp;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use elisp::print_error;

#[cfg(not(feature = "thread"))]
extern "C" fn interrupt(sig: u32) {
    println!("\nUnhandled signal {}.\nPlease Press Enter.", sig);
}

#[cfg(feature = "thread")]
fn set_interrupt(env: &lisp::Environment) {
    let env = env.clone();

    ctrlc::set_handler(move || {
        env.set_force_stop(true);
    })
    .expect("Error setting Ctrl-C handler");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    env_logger::init();

    let env = lisp::Environment::new();

    if args.len() < 2 {
        #[cfg(not(feature = "thread"))]
        unsafe {
            signal(2, interrupt);
        }

        #[cfg(feature = "thread")]
        set_interrupt(&env);

        let mut stream = BufReader::new(std::io::stdin());

        match lisp::repl(&mut stream, &env, false) {
            Err(e) => println!("{}", e),
            Ok(_) => {}
        }
    } else if args[1] == "--profile" {
        match lisp::do_core_logic(
            &String::from("(let loop ((i 0)) (if (<= 1000000 i) i (loop (+ i 1))))"),
            &env,
        ) {
            Ok(r) => println!("{}", r.to_string()),
            Err(e) => print_error!(e),
        }
    } else {
        let filename = &args[1];
        let mut program: Vec<String> = Vec::new();

        for result in BufReader::new(File::open(filename)?).lines() {
            let l = result?;
            if l.starts_with(";") {
                continue;
            }
            program.push(l);
        }
        match lisp::do_core_logic(&program.join(" "), &env) {
            Ok(r) => println!("{}", r.to_string()),
            Err(e) => print_error!(e),
        }
    }
    Ok(())
}
