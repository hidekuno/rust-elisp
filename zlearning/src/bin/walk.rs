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
use zlearning::path;
use zlearning::walker;

use param::parse_arg;
use param::DisplayMode;
use path::create_tree;
use walker::create_line_walker;
use walker::create_walker;

fn do_main(args: &[String]) -> Result<(), Box<dyn Error>> {
    let config = parse_arg(args)?;

    let cache = create_tree(&config)?;

    if let Some(top) = cache.top {
        let mut o = stdout();

        let mut c = match config.mode() {
            DisplayMode::Space => create_walker(),
            DisplayMode::SingleCharLine => create_line_walker("   ", "|  ", "`--", "|--"),
            DisplayMode::MultiCharLine => create_line_walker("　　 ", "│　 ", "└── ", "├── "),
            DisplayMode::BoldMultiCharLine => create_line_walker("　　 ", "┃　 ", "┗━━ ", "┣━━ "),
        };
        c(top, &mut o);
    } else {
        return Err("No record".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    do_main(&args[1..])
}
#[test]
fn test_main() {
    use crate::zlearning::create_test_file;

    let testfile = create_test_file();

    let args = ["-f", &testfile];
    let _ = do_main(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>());

    let args = ["-f", &testfile, "-l"];
    let _ = do_main(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>());

    let args = ["-f", &testfile, "-m"];
    let _ = do_main(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>());

    let args = ["-f", &testfile, "-b"];
    let _ = do_main(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>());

    std::fs::remove_file(testfile).unwrap();
}

#[test]
fn test_main_error() {
    let args = ["-f", "/dev/null"];
    let _ = do_main(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>());
}
