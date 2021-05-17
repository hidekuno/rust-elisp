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

use crate::param::Config;
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
pub fn create_tree(config: &Config) -> Result<Cache, String> {
    let cache = match config.filename() {
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
            Cache::create_tree::<BufReader<File>>(&mut stream, config.delimiter())
        }
        None => {
            let s = stdin();
            let mut cin = s.lock();
            Cache::create_tree::<StdinLock>(&mut cin, config.delimiter())
        }
    };
    Ok(cache)
}
#[test]
fn test_create_tree_01() {
    use crate::param::parse_arg;
    let args = vec!["", "-f", "/proc/version", "-d", " "];

    match create_tree(
        &parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    ) {
        Ok(cache) => {
            let top = cache.top.unwrap();
            assert_eq!(top.borrow().name, "Linux");
            assert_eq!(top.borrow().last_name, "Linux");
            assert!(top.borrow().parent.is_none());
        }
        Err(_) => {}
    }
}
#[test]
fn test_create_tree_02() {
    use crate::param::parse_arg;
    let args = vec!["", "-f", "/proc/hogehoge"];

    match create_tree(
        &parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    ) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string().starts_with("No such file or directory"), true);
            assert_eq!(&e.to_string().as_str()[..7], "No such");
        }
    }
}
#[test]
fn test_create_tree_03() {
    use crate::param::parse_arg;
    let args = vec!["", "-f", "/proc"];

    match create_tree(
        &parse_arg(args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    ) {
        Ok(_) => {}
        Err(e) => {
            assert_eq!(e.to_string(), "It's directory.");
        }
    }
}
