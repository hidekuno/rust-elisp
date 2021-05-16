/*
  Rust study program.
  This is 1st program.

  hidekuno@gmail.com
*/
#[derive(Debug, PartialEq)]
pub enum DisplayMode {
    Space,
    SingleCharLine,
    MultiCharLine,
    BoldMultiCharLine,
}
enum ParamParse {
    DelimiterOn,
    FilenameOn,
    Off,
}
impl ParamParse {
    fn check_option(arg: &String) -> bool {
        if arg == "-l" || arg == "-m" || arg == "-b" || arg == "-d" || arg == "-f" {
            return true;
        }
        return false;
    }
}
pub fn parse_arg(args: Vec<String>) -> Result<(char, DisplayMode, Option<String>), String> {
    let mut mode = DisplayMode::Space;
    let mut delimiter = '.';
    let mut parse = ParamParse::Off;
    let mut filename = None;

    if args.len() < 1 {
        return Err(String::from("ivalid option"));
    }
    for arg in &args[1..] {
        match parse {
            ParamParse::Off => {
                if arg == "-l" {
                    mode = DisplayMode::SingleCharLine;
                } else if arg == "-m" {
                    mode = DisplayMode::MultiCharLine;
                } else if arg == "-b" {
                    mode = DisplayMode::BoldMultiCharLine;
                } else if arg == "-d" {
                    parse = ParamParse::DelimiterOn;
                } else if arg == "-f" {
                    parse = ParamParse::FilenameOn;
                } else {
                    return Err(String::from("ivalid option"));
                }
            }
            ParamParse::DelimiterOn => {
                if ParamParse::check_option(arg) || arg.len() != 1 {
                    return Err(String::from("ivalid option"));
                }
                delimiter = arg.chars().next().unwrap();
                parse = ParamParse::Off;
            }
            ParamParse::FilenameOn => {
                if ParamParse::check_option(arg) {
                    return Err(String::from("ivalid option"));
                }
                filename = Some(arg.to_string());
                parse = ParamParse::Off;
            }
        }
    }
    Ok((delimiter, mode, filename))
}
#[test]
fn test_parse_arg_01() {
    let args = vec![""];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '.');
            assert_eq!(mode, DisplayMode::Space);
            assert_eq!(filename, None);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_02() {
    let args = vec!["", "-l"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '.');
            assert_eq!(mode, DisplayMode::SingleCharLine);
            assert_eq!(filename, None);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_03() {
    let args = vec!["", "-m"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '.');
            assert_eq!(mode, DisplayMode::MultiCharLine);
            assert_eq!(filename, None);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_04() {
    let args = vec!["", "-b"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '.');
            assert_eq!(mode, DisplayMode::BoldMultiCharLine);
            assert_eq!(filename, None);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_05() {
    let args = vec!["", "-d", "/"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '/');
            assert_eq!(mode, DisplayMode::Space);
            assert_eq!(filename, None);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_06() {
    let args = vec!["", "-f", "/etc/passwd"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok((delimiter, mode, filename)) => {
            assert_eq!(delimiter, '.');
            assert_eq!(mode, DisplayMode::Space);
            assert_eq!(filename, Some(String::from("/etc/passwd")));
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_07() {
    let args = vec!["", "-f", "-d"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "ivalid option");
        }
    }
}
#[test]
fn test_parse_arg_08() {
    let args = vec!["", "-d", "f"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "ivalid option");
        }
    }
}
#[test]
fn test_parse_arg_09() {
    let args = vec!["", "-d", "123"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "ivalid option");
        }
    }
}
#[test]
fn test_parse_arg_10() {
    let args = vec!["", "10", "123"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "ivalid option");
        }
    }
}
#[test]
fn test_parse_arg_11() {
    match parse_arg(Vec::new()) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "ivalid option");
        }
    }
}
