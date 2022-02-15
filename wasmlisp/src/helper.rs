/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   cargo test --lib

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

pub const HISTORY_SIZE: usize = 10;
const MAX_LINE_SIZE: usize = 60;
//--------------------------------------------------------
// LISP History table
//--------------------------------------------------------
#[derive(Clone)]
pub struct History {
    items: Rc<RefCell<LinkedList<String>>>,
    max_item: usize,
}
impl History {
    pub fn new(n: usize) -> Self {
        History {
            items: Rc::new(RefCell::new(LinkedList::new())),
            max_item: n,
        }
    }
    pub fn push(&self, exp: &str) {
        let mut h = self.items.borrow_mut();

        let s = if let Some(v) = exp.get(0..MAX_LINE_SIZE) {
            v
        } else {
            exp
        };

        h.push_front(s.to_string());

        if h.len() > self.max_item {
            h.pop_back();
        }
    }
    pub fn get_value(&self, idx: usize) -> Option<String> {
        let h = self.items.borrow();
        for (i, e) in h.iter().enumerate() {
            if i == idx {
                return Some(e.to_string());
            }
        }
        None
    }
    pub fn walk_inner<F>(&self, func: F)
    where
        F: Fn(usize, &String),
    {
        let h = self.items.borrow();
        for (i, e) in h.iter().enumerate() {
            func(i, e);
        }
    }
}
#[test]
fn test_herlper_01() {
    let history = History::new(HISTORY_SIZE);
    assert_eq!(history.max_item, HISTORY_SIZE);
}
#[test]
fn test_herlper_02() {
    let history = History::new(HISTORY_SIZE);
    history.push("test");
    let h = history.items.borrow();

    assert_eq!(h.front(), Some(&"test".to_string()));
}
#[test]
fn test_herlper_03() {
    let history = History::new(HISTORY_SIZE);

    for i in 0..10 {
        history.push(&i.to_string());
    }
    let h = history.items.borrow();
    assert_eq!(h.front(), Some(&"9".to_string()));
}
#[test]
fn test_herlper_04() {
    let history = History::new(HISTORY_SIZE);

    for i in 0..11 {
        history.push(&i.to_string());
    }
    let h = history.items.borrow();
    assert_eq!(h.front(), Some(&"10".to_string()));
}
#[test]
fn test_herlper_05() {
    let mut s = String::new();
    for _ in 0..64 {
        s.push('0');
    }

    let history = History::new(HISTORY_SIZE);
    history.push(&s);

    let h = history.items.borrow();
    if let Some(s) = h.front() {
        assert_eq!(s.len(), MAX_LINE_SIZE);
    }
}
#[test]
fn test_herlper_06() {
    let history = History::new(HISTORY_SIZE);

    for i in 0..5 {
        history.push(&i.to_string());
    }
    assert_eq!(history.get_value(2), Some("2".to_string()));
}
