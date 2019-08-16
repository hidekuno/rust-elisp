/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;

use crate::concurrency::ThreadPool;
use crate::web::handle_connection;
use elisp::lisp;

use std::error::Error;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

pub const MAX_TRANSACTION: usize = 1000;
pub const BIND_ADDRESS: &'static str = "127.0.0.1:9000";
const MAX_CONCURRENCY: usize = 4;

pub fn run_web_service(count: usize) -> Result<(), Box<Error>> {
    let listenner = TcpListener::bind(BIND_ADDRESS)?;
    listenner.set_nonblocking(false)?;

    let pool = ThreadPool::new(MAX_CONCURRENCY);
    let env = lisp::Environment::new();
    for stream in listenner.incoming().take(count) {
        match stream {
            Ok(stream) => {
                let env = env.clone();
                pool.execute(|| {
                    handle_connection(stream, env);
                });
            }
            Err(ref e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    eprintln!("take fault: {:?}", e);
                    break;
                }
                // listenner.set_nonblocking(true)
                thread::sleep(Duration::from_secs(1));
            }
        }
        if env.get_force_stop() {
            break;
        }
    }
    Ok(())
}
