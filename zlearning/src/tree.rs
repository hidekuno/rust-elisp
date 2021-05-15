/*
  Rust study program.
  This is 1st program.

  hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::StdinLock;
use std::rc::Rc;
use std::rc::Weak;

use crate::visitor::Visitor;

// Prevent Broken pipe
#[macro_export]
macro_rules! write_unwrap {
    ($o: expr, $s: expr) => {
        match write!($o, "{}", $s) {
            Ok(_) => {}
            Err(_) => return,
        }
    };
}
// Prevent Broken pipe
#[macro_export]
macro_rules! writeln_unwrap {
    ($o: expr, $s: expr) => {
        match writeln!($o, "{}", $s) {
            Ok(_) => {}
            Err(_) => return,
        }
    };
}
pub type ItemRef = Rc<RefCell<Item>>;
pub type ItemWeakRef = Weak<RefCell<Item>>;

pub struct Item {
    pub name: String,
    pub last_name: String,
    pub parent: Option<ItemRef>,
    pub children: LinkedList<ItemWeakRef>,
}
impl Item {
    fn new(name: String, last_name: &str, parent: Option<ItemRef>) -> Self {
        Item {
            name: name,
            last_name: last_name.into(),
            parent: parent,
            children: LinkedList::new(),
        }
    }
    pub fn add(&mut self, c: ItemWeakRef) {
        self.children.push_back(c);
    }
    pub fn accept<T>(&self, v: &mut T)
    where
        T: Visitor + 'static,
    {
        v.visit(self);
    }
    pub fn is_last(&self) -> bool {
        match self.parent {
            Some(ref p) => {
                self.name
                    == p.borrow()
                        .children
                        .back() //linked list
                        .unwrap()
                        .upgrade() //weak
                        .unwrap()
                        .borrow() // refcell
                        .name
            }
            None => true,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum DisplayMode {
    Space,
    SingleCharLine,
    MultiCharLine,
    BoldMultiCharLine,
}
pub struct Cache {
    cache: HashMap<String, ItemRef>,
    pub top: Option<ItemRef>,
}
impl Cache {
    fn new() -> Self {
        Cache {
            cache: HashMap::new(),
            top: None,
        }
    }
    fn insert(&mut self, key: String, value: ItemRef) {
        self.cache.insert(key, value);
    }
    fn get(&self, key: &str) -> Option<ItemRef> {
        match self.cache.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    pub fn create_tree<T>(reader: &mut T, sep: char) -> Cache
    where
        T: BufRead,
    {
        let mut cache = Cache::new();

        for line in reader.lines().filter_map(|result| result.ok()) {
            let mut fullname = String::new();
            let vec: Vec<&str> = line.split(sep).collect();
            for (i, s) in vec.iter().enumerate() {
                if i != 0 {
                    fullname.push(sep);
                }
                fullname.push_str(s);
                if let Some(_) = cache.get(&fullname) {
                    continue;
                }
                let idx = match fullname.rfind(sep) {
                    Some(v) => v,
                    None => {
                        let item = Item::new(fullname.clone(), s, None);
                        let rec = Rc::new(RefCell::new(item));
                        cache.top = Some(rec.clone());
                        cache.insert(fullname.clone(), rec);
                        continue;
                    }
                };
                let parent_name = &fullname.as_str()[0..idx];
                let parent = cache.get(parent_name).unwrap();

                let item = Item::new(fullname.clone(), s, Some(parent.clone()));
                let rec = Rc::new(RefCell::new(item));
                let weak = Rc::downgrade(&rec);
                cache.insert(fullname.clone(), rec);
                parent.borrow_mut().add(weak);
            }
        }
        cache
    }
}
pub fn create_tree(delimiter: char, filename: Option<String>) -> Result<Cache, String> {
    let cache = match filename {
        Some(s) => {
            let file = match File::open(s) {
                Ok(f) => f,
                Err(e) => return Err(e.to_string()),
            };
            let meta = match file.metadata() {
                Ok(m) => m,
                Err(e) => return Err(e.to_string()),
            };
            if true == meta.is_dir() {
                return Err(String::from("It's directory."));
            }
            let mut stream = BufReader::new(file);
            Cache::create_tree::<BufReader<File>>(&mut stream, delimiter)
        }
        None => {
            let s = stdin();
            let mut cin = s.lock();
            Cache::create_tree::<StdinLock>(&mut cin, delimiter)
        }
    };
    Ok(cache)
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
