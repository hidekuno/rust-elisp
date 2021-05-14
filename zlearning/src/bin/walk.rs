/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::io::stdout;

use zlearning::tree;
use zlearning::walker;

use tree::create_tree;
use tree::parse_arg;
use tree::DisplayMode;
use walker::create_line_walker;
use walker::create_walker;

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

        let mut c = match mode {
            DisplayMode::Space => create_walker(o),
            DisplayMode::SingleCharLine => create_line_walker(o, "   ", "|  ", "`--", "|--"),
            DisplayMode::MultiCharLine => create_line_walker(o, "　　 ", "│　 ", "└── ", "├── "),
            DisplayMode::BoldMultiCharLine => {
                create_line_walker(o, "　　 ", "┃　 ", "┗━━ ", "┣━━ ")
            }
        };
        c(top);
    }
}
