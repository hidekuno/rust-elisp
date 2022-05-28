/*
  Rust study program.
  This is 1st program.

  hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
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
pub trait Element {
    fn get_name(&self) -> &str;
    fn get_display_string(&self) -> &str;
}
pub type TreeRef<T> = Rc<RefCell<Tree<T>>>;
pub type TreeWeakRef<T> = Weak<RefCell<Tree<T>>>;

pub struct Tree<T> {
    item: T,
    pub parent: Option<TreeRef<T>>,
    pub children: LinkedList<TreeWeakRef<T>>,
}
impl<T> Tree<T>
where
    T: Element,
{
    pub fn new(item: T, parent: Option<TreeRef<T>>) -> Self {
        Tree {
            item,
            parent,
            children: LinkedList::new(),
        }
    }
    pub fn add(&mut self, c: TreeWeakRef<T>) {
        self.children.push_back(c);
    }
    pub fn accept<S>(&self, v: &mut S)
    where
        S: Visitor<T>,
    {
        v.visit(self);
    }
    pub fn is_last(&self) -> bool {
        match self.parent {
            Some(ref p) => {
                self.item.get_name()
                    == p.borrow()
                        .children
                        .back() //linked list
                        .unwrap()
                        .upgrade() //weak
                        .unwrap()
                        .borrow() // refcell
                        .item
                        .get_name()
            }
            None => true,
        }
    }
    pub fn get_name(&self) -> &str {
        self.item.get_display_string()
    }
}
pub struct Cache<T> {
    cache: HashMap<String, TreeRef<T>>,
    pub top: Option<TreeRef<T>>,
}
impl<T> Cache<T>
where
    T: Element,
{
    pub fn new() -> Self {
        Cache {
            cache: HashMap::new(),
            top: None,
        }
    }
    pub fn insert(&mut self, key: String, value: TreeRef<T>) {
        self.cache.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<TreeRef<T>> {
        self.cache.get(key).cloned()
    }
}
impl<T> Default for Cache<T>
where
    T: Element,
{
    fn default() -> Self {
        Self::new()
    }
}
