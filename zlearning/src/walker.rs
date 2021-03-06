/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::tree::Item;
use crate::tree::ItemRef;
use crate::write_unwrap;
use crate::writeln_unwrap;

type PrintTree = Box<dyn FnMut(ItemRef) + 'static>;
pub fn create_walker(mut out: Box<dyn Write>) -> PrintTree {
    fn walk(item: &Item, level: i32, out: &mut dyn Write) {
        for _ in 0..level {
            write_unwrap!(out, "    ");
        }
        writeln_unwrap!(out, item.last_name);

        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            walk(&e.borrow(), level + 1, out);
        }
    }
    let print_tree = move |rc| {
        // For more information about this error, try `rustc --explain E0282`.
        walk(&(rc as ItemRef).borrow(), 0, &mut out);
    };
    Box::new(print_tree)
}
struct KeisenParam {
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
}
pub fn create_line_walker(
    mut out: Box<dyn Write>,
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
) -> PrintTree {
    let param = KeisenParam {
        vline_last,
        vline_not_last,
        hline_last,
        hline_not_last,
    };
    let print_tree = move |rc| {
        fn make_vline(param: &KeisenParam, keisen: &mut Vec<&str>, item: &Item) {
            if let Some(ref p) = item.parent {
                if p.borrow().parent.is_some() {
                    keisen.push(if p.borrow().is_last() {
                        param.vline_last
                    } else {
                        param.vline_not_last
                    });
                }
                make_vline(param, keisen, &p.borrow());
            }
        }
        fn walk(item: &Item, param: &KeisenParam, out: &mut dyn Write) {
            if item.parent.is_some() {
                let mut keisen = vec![if item.is_last() {
                    param.hline_last
                } else {
                    param.hline_not_last
                }];

                make_vline(param, &mut keisen, item);
                keisen.reverse();
                for line in keisen {
                    write_unwrap!(out, line);
                }
            }
            writeln_unwrap!(out, item.last_name);
            for it in item.children.iter() {
                let e = it.upgrade().unwrap();
                walk(&e.borrow(), param, out);
            }
        }
        // For more information about this error, try `rustc --explain E0282`.
        walk(&(rc as Rc<RefCell<Item>>).borrow(), &param, &mut out);
    };
    Box::new(print_tree)
}
