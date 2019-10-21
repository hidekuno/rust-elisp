/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::env;
use std::io::{stdin, stdout, StdinLock};

use zlearning::tree;
use zlearning::walker;

use tree::create_tree;
use walker::create_line_walker;
use walker::create_walker;

fn main() {
    let s = stdin();
    let mut cin = s.lock();

    let cache = create_tree::<StdinLock>(&mut cin, '.');

    if let Some(top) = cache.top {
        let args: Vec<String> = env::args().collect();
        let o = Box::new(stdout());

        let mut c = if args.len() < 2 {
            create_walker(o)
        } else if args[1] == "-l" {
            create_line_walker(o, "   ", "|  ", "`--", "|--")
        } else if args[1] == "-m" {
            create_line_walker(o, "　　", "　┃", "　┗━", "　┣━")
        } else {
            create_walker(o)
        };
        c(top);
    }
}
