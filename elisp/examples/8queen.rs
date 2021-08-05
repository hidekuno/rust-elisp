/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
// cargo run --example 8queen
//
extern crate elisp;
extern crate surf;

use std::io;
use surf::http::StatusCode;
use async_std::task;

use elisp::lisp;
use lisp::Environment;
use lisp::repl;

const PROGRAM_URL: &str =
    "https://raw.githubusercontent.com/hidekuno/rust-elisp/master/elisp/samples/8queen.scm";
const TEST_CODE: &str = "(8queen (iota 8 1) '())";

fn load_url() -> Result<(String,StatusCode),
                        Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(
        async {
            let mut res = surf::get(PROGRAM_URL).await?;
            let lisp = res.body_string().await?;
            Ok((lisp, res.status()))
        }
    )
}
fn main() {
    let lisp = match load_url() {
        Err(e) => {
            println!("{:?}", e);
            return;
        },
        Ok(s) => {
            if s.1 != 200 {
                return;
            }
            s.0
        }
    };
    println!("{}",lisp);

    let env = Environment::new();
    let mut cursor =io::Cursor::new(lisp.into_bytes());
    if let Err(e) = repl(&mut cursor,&env, None) {
        println!("{:?}", e);
    }
    let result = match lisp::do_core_logic(&String::from(TEST_CODE), &env) {
        Ok(r) => r.to_string(),
        Err(e) =>e.get_msg(),
    };
    println!("{}", result);
}
