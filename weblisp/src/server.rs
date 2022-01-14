/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   hidekuno@gmail.com
*/
extern crate elisp;

use crate::buildin;
use crate::concurrency;
use crate::config;
use crate::web;

use concurrency::ThreadPool;
use elisp::lisp;

use std::error::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use config::Config;
use config::OperationMode;
use config::BIND_ADDRESS;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

const READ_TIMEOUT: u64 = 60;
pub fn run_web_service(config: Config) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(BIND_ADDRESS)?;

    if config.is_nonblock() {
        listener.set_nonblocking(true)?;
    }

    let pool = ThreadPool::new(config.thread_max());
    let env = lisp::Environment::new();
    buildin::build_lisp_function(&env);

    if config.mode() == OperationMode::ThreadPool {
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    info!("{}", addr);

                    let env = env.clone();
                    pool.execute(|id| {
                        handle_connection(stream, env, id);
                    });
                }
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        error!("accept fault: {:?}", e);
                        break;
                    }
                    // listener.set_nonblocking(true)
                    if config.is_nonblock() {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
    } else if config.mode() == OperationMode::Limit {
        // It's only testing.
        // ex) cargo test --lib -- --test-threads=1
        for stream in listener.incoming().take(config.transaction_max()) {
            match stream {
                Ok(stream) => {
                    let env = env.clone();
                    pool.execute(|id| {
                        handle_connection(stream, env, id);
                    });
                }
                Err(ref e) => {
                    error!("take fault: {:?}", e);
                    break;
                }
            }
        }
    } else {
        error!("not reachable");
    }
    Ok(())
}
fn handle_connection(mut stream: TcpStream, env: lisp::Environment, id: usize) {
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
    if let Err(e) = web::entry_proc(mio::net::TcpStream::from_std(stream), env, &buffer, id) {
        error!("core proc {}", e);
    }
}
