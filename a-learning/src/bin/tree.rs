/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::env;
use std::io::{stdin, stdout, BufRead, Write};
use std::rc::Rc;
use std::rc::Weak;

type ItemRef = Rc<RefCell<Item>>;
type ItemWeakRef = Weak<RefCell<Item>>;

struct Item {
    name: String,
    last_name: String,
    parent: Option<ItemRef>,
    children: LinkedList<ItemWeakRef>,
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
    fn add(&mut self, c: ItemWeakRef) {
        self.children.push_back(c);
    }
    fn accept<T>(&self, v: &mut T)
    where
        T: Visitor + 'static,
    {
        v.visit(self);
    }
    fn is_last(&self) -> bool {
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
trait Visitor {
    fn visit(&mut self, item: &Item);
}
struct ItemVisitor {
    level: i32,
}
impl ItemVisitor {
    fn new() -> Self {
        ItemVisitor { level: 0 }
    }
}
impl Visitor for ItemVisitor {
    fn visit(&mut self, item: &Item) {
        let mut out = stdout();
        for _ in 0..self.level {
            write!(out, "    ").unwrap();
        }
        writeln!(out, "{}", item.last_name).unwrap();

        for it in item.children.iter() {
            self.level += 1;
            let e = it.upgrade().unwrap();
            e.borrow().accept::<ItemVisitor>(self);
            self.level -= 1;
        }
    }
}
struct LineItemVisitor {
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
}
impl LineItemVisitor {
    fn new(
        vline_last: &'static str,
        vline_not_last: &'static str,
        hline_last: &'static str,
        hline_not_last: &'static str,
    ) -> Self {
        LineItemVisitor {
            vline_last: vline_last,
            vline_not_last: vline_not_last,
            hline_last: hline_last,
            hline_not_last: hline_not_last,
        }
    }
    fn make_vline(&self, keisen: &mut Vec<&str>, item: &Item) {
        match item.parent {
            Some(ref p) => {
                if let Some(_) = p.borrow().parent {
                    keisen.push(if p.borrow().is_last() {
                        self.vline_last
                    } else {
                        self.vline_not_last
                    });
                }
                self.make_vline(keisen, &p.borrow());
            }
            None => return,
        }
    }
}
impl Visitor for LineItemVisitor {
    fn visit(&mut self, item: &Item) {
        let mut out = stdout();

        if let Some(_) = item.parent {
            let mut keisen = Vec::new();
            keisen.push(if item.is_last() {
                self.hline_last
            } else {
                self.hline_not_last
            });
            self.make_vline(&mut keisen, item);
            keisen.reverse();
            for line in keisen {
                write!(out, "{}", line).unwrap();
            }
        }
        writeln!(out, "{}", item.last_name).unwrap();
        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            e.borrow().accept::<LineItemVisitor>(self);
        }
    }
}
struct Cache {
    cache: HashMap<String, ItemRef>,
    top: Option<ItemRef>,
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
fn create_tree(reader: &mut dyn BufRead, sep: char) -> Cache {
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
fn main() {
    let s = stdin();
    let mut cin = s.lock();

    let cache = create_tree(&mut cin, '.');

    if let Some(top) = cache.top {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            top.borrow().accept(&mut ItemVisitor::new());
        } else if args[1] == "-l" {
            top.borrow()
                .accept(&mut LineItemVisitor::new("   ", "|  ", "`--", "|--"));
        } else if args[1] == "-m" {
            top.borrow().accept(&mut LineItemVisitor::new(
                "　　",
                "　┃",
                "　┗━",
                "　┣━",
            ));
        } else {
            top.borrow().accept(&mut ItemVisitor::new());
        }
    }
}
