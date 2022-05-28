/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::io::Write;

use crate::tree::Element;
use crate::tree::Tree;
use crate::write_unwrap;
use crate::writeln_unwrap;

pub trait Visitor<T> {
    fn visit(&mut self, item: &Tree<T>);
}
pub struct TestVisitor {
    out: Box<dyn Write>,
    v: Vec<String>,
}
impl TestVisitor {
    pub fn new(out: Box<dyn Write>) -> Self {
        TestVisitor { out, v: Vec::new() }
    }
    pub fn get_items(&self) -> &Vec<String> {
        &self.v
    }
}
impl<T> Visitor<T> for TestVisitor
where
    T: Element,
{
    fn visit(&mut self, item: &Tree<T>) {
        self.v.push(item.get_name().to_string());
        writeln_unwrap!(self.out, item.get_name());

        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            e.borrow().accept(self);
        }
    }
}
pub struct SimpleVisitor<'a> {
    out: &'a mut dyn Write,
    level: i32,
}
impl<'a> SimpleVisitor<'a> {
    pub fn new(out: &'a mut dyn Write) -> Self {
        SimpleVisitor { out, level: 0 }
    }
}
impl<'a, T> Visitor<T> for SimpleVisitor<'a>
where
    T: Element,
{
    fn visit(&mut self, item: &Tree<T>) {
        for _ in 0..self.level {
            write_unwrap!(self.out, "    ");
        }
        writeln_unwrap!(self.out, item.get_name());

        for it in item.children.iter() {
            self.level += 1;

            let e = it.upgrade().unwrap();
            e.borrow().accept(self);

            self.level -= 1;
        }
    }
}
pub struct LineVisitor<'a> {
    out: &'a mut dyn Write,
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
}
impl<'a> LineVisitor<'a> {
    pub fn new(
        out: &'a mut dyn Write,
        vline_last: &'static str,
        vline_not_last: &'static str,
        hline_last: &'static str,
        hline_not_last: &'static str,
    ) -> Self {
        LineVisitor {
            out,
            vline_last,
            vline_not_last,
            hline_last,
            hline_not_last,
        }
    }
    fn make_vline<T>(&self, keisen: &mut Vec<&str>, item: &Tree<T>)
    where
        T: Element,
    {
        if let Some(ref p) = item.parent {
            if p.borrow().parent.is_some() {
                keisen.push(if p.borrow().is_last() {
                    self.vline_last
                } else {
                    self.vline_not_last
                });
            }
            self.make_vline(keisen, &p.borrow());
        }
    }
}
impl<'a, T> Visitor<T> for LineVisitor<'a>
where
    T: Element,
{
    fn visit(&mut self, item: &Tree<T>) {
        if item.parent.is_some() {
            let mut keisen = vec![if item.is_last() {
                self.hline_last
            } else {
                self.hline_not_last
            }];

            self.make_vline(&mut keisen, item);
            keisen.reverse();
            for line in keisen {
                write_unwrap!(self.out, line);
            }
        }
        writeln_unwrap!(self.out, item.get_name());
        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            e.borrow().accept::<LineVisitor>(self);
        }
    }
}
