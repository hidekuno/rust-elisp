/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::BufRead;
use std::rc::Rc;
use std::rc::Weak;

use crate::visitor::Visitor;

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
