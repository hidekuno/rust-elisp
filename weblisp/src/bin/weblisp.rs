/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate weblisp;

use server::run_web_service;
use weblisp::server;

use std::env;
extern crate env_logger;

fn main() {
    let args: Vec<String> = env::args().collect();
    env_logger::init();

    let t = if args.len() < 2 {
        server::MAX_TRANSACTION
    } else {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        } else {
            eprintln!("bad paramemter: {}", args[1]);
            return;
        }
    };
    if let Err(e) = run_web_service(t) {
        eprintln!("main fault: {:?}", e);
    }
}
