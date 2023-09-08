/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

const MAX_LEVEL: i32 = 20;

const SINGLE_CHAR_PARAM: &str = "-l";
const MULTI_CHAR_PARAM: &str = "-m";
const BOLD_MULTI_CHAR_LINE_PARAM: &str = "-b";
const DELIMITER_PARAM: &str = "-d";
const FILENAME_PARAM: &str = "-f";
const LEVEL_PARAM: &str = "-n";

#[derive(Debug, Clone)]
struct InvalidOptionError;

impl Display for InvalidOptionError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "invalid option")
    }
}
impl Error for InvalidOptionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        arg == SINGLE_CHAR_PARAM
            || arg == MULTI_CHAR_PARAM
            || arg == BOLD_MULTI_CHAR_LINE_PARAM
            || arg == DELIMITER_PARAM
            || arg == FILENAME_PARAM
            || arg == LEVEL_PARAM
    }
    fn parse_level(arg: &str) -> Result<i32, Box<dyn Error>> {
        match arg.parse::<i32>() {
            Ok(n) => match n {
                0..=MAX_LEVEL => Ok(n),
                _ => Err(Box::new(InvalidOptionError {})),
            },
            Err(_) => Err(Box::new(InvalidOptionError {})),
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
pub fn parse_arg(args: &[String]) -> Result<Config, Box<dyn Error>> {
    let mut parse = ParamParse::Off;
    let mut config = Config::new();

    for arg in args {
        match parse {
            ParamParse::Off => {
                if arg == SINGLE_CHAR_PARAM {
                    config.mode = DisplayMode::SingleCharLine;
                } else if arg == MULTI_CHAR_PARAM {
                    config.mode = DisplayMode::MultiCharLine;
                } else if arg == BOLD_MULTI_CHAR_LINE_PARAM {
                    config.mode = DisplayMode::BoldMultiCharLine;
                } else if arg == DELIMITER_PARAM {
                    parse = ParamParse::DelimiterOn;
                } else if arg == FILENAME_PARAM {
                    parse = ParamParse::FilenameOn;
                } else if arg == LEVEL_PARAM {
                    parse = ParamParse::LevelOn;
                } else {
                    return Err(Box::new(InvalidOptionError {}));
                }
            }
            ParamParse::DelimiterOn => {
                if ParamParse::check_option(arg) || arg.len() != 1 {
                    return Err(Box::new(InvalidOptionError {}));
                }
                config.delimiter = arg.chars().next().unwrap();
                parse = ParamParse::Off;
            }
            ParamParse::FilenameOn => {
                if ParamParse::check_option(arg) {
                    return Err(Box::new(InvalidOptionError {}));
                }
                config.filename = Some(arg.to_string());
                parse = ParamParse::Off;
            }
            ParamParse::LevelOn => {
                if ParamParse::check_option(arg) {
                    return Err(Box::new(InvalidOptionError {}));
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
    let vec: Vec<String> = Vec::new();
    let config = parse_arg(&vec).unwrap();

    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_02() {
    let args = ["-l"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::SingleCharLine);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_03() {
    let args = ["-m"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::MultiCharLine);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_04() {
    let args = ["-b"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::BoldMultiCharLine);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_05() {
    let args = ["-d", "/"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '/');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_06() {
    let args = ["-f", "/etc/passwd"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, Some(String::from("/etc/passwd")));
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_07() {
    let args = ["-n", "2"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, 2);
}
#[test]
fn test_parse_arg_08() {
    let args = ["-n", "0"];

    let config = parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, 0);
}
#[test]
fn test_parse_arg_09() {
    let args = ["-n"];
    let mut params = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    params.push(MAX_LEVEL.to_string());

    let config = parse_arg(&params).unwrap();
    assert_eq!(config.delimiter, '.');
    assert_eq!(config.mode, DisplayMode::Space);
    assert_eq!(config.filename, None);
    assert_eq!(config.level, MAX_LEVEL);
}
#[test]
fn test_parse_arg_err_01() {
    let args = ["-f", "-d"];

    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
#[test]
fn test_parse_arg_err_02() {
    let args = ["-d", ""];
    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
#[test]
fn test_parse_arg_err_03() {
    let args = ["-d", "123"];
    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
#[test]
fn test_parse_arg_err_04() {
    let args = ["10", "123"];
    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
#[test]
fn test_parse_arg_err_05() {
    let args = ["-n", "abc"];

    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
#[test]
fn test_parse_arg_err_06() {
    let args = ["-n", "-1"];

    match parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()) {
        Ok(_) => panic!("test fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid option"),
    }
}
