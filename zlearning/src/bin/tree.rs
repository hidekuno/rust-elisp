/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::io::{stdin, stdout, StdinLock};

use tree::create_tree;
use tree::parse_arg;
use tree::DisplayMode;
use visitor::ItemVisitor;
use visitor::LineItemVisitor;
use zlearning::tree;
use zlearning::visitor;

fn main() {
    let (delimiter, mode) = parse_arg();
    let s = stdin();
    let mut cin = s.lock();

    let cache = create_tree::<StdinLock>(&mut cin, delimiter);
    if let Some(top) = cache.top {
        let o = Box::new(stdout());
        match mode {
            DisplayMode::Space => top.borrow().accept(&mut ItemVisitor::new(o)),
            DisplayMode::SingleCharLine => top
                .borrow()
                .accept(&mut LineItemVisitor::new(o, "   ", "|  ", "`--", "|--")),
            DisplayMode::MultiCharLine => {
                top.borrow()
                    .accept(&mut LineItemVisitor::new(o, "　　", "　┃", "　┗━", "　┣━"))
            }
        }
    }
}
