/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::env;
use std::io::{stdin, stdout, StdinLock};

use zlearning::tree;
use zlearning::visitor;

use tree::create_tree;
use visitor::ItemVisitor;
use visitor::LineItemVisitor;

fn main() {
    let s = stdin();
    let mut cin = s.lock();

    let cache = create_tree::<StdinLock>(&mut cin, '.');

    if let Some(top) = cache.top {
        let args: Vec<String> = env::args().collect();
        let o = Box::new(stdout());

        if args.len() < 2 {
            top.borrow().accept(&mut ItemVisitor::new(o));
        } else if args[1] == "-l" {
            top.borrow()
                .accept(&mut LineItemVisitor::new(o, "   ", "|  ", "`--", "|--"));
        } else if args[1] == "-m" {
            top.borrow().accept(&mut LineItemVisitor::new(
                o,
                "　　",
                "　┃",
                "　┗━",
                "　┣━",
            ));
        } else {
            top.borrow().accept(&mut ItemVisitor::new(o));
        }
    }
}
