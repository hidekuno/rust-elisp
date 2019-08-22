/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;

use crate::concurrency::ThreadPool;
use crate::web;
use elisp::lisp;

use std::error::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

pub const MAX_TRANSACTION: usize = 1000;
pub const BIND_ADDRESS: &'static str = "127.0.0.1:9000";
const MAX_CONCURRENCY: usize = 4;
const READ_TIMEOUT: u64 = 60;

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
                    error!("take fault: {:?}", e);
                    break;
                }
                // listenner.set_nonblocking(true)
                thread::sleep(Duration::from_secs(1));
            }
        }
        if env.is_force_stop() {
            break;
        }
    }
    Ok(())
}
fn handle_connection(mut stream: TcpStream, env: lisp::Environment) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))
        .unwrap();

    let mut buffer = [0; 2048];

    // read() is Not Good.(because it's not detected EOF)
    // I try read_to_end() and read_exact(), But it was NG
    if let Err(e) = stream.read(&mut buffer) {
        error!("read {}", e);
        return;
    }
    if let Err(e) = web::core_proc(stream, env, &buffer) {
        error!("core proc {}", e);
    }
}
