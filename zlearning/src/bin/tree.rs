/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::env;
use std::error::Error;
use std::io::stdout;

use zlearning::tree;
use zlearning::visitor;

use tree::create_tree;
use tree::parse_arg;
use tree::DisplayMode;
use visitor::ItemVisitor;
use visitor::LineItemVisitor;

fn main() -> Result<(), Box<dyn Error>> {
    let (delimiter, mode, filename) = parse_arg(env::args().collect())?;
    let cache = create_tree(delimiter, filename)?;

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
    Ok(())
}
