/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
extern crate zlearning;

use std::io::{stdin, stdout, StdinLock};

use zlearning::tree;
use zlearning::walker;

use tree::create_tree;
use tree::parse_arg;
use tree::DisplayMode;
use walker::create_line_walker;
use walker::create_walker;

fn main() {
    let (delimiter, mode) = parse_arg();

    let s = stdin();
    let mut cin = s.lock();

    let cache = create_tree::<StdinLock>(&mut cin, delimiter);

    if let Some(top) = cache.top {
        let o = Box::new(stdout());

        let mut c = match mode {
            DisplayMode::Space => create_walker(o),
            DisplayMode::SingleCharLine => create_line_walker(o, "   ", "|  ", "`--", "|--"),
            DisplayMode::MultiCharLine => create_line_walker(o, "　　", "　┃", "　┗━", "　┣━"),
        };
        c(top);
    }
}
