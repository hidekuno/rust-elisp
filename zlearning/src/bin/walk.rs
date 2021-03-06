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
use zlearning::walker;

use param::parse_arg;
use param::DisplayMode;
use tree::create_tree;
use walker::create_line_walker;
use walker::create_walker;

fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_arg(env::args().collect())?;
    let cache = create_tree(&config)?;

    if let Some(top) = cache.top {
        let o = Box::new(stdout());

        let mut c = match config.mode() {
            DisplayMode::Space => create_walker(o),
            DisplayMode::SingleCharLine => create_line_walker(o, "   ", "|  ", "`--", "|--"),
            DisplayMode::MultiCharLine => create_line_walker(o, "　　 ", "│　 ", "└── ", "├── "),
            DisplayMode::BoldMultiCharLine => {
                create_line_walker(o, "　　 ", "┃　 ", "┗━━ ", "┣━━ ")
            }
        };
        c(top);
    }
    Ok(())
}
