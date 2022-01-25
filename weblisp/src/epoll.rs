/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://github.com/tokio-rs/mio

   hidekuno@gmail.com
*/
use crate::buildin;
use crate::config;
use crate::web;

use config::Config;
use config::BIND_ADDRESS;
use elisp::lisp;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read};

const SERVER: Token = Token(0);
const MAX_ID: usize = 1000;

pub fn run_web_epoll_service(config: Config) -> Result<(), Box<dyn Error>> {
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
    let mut requests = HashMap::new();
    let mut id: usize = 1;
    for c in 0..config.transaction_max() {
        poll.poll(&mut events, None)?;
        debug!("poll.poll");

        for event in events.iter() {
            debug!("{:?}", event);
            match event.token() {
                SERVER => loop {
                    match server.accept() {
                        Ok((mut stream, addr)) => {
                            info!("{} {}", addr, id);
                            poll.registry()
                                .register(&mut stream, Token(id), Interest::READABLE)?;

                            connections.insert(id, stream);
                            id += 1;
                            if MAX_ID < id {
                                id = 1;
                            }
                        }
                        Err(e) => {
                            // If we get a `WouldBlock` error we know our
                            // listener has no more incoming connections queued,
                            // so we can return to polling and wait for some
                            // more.
                            if e.kind() == io::ErrorKind::WouldBlock {
                                break;
                            }
                            error!("accept fault: {:?}", e);
                            break;
                        }
                    }
                },
                Token(conn_id) => {
                    if event.is_readable() {
                        let mut stream = connections.remove(&conn_id).unwrap();
                        poll.registry().deregister(&mut stream)?;

                        let (buffer, n) = handle_connection(&stream);
                        poll.registry().register(
                            &mut stream,
                            Token(conn_id),
                            Interest::WRITABLE,
                        )?;
                        requests.insert(conn_id, (stream, buffer, n));
                    } else if event.is_writable() {
                        let (mut stream, buffer, n) = requests.remove(&conn_id).unwrap();
                        poll.registry().deregister(&mut stream)?;

                        if let Err(e) = web::entry_proc(stream, env.clone(), &buffer[..n], conn_id)
                        {
                            error!("entry_proc {}", e);
                        }
                    }
                }
            }
        }
        debug!("times = {}", c);
    }
    Ok(())
}
fn handle_connection(mut stream: &TcpStream) -> ([u8; 2048], usize) {
    let mut buffer = [0; 2048];
    let mut n = 0;

    n += loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                break 0;
            }
            Ok(n) => {
                debug!("recv datasize = {}", n);
                break n;
            }
            Err(e) => {
                error!("read {}", e);
            }
        }
    };
    (buffer, n)
}
