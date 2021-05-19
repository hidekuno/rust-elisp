/*
   Rust study program.
   This is prototype program.

   hidekuno@gmail.com
*/
pub mod param;
pub mod tree;
pub mod visitor;
pub mod walker;

#[cfg(test)]
mod tests {
    use crate::tree::Cache;
    use crate::tree::Item;
    use crate::visitor::LineItemVisitor;
    use crate::visitor::Visitor;
    use crate::walker::create_line_walker;
    use crate::walker::create_walker;

    #[test]
    fn test_tree() {
        use std::io::{self};
        let mut cursor =
            io::Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Cache::create_tree::<io::Cursor<Vec<u8>>>(&mut cursor, '.', 10);

        if let Some(top) = cache.top {
            assert_eq!(top.borrow().name, "fj");
        } else {
            panic!("test failure");
        }
    }
    #[test]
    fn test_visitor() {
        struct TestVisitor {
            v: Vec<String>,
        }
        impl Visitor for TestVisitor {
            fn visit(&mut self, item: &Item) {
                self.v.push(item.last_name.to_string());

                for it in item.children.iter() {
                    let e = it.upgrade().unwrap();
                    e.borrow().accept::<TestVisitor>(self);
                }
            }
        }
        use std::io::Cursor;
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);

        if let Some(top) = cache.top {
            assert_eq!(top.borrow().name, "fj");
            let mut test = TestVisitor { v: Vec::new() };
            top.borrow().accept(&mut test);

            let mut iterator = test.v.iter();
            assert_eq!(iterator.next(), Some(&String::from("fj")));
            assert_eq!(iterator.next(), Some(&String::from("news")));
            assert_eq!(iterator.next(), Some(&String::from("reader")));
            assert_eq!(iterator.next(), Some(&String::from("server")));
            assert_eq!(iterator.next(), None);
        } else {
            panic!("test failure");
        }
    }
    #[test]
    fn test_visitor_line() {
        use std::io::Cursor;
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '\n', 10);
        let cursor = Cursor::new(String::from("").into_bytes());

        if let Some(top) = cache.top {
            let mut v = LineItemVisitor::new(Box::new(cursor), "   ", "|  ", "`--", "|--");
            top.borrow().accept(&mut v);
        } else {
            panic!("test failure");
        }
    }
    #[test]
    fn test_walker() {
        use std::io::Cursor;
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '\n', 10);
        let cursor = Cursor::new(String::from("").into_bytes());

        if let Some(top) = cache.top {
            let mut c = create_walker(Box::new(cursor));
            c(top);
        } else {
            panic!("test failure");
        }
    }
    #[test]
    fn test_walker_line() {
        use std::io::Cursor;
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '\n', 10);
        let cursor = Cursor::new(String::from("").into_bytes());

        if let Some(top) = cache.top {
            let mut c = create_line_walker(Box::new(cursor), "   ", "|  ", "`--", "|--");
            c(top);
        } else {
            panic!("test failure");
        }
    }
}
