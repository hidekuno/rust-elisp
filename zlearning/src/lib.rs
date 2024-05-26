/*
   Rust study program.
   This is prototype program.

   hidekuno@gmail.com
*/
pub mod param;
pub mod path;
pub mod tree;
pub mod visitor;
pub mod walker;

#[cfg(test)]
mod tests {
    extern "C" {
        fn close(fd: u32) -> u32;
    }
    use crate::param::Config;
    use crate::path::create_tree;
    use crate::path::Path;
    use crate::tree::Cache;
    use crate::visitor::LineVisitor;
    use crate::visitor::SimpleVisitor;
    use crate::visitor::TestVisitor;
    use crate::walker::create_line_walker;
    use crate::walker::create_test_walker;
    use crate::walker::create_walker;

    use std::io::Cursor;
    use std::io::Seek;
    use std::io::{self};
    use std::str::from_utf8;

    #[test]
    fn test_tree() {
        let mut cursor =
            io::Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Path::create_tree::<io::Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        assert_eq!(cache.top.unwrap().borrow().get_name(), "fj");
    }
    #[test]
    fn test_visitor() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());
        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let cursor = Cursor::new(Vec::new());

        let top = cache.top.unwrap();
        assert_eq!(top.borrow().get_name(), "fj");
        let mut test = TestVisitor::new(Box::new(cursor));
        top.borrow().accept(&mut test);

        let mut iter = test.get_items().iter();
        assert_eq!(iter.next(), Some(&String::from("fj")));
        assert_eq!(iter.next(), Some(&String::from("news")));
        assert_eq!(iter.next(), Some(&String::from("reader")));
        assert_eq!(iter.next(), Some(&String::from("server")));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn test_item_visitor() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let mut cursor = Cursor::new(Vec::new());

        let top = cache.top.unwrap();
        let mut v = SimpleVisitor::new(&mut cursor);
        top.borrow().accept(&mut v);

        assert_eq!(
            Ok("fj\n    news\n        reader\n        server\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_item_visitor_line() {
        let test_data = [
            "fj",
            "fj.news",
            "fj.news.b",
            "fj.news.config",
            "fj.news.group",
            "fj.news.group.archives",
            "fj.news.group.comp",
            "fj.news.group.soc",
            "fj.news.usage",
            "fj.org",
            "fj.org.ieee",
            "fj.org.jus",
        ];
        let mut cursor = Cursor::new(test_data.join("\n").into_bytes());
        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);

        let mut cursor = Cursor::new(Vec::new());

        let top = cache.top.unwrap();
        let mut v = LineVisitor::new(&mut cursor, "   ", "|  ", "`--", "|--");
        top.borrow().accept(&mut v);

        assert_eq!(
            Ok("fj\n|--news\n|  |--b\n|  |--config\n|  |--group\n|  |  |--archives\n|  |  |--comp\n|  |  `--soc\n|  `--usage\n`--org\n   |--ieee\n   `--jus\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_test_walker() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let cursor = Cursor::new(Vec::new());

        let top = cache.top.unwrap();
        let mut c = create_test_walker(Box::new(cursor));
        let vec = c(top);

        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&String::from("fj")));
        assert_eq!(iter.next(), Some(&String::from("news")));
        assert_eq!(iter.next(), Some(&String::from("reader")));
        assert_eq!(iter.next(), Some(&String::from("server")));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn test_walker() {
        let mut cursor =
            Cursor::new(String::from("fj.news\nfj.news.reader\nfj.news.server\n").into_bytes());

        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);
        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_walker();
            c(top, &mut cursor);
        }
        // cursor.seek(SeekFrom::Start(0)).unwrap();
        cursor.rewind().unwrap();
        assert_eq!(
            Ok("fj\n    news\n        reader\n        server\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_walker_line() {
        let test_data = [
            "fj",
            "fj.news",
            "fj.news.b",
            "fj.news.config",
            "fj.news.group",
            "fj.news.group.archives",
            "fj.news.group.comp",
            "fj.news.group.soc",
            "fj.news.usage",
            "fj.org",
            "fj.org.ieee",
            "fj.org.jus",
        ];
        let mut cursor = Cursor::new(test_data.join("\n").into_bytes());
        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 10);

        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_line_walker("   ", "|  ", "`--", "|--");
            c(top, &mut cursor);
        }
        cursor.rewind().unwrap();
        assert_eq!(
            Ok("fj\n|--news\n|  |--b\n|  |--config\n|  |--group\n|  |  |--archives\n|  |  |--comp\n|  |  `--soc\n|  `--usage\n`--org\n   |--ieee\n   `--jus\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_level_limit() {
        let test_data = [
            "fj",
            "fj.news",
            "fj.news.b",
            "fj.news.config",
            "fj.news.group",
            "fj.news.group.archives",
        ];
        let mut cursor = Cursor::new(test_data.join("\n").into_bytes());
        let cache = Path::create_tree::<Cursor<Vec<u8>>>(&mut cursor, '.', 2);

        let mut cursor = Cursor::new(Vec::new());

        if let Some(top) = cache.top {
            let mut c = create_line_walker("   ", "|  ", "`--", "|--");
            c(top, &mut cursor);
        }
        cursor.rewind().unwrap();
        assert_eq!(
            Ok("fj\n`--news\n   |--b\n   |--config\n   `--group\n"),
            from_utf8(cursor.get_ref())
        );
    }
    #[test]
    fn test_default() {
        let _: Cache<Path> = Default::default();
        let _: Config = Default::default();
    }
    #[test]
    fn test_stdin() {
        unsafe {
            close(0);
        }
        let config = Config::new();
        let _ = create_tree(&config);
    }
}
