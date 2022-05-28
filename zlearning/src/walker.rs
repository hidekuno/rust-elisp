/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use std::io::Write;

use crate::tree::Element;
use crate::tree::Tree;
use crate::tree::TreeRef;

use crate::write_unwrap;
use crate::writeln_unwrap;

type PrintTree<'a, T> = Box<dyn FnMut(TreeRef<T>, &'a mut dyn Write) + 'static>;

pub fn create_walker<'a, T>() -> PrintTree<'a, T>
where
    T: Element,
{
    fn walk<T>(item: &Tree<T>, level: i32, out: &mut dyn Write)
    where
        T: Element,
    {
        for _ in 0..level {
            write_unwrap!(out, "    ");
        }
        writeln_unwrap!(out, item.get_name());

        for it in item.children.iter() {
            let e = it.upgrade().unwrap();
            walk(&e.borrow(), level + 1, out);
        }
    }
    let print_tree = move |rc, out| {
        // For more information about this error, try `rustc --explain E0282`.
        walk(&(rc as TreeRef<T>).borrow(), 0, out);
    };
    Box::new(print_tree)
}

struct KeisenParam {
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
}

pub fn create_line_walker<'a, T>(
    vline_last: &'static str,
    vline_not_last: &'static str,
    hline_last: &'static str,
    hline_not_last: &'static str,
) -> PrintTree<'a, T>
where
    T: Element,
{
    let param = KeisenParam {
        vline_last,
        vline_not_last,
        hline_last,
        hline_not_last,
    };
    let print_tree = move |rc, out| {
        fn make_vline<T>(param: &KeisenParam, keisen: &mut Vec<&str>, item: &Tree<T>)
        where
            T: Element,
        {
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
        fn walk<T>(item: &Tree<T>, param: &KeisenParam, out: &mut dyn Write)
        where
            T: Element,
        {
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
            writeln_unwrap!(out, item.get_name());

            for it in item.children.iter() {
                let e = it.upgrade().unwrap();
                walk(&e.borrow(), param, out);
            }
        }
        // For more information about this error, try `rustc --explain E0282`.
        walk(&(rc as TreeRef<T>).borrow(), &param, out);
    };
    Box::new(print_tree)
}

pub fn create_test_walker<T>(
    mut out: Box<dyn Write>,
) -> Box<dyn FnMut(TreeRef<T>) -> Vec<String> + 'static>
where
    T: Element,
{
    let print_tree = move |rc| {
        fn walk<T>(item: &Tree<T>, out: &mut dyn Write, vec: &mut Vec<String>)
        where
            T: Element,
        {
            writeln_unwrap!(out, item.get_name());
            vec.push(item.get_name().to_string());

            for it in item.children.iter() {
                let e = it.upgrade().unwrap();
                walk(&e.borrow(), out, vec);
            }
        }
        // For more information about this error, try `rustc --explain E0282`.
        let mut vec = Vec::new();
        walk(&(rc as TreeRef<T>).borrow(), &mut out, &mut vec);
        vec
    };
    Box::new(print_tree)
}
