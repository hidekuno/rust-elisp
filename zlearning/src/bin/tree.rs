/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::io::stdout;

use zlearning::tree;
use zlearning::visitor;

use tree::create_tree;
use tree::parse_arg;
use tree::DisplayMode;
use visitor::ItemVisitor;
use visitor::LineItemVisitor;

fn main() {
    let (delimiter, mode, filename) = parse_arg();

    let cache = match create_tree(delimiter, filename) {
        Ok(t) => t,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };
    if let Some(top) = cache.top {
        let o = Box::new(stdout());
        match mode {
            DisplayMode::Space => top.borrow().accept(&mut ItemVisitor::new(o)),
            DisplayMode::SingleCharLine => top
                .borrow()
                .accept(&mut LineItemVisitor::new(o, "   ", "|  ", "`--", "|--")),
            DisplayMode::MultiCharLine => top.borrow().accept(&mut LineItemVisitor::new(
                o,
                "　　 ",
                "│　 ",
                "└── ",
                "├── ",
            )),
            DisplayMode::BoldMultiCharLine => top.borrow().accept(&mut LineItemVisitor::new(
                o,
                "　　 ",
                "┃　 ",
                "┗━━ ",
                "┣━━ ",
            )),
        }
    }
}
