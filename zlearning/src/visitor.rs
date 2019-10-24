/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::io::Write;

use crate::tree::Item;
use crate::write_unwrap;
use crate::writeln_unwrap;

pub trait Visitor {
    fn visit(&mut self, item: &Item);
}
pub struct ItemVisitor {
    out: Box<dyn Write>,
    level: i32,
}
impl ItemVisitor {
    pub fn new(out: Box<dyn Write>) -> Self {
        ItemVisitor { out: out, level: 0 }
    }
}
impl Visitor for ItemVisitor {
    fn visit(&mut self, item: &Item) {
        for _ in 0..self.level {
            write_unwrap!(self.out, "    ");
        }
        writeln_unwrap!(self.out, item.last_name);

        for it in item.children.iter() {
            self.level += 1;
            let e = it.upgrade().unwrap();
            e.borrow().accept::<ItemVisitor>(self);
            self.level -= 1;
        }
    }
}
pub struct LineItemVisitor {
    out: Box<dyn Write>,
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
}
impl LineItemVisitor {
    pub fn new(
        out: Box<dyn Write>,
        vline_last: &'static str,
        vline_not_last: &'static str,
        hline_last: &'static str,
        hline_not_last: &'static str,
    ) -> Self {
        LineItemVisitor {
            out: out,
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
                write_unwrap!(self.out, line);
            }
        }
        writeln_unwrap!(self.out, item.last_name);
        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            e.borrow().accept::<LineItemVisitor>(self);
        }
    }
}
