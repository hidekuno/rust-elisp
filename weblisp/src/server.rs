/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

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
const MAX_CONCURRENCY: usize = 4;
const READ_TIMEOUT: u64 = 60;

#[cfg(not(feature = "all-interface"))]
pub const BIND_ADDRESS: &str = "127.0.0.1:9000";

#[cfg(feature = "all-interface")]
pub const BIND_ADDRESS: &str = "0.0.0.0:9000";

pub fn run_web_service(count: usize) -> Result<(), Box<dyn Error>> {
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
    }
    Ok(())
}
fn handle_connection(mut stream: TcpStream, env: lisp::Environment) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))
        .unwrap();

    // read() is Not Good.(because it's not detected EOF)
    // I try read_to_end() and read_exact(), But it was NG
    let mut buffer = [0; 2048];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                debug!("recv datasize = {}", n);
                if n > 0 {
                    break;
                }
            }
            Err(e) => {
                error!("read {}", e);
                return;
            }
        }
    }
    if let Err(e) = web::core_proc(stream, env, &buffer) {
        error!("core proc {}", e);
    }
}
