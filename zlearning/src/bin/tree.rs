/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::env;
use std::error::Error;
use std::io::stdout;

use zlearning::param;
use zlearning::tree;
use zlearning::visitor;

use param::parse_arg;
use param::DisplayMode;
use tree::create_tree;
use visitor::ItemVisitor;
use visitor::LineItemVisitor;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = parse_arg(&args[1..])?;

    let cache = create_tree(&config)?;

    if let Some(top) = cache.top {
        let mut o = stdout();
        match config.mode() {
            DisplayMode::Space => top.borrow().accept(&mut ItemVisitor::new(&mut o)),
            DisplayMode::SingleCharLine => top.borrow().accept(&mut LineItemVisitor::new(
                &mut o, "   ", "|  ", "`--", "|--",
            )),
            DisplayMode::MultiCharLine => top.borrow().accept(&mut LineItemVisitor::new(
                &mut o,
                "　　 ",
                "│　 ",
                "└── ",
                "├── ",
            )),
            DisplayMode::BoldMultiCharLine => top.borrow().accept(&mut LineItemVisitor::new(
                &mut o,
                "　　 ",
                "┃　 ",
                "┗━━ ",
                "┣━━ ",
            )),
        }
    }
    Ok(())
}
