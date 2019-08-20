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

use chrono::Local;
use env_logger::Builder;
use std::io::Write;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

fn main() {
    let args: Vec<String> = env::args().collect();

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

    let t = if args.len() < 2 {
        server::MAX_TRANSACTION
    } else {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        } else {
            error!("bad paramemter: {}", args[1]);
            return;
        }
    };
    if let Err(e) = run_web_service(t) {
        error!("main fault: {:?}", e);
    }
}
