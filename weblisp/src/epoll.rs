/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://github.com/tokio-rs/mio

   hidekuno@gmail.com
*/
use crate::buildin;
use crate::config;
use crate::web;

use config::BIND_ADDRESS;
use elisp::lisp;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

const SERVER: Token = Token(0);
const MAX_ID: usize = 1000;

pub fn run_web_epoll_service() -> Result<(), Box<dyn Error>> {
    let mut poll = Poll::new()?;

    // Create storage for events.
    let mut events = Events::with_capacity(128);

    let addr = BIND_ADDRESS.parse()?;
    let mut server = TcpListener::bind(addr)?;

    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    let env = lisp::Environment::new();
    buildin::build_lisp_function(&env);

    let mut connections = HashMap::new();
    let mut id: usize = 1;
    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => match server.accept() {
                    Ok((mut stream, addr)) => {
                        info!("{}", addr);
                        poll.registry()
                            .register(&mut stream, Token(id), Interest::READABLE)?;

                        connections.insert(id, stream);
                        id += 1;
                    }
                    Err(e) => {
                        error!("accept fault: {:?}", e);
                    }
                },
                Token(conn_id) => {
                    if event.is_readable() {
                        let mut stream = connections.remove(&conn_id).unwrap();
                        poll.registry().deregister(&mut stream)?;
                        handle_connection(stream, env.clone(), conn_id);
                    }
                }
            }
        }
        if MAX_ID < id {
            id = 1;
        }
    }
}
fn handle_connection(mut stream: TcpStream, env: lisp::Environment, id: usize) {
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
    if let Err(e) = web::entry_proc(stream, env, &buffer, id) {
        error!("entry_proc {}", e);
    }
}
