/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.
   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   ex) RUST_LOG=info cargo run --bin weblisp
   ex) RUST_LOG=debug cargo run --bin weblisp

   hidekuno@gmail.com
*/
extern crate env_logger;
extern crate weblisp;

use weblisp::config;
use weblisp::epoll;
use weblisp::server;

use config::parse_arg;
use config::OperationMode;
use epoll::run_web_epoll_service;
use server::run_web_limit_service;
use server::run_web_service;

use chrono::Local;
use env_logger::Builder;
use std::env;
use std::error::Error;
use std::io::Write;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = parse_arg(&args[1..])?;

    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| {
            let m = if let Some(m) = record.module_path() {
                m
            } else {
                ""
            };
            writeln!(
                buf,
                "{} {:<6} {:<20} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                m,
                record.args()
            )
        })
        .init();

    match config.mode() {
        OperationMode::ThreadPool => {
            if let Err(e) = run_web_service(config) {
                error!("run_web_service fault: {:?}", e);
                return Err(e);
            }
        }
        OperationMode::Limit => {
            if let Err(e) = run_web_limit_service(config) {
                error!("main fault: {:?}", e);
                return Err(e);
            }
        }
        OperationMode::Epoll => {
            if let Err(e) = run_web_epoll_service() {
                error!("run_web_limit_service fault: {:?}", e);
                return Err(e);
            }
        }
    }
    Ok(())
}
