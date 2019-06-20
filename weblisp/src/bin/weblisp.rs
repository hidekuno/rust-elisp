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

use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const MAX_TRANSACTION: usize = 1000;
const MAX_CONCURRENCY: usize = 4;
const BIND_ADDRESS: &'static str = "127.0.0.1:9000";

fn main() {
    let listenner = match TcpListener::bind(BIND_ADDRESS) {
        Ok(v) => v,
        Err(e) => panic!(format!("{:?}", e)),
    };
    let pool = ThreadPool::new(MAX_CONCURRENCY);

    match listenner.set_nonblocking(false) {
        Ok(()) => {}
        Err(e) => panic!(format!("{:?}", e)),
    }

    let env = lisp::Environment::new();
    for stream in listenner.incoming().take(MAX_TRANSACTION) {
        match stream {
            Ok(stream) => {
                let env = env.clone();
                pool.execute(|| {
                    web::handle_connection(stream, env);
                });
            }
            Err(ref e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    println!("{:?}", e);
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
