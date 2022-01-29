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
    use crate::visitor::ItemVisitor;
    use crate::visitor::LineItemVisitor;
    use crate::visitor::TestVisitor;
    use crate::walker::create_line_walker;
    use crate::walker::create_test_walker;
    use crate::walker::create_walker;

    use std::io::Cursor;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::io::{self};
    use std::str::from_utf8;

    #[test]
    fn test_tree() {
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
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            assert_eq!(top.borrow().name, "fj");
            let mut test = TestVisitor::new(Box::new(cursor));
            top.borrow().accept(&mut test);

            let mut iterator = test.get_items().iter();
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
    fn test_item_visitor() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut v = ItemVisitor::new(&mut cursor);
            top.borrow().accept(&mut v);
        } else {
            panic!("test failure");
        }
        assert_eq!(
            Ok("fj\n    news\n        reader\n        server\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_item_visitor_line() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut v = LineItemVisitor::new(&mut cursor, "   ", "|  ", "`--", "|--");
            top.borrow().accept(&mut v);
        } else {
            panic!("test failure");
        }
        assert_eq!(
            Ok("fj\n`--news\n   |--reader\n   `--server\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_test_walker() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_test_walker(Box::new(cursor));
            c(top);
        } else {
            panic!("test failure");
        }
    }
    #[test]
    fn test_walker() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_walker();
            c(top, &mut cursor);
        } else {
            panic!("test failure");
        }
        cursor.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(
            Ok("fj\n    news\n        reader\n        server\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_walker_line() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Cache::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);

        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_line_walker("   ", "|  ", "`--", "|--");
            c(top, &mut cursor);
        } else {
            panic!("test failure");
        }
        cursor.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(
            Ok("fj\n`--news\n   |--reader\n   `--server\n"),
            from_utf8(cursor.get_ref())
        );
    }
}
