/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate weblisp;

use concurrency::ThreadPool;
use elisp::lisp;
use weblisp::concurrency;
use weblisp::web;

use std::env;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const MAX_TRANSACTION: usize = 1000;
const MAX_CONCURRENCY: usize = 4;
const BIND_ADDRESS: &'static str = "127.0.0.1:9000";

pub fn run_web_service(count: usize) -> Result<(), Box<std::error::Error>> {
    let listenner = TcpListener::bind(BIND_ADDRESS)?;
    listenner.set_nonblocking(false)?;

    let pool = ThreadPool::new(MAX_CONCURRENCY);
    let env = lisp::Environment::new();
    for stream in listenner.incoming().take(count) {
        match stream {
            Ok(stream) => {
                let env = env.clone();
                pool.execute(|| {
                    web::handle_connection(stream, env);
                });
            }
            Err(ref e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    println!("take fault: {:?}", e);
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    Ok(())
}
fn main() {
    let args: Vec<String> = env::args().collect();

    let t = if args.len() < 2 {
        MAX_TRANSACTION
    } else {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        } else {
            println!("bad paramemter: {}", args[1]);
            return;
        }
    };
    match run_web_service(t) {
        Ok(_) => {}
        Err(e) => println!("main fault: {:?}", e),
    }
}
