/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
const MAX_LEVEL: i32 = 20;

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
    LevelOn,
    Off,
}
impl ParamParse {
    fn check_option(arg: &str) -> bool {
        if arg == "-l" || arg == "-m" || arg == "-b" || arg == "-d" || arg == "-f" {
            return true;
        }
        false
    }
    fn parse_level(arg: &str) -> Result<i32, String> {
        match arg.parse::<i32>() {
            Ok(n) => match n {
                0..=MAX_LEVEL => Ok(n),
                _ => Err(String::from("ivalid option")),
            },
            Err(_) => Err(String::from("ivalid option")),
        }
    }
}
pub struct Config {
    mode: DisplayMode,
    filename: Option<String>,
    delimiter: char,
    level: i32,
}
impl Config {
    fn new() -> Self {
        Config {
            mode: DisplayMode::Space,
            filename: None,
            delimiter: '.',
            level: MAX_LEVEL,
        }
    }
    pub fn mode(&self) -> &DisplayMode {
        &self.mode
    }
    pub fn filename(&self) -> &Option<String> {
        &self.filename
    }
    pub fn delimiter(&self) -> char {
        self.delimiter
    }
    pub fn level(&self) -> i32 {
        self.level
    }
}
pub fn parse_arg(args: Vec<String>) -> Result<Config, String> {
    let mut parse = ParamParse::Off;
    let mut config = Config::new();

    if args.is_empty() {
        return Err(String::from("ivalid option"));
    }
    for arg in &args[1..] {
        match parse {
            ParamParse::Off => {
                if arg == "-l" {
                    config.mode = DisplayMode::SingleCharLine;
                } else if arg == "-m" {
                    config.mode = DisplayMode::MultiCharLine;
                } else if arg == "-b" {
                    config.mode = DisplayMode::BoldMultiCharLine;
                } else if arg == "-d" {
                    parse = ParamParse::DelimiterOn;
                } else if arg == "-f" {
                    parse = ParamParse::FilenameOn;
                } else if arg == "-n" {
                    parse = ParamParse::LevelOn;
                } else {
                    return Err(String::from("ivalid option"));
                }
            }
            ParamParse::DelimiterOn => {
                if ParamParse::check_option(arg) || arg.len() != 1 {
                    return Err(String::from("ivalid option"));
                }
                config.delimiter = arg.chars().next().unwrap();
                parse = ParamParse::Off;
            }
            ParamParse::FilenameOn => {
                if ParamParse::check_option(arg) {
                    return Err(String::from("ivalid option"));
                }
                config.filename = Some(arg.to_string());
                parse = ParamParse::Off;
            }
            ParamParse::LevelOn => {
                if ParamParse::check_option(arg) {
                    return Err(String::from("ivalid option"));
                }
                config.level = ParamParse::parse_level(arg)?;
                parse = ParamParse::Off;
            }
        }
    }
    Ok(config)
}
#[test]
fn test_parse_arg_01() {
    let args = vec![""];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_02() {
    let args = vec!["", "-l"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::SingleCharLine);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_03() {
    let args = vec!["", "-m"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::MultiCharLine);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_04() {
    let args = vec!["", "-b"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::BoldMultiCharLine);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_05() {
    let args = vec!["", "-d", "/"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '/');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_06() {
    let args = vec!["", "-f", "/etc/passwd"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, Some(String::from("/etc/passwd")));
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_07() {
    let args = vec!["", "-n", "2"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, 2);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_08() {
    let args = vec!["", "-n", "0"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, 0);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_09() {
    let args = vec!["", "-n"];
    let mut params = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    params.push(MAX_LEVEL.to_string());

    match parse_arg(params) {
        Ok(config) => {
            assert_eq!(config.delimiter, '.');
            assert_eq!(config.mode, DisplayMode::Space);
            assert_eq!(config.filename, None);
            assert_eq!(config.level, MAX_LEVEL);
        }
        Err(_) => {}
    }
}
#[test]
fn test_parse_arg_err_01() {
    let args = vec!["", "-f", "-d"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_02() {
    let args = vec!["", "-d", "f"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_03() {
    let args = vec!["", "-d", "123"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_04() {
    let args = vec!["", "10", "123"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_05() {
    match parse_arg(Vec::new()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_06() {
    let args = vec!["", "-n", "abc"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
#[test]
fn test_parse_arg_err_07() {
    let args = vec!["", "-n", "-1"];

    match parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => {}
        Err(e) => assert_eq!(e.to_string(), "ivalid option"),
    }
}
