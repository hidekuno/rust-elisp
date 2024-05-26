/*
  Rust study program.
  This is 1st program.

  hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::StdinLock;
use std::rc::Rc;

use crate::param::Config;
use crate::tree::Cache;
use crate::tree::Element;
use crate::tree::Tree;

pub struct Path {
    name: String,
    full_name: String,
}
impl Element for Path {
    fn get_name(&self) -> &str {
        &self.full_name
    }
    fn get_display_string(&self) -> &str {
        &self.name
    }
}
impl Path {
    pub fn new(full_name: String, name: String) -> Self {
        Path { full_name, name }
    }
    pub fn create_tree<R>(reader: &mut R, sep: char, level: i32) -> Cache<Path>
    where
        R: BufRead,
    {
        let mut cache = Cache::new();

        // https://rust-lang.github.io/rust-clippy/master/index.html#/lines_filter_map_ok
        for line in reader.lines().map_while(Result::ok) {
            let mut fullname = String::new();
            let vec: Vec<&str> = line.split(sep).collect();
            for (i, s) in vec.iter().enumerate() {
                if i > level as usize {
                    break;
                }

                if i != 0 {
                    fullname.push(sep);
                }
                fullname.push_str(s);
                if cache.get(&fullname).is_some() {
                    continue;
                }
                let idx = match fullname.rfind(sep) {
                    Some(v) => v,
                    None => {
                        let item = Tree::new(Path::new(fullname.clone(), s.to_string()), None);
                        let rec = Rc::new(RefCell::new(item));
                        cache.top = Some(rec.clone());
                        cache.insert(fullname.clone(), rec);
                        continue;
                    }
                };
                let parent_name = &fullname.as_str()[0..idx];
                let parent = cache.get(parent_name).unwrap();

                let item = Tree::new(
                    Path::new(fullname.clone(), s.to_string()),
                    Some(parent.clone()),
                );
                let rec = Rc::new(RefCell::new(item));
                let weak = Rc::downgrade(&rec);
                cache.insert(fullname.clone(), rec);
                parent.borrow_mut().add(weak);
            }
        }
        cache
    }
}
pub fn create_tree(config: &Config) -> Result<Cache<Path>, String> {
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
            if meta.is_dir() {
                return Err(String::from("It's directory."));
            }
            let mut stream = BufReader::new(file);
            Path::create_tree::<BufReader<File>>(&mut stream, config.delimiter(), config.level())
        }
        None => {
            let s = stdin();
            let mut cin = s.lock();
            Path::create_tree::<StdinLock>(&mut cin, config.delimiter(), config.level())
        }
    };
    Ok(cache)
}
#[test]
fn test_create_tree_01() {
    use crate::param::parse_arg;
    let args = ["-f", "/proc/version", "-d", " "];

    let cache = create_tree(
        &parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    )
    .unwrap();

    let top = cache.top.unwrap();
    assert_eq!(top.borrow().get_name(), "Linux");
    assert!(top.borrow().parent.is_none());
}
#[test]
fn test_create_tree_02() {
    use crate::param::parse_arg;
    let args = ["-f", "/proc/hogehoge"];

    let _ = create_tree(
        &parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    )
    .map_err(|e| {
        assert!(e.starts_with("No such file or directory"));
        assert_eq!(&e.as_str()[..7], "No such");
    });
}
#[test]
fn test_create_tree_03() {
    use crate::param::parse_arg;
    let args = ["-f", "/proc"];

    let _ = create_tree(
        &parse_arg(&args.iter().map(|s| s.to_string()).collect::<Vec<String>>()).unwrap(),
    )
    .map_err(|e| assert_eq!(e, "It's directory."));
}
