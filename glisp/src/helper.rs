/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::io::{Error, ErrorKind};

pub const HISTORY_SIZE: usize = 10;
const HISTORY_COL_SIZE: usize = 32;

//--------------------------------------------------------
// LISP History table
//--------------------------------------------------------
#[derive(Clone)]
pub struct History {
    menu: gtk::MenuItem,
    children: Rc<RefCell<LinkedList<(gtk::MenuItem, String)>>>,
    max_item: usize,
}
impl History {
    pub fn new(n: usize) -> Self {
        History {
            menu: gtk::MenuItem::new_with_mnemonic("_History"),
            children: Rc::new(RefCell::new(LinkedList::new())),
            max_item: n,
        }
    }
    pub fn menu(&self) -> &gtk::MenuItem {
        &self.menu
    }
    pub fn push(&self, exp: &String, tb: &gtk::TextBuffer) {
        let s = String::from(exp).replace("\n", " ");
        let c = if let Some(ref v) = s.get(0..HISTORY_COL_SIZE) {
            gtk::MenuItem::new_with_mnemonic(format!("{} ..", v).as_str())
        } else {
            gtk::MenuItem::new_with_mnemonic(s.as_str())
        };
        let exp_ = exp.clone();
        let exp_ = exp_.into_boxed_str();
        let text_buffer = tb.clone();
        c.connect_activate(move |_| {
            text_buffer.set_text(&exp_);
        });

        if None == self.menu.get_submenu() {
            self.menu.set_submenu(Some(&gtk::Menu::new()));
        }
        if let Some(w) = self.menu.get_submenu() {
            if let Ok(w) = w.downcast::<gtk::Menu>() {
                w.append(&c);

                let mut h = self.children.borrow_mut();
                h.push_front((c, exp.clone()));
                if h.len() > self.max_item {
                    if let Some((c, _)) = h.pop_back() {
                        w.remove(&c);
                    }
                }
                w.show_all();
            }
        }
    }
    pub fn is_once(&self, exp: &String) -> bool {
        for (_, e) in self.children.borrow().iter() {
            if e == exp {
                return true;
            }
        }
        false
    }
}
//--------------------------------------------------------
// LISP Source code view
//--------------------------------------------------------
#[derive(Clone)]
pub struct SourceView {
    keyword: Box<gtk::TextTag>,
    string: Box<gtk::TextTag>,
    digit: Box<gtk::TextTag>,
}
impl SourceView {
    pub fn new(tb: &gtk::TextBuffer) -> Self {
        let keyword = gtk::TextTag::new(Some("keyword"));
        keyword.set_property_foreground(Some("#0033ee"));

        let string = gtk::TextTag::new(Some("string"));
        string.set_property_foreground(Some("#660000"));

        let digit = gtk::TextTag::new(Some("digit"));
        digit.set_property_foreground(Some("#009900"));

        let table: gtk::TextTagTable = tb.get_tag_table().unwrap();
        table.add(&keyword);
        table.add(&string);
        table.add(&digit);

        SourceView {
            keyword: Box::new(keyword),
            string: Box::new(string),
            digit: Box::new(digit),
        }
    }
    fn core_highlight(&self, text_buffer: &gtk::TextBuffer, mut start: gtk::TextIter) {
        enum Status {
            Ready,
            Number,
            Keyword,
            String,
        }
        let mut vec: Option<gtk::TextIter> = None;
        let mut state = Status::Ready;
        loop {
            if let Some(c) = start.get_char() {
                match state {
                    Status::Ready => match c {
                        '(' | ')' | ' ' | '\n' => state = Status::Ready,
                        '"' => {
                            vec = Some(start.clone());
                            state = Status::String;
                        }
                        _ => {
                            if true == c.is_digit(10) {
                                vec = Some(start.clone());
                                state = Status::Number;
                            } else if true == c.is_lowercase() {
                                vec = Some(start.clone());
                                state = Status::Keyword;
                            } else {
                                state = Status::Ready;
                            }
                        }
                    },
                    Status::Number => match c {
                        '(' | ')' | ' ' | '\n' => {
                            if let Some(s) = &vec {
                                if self.is_number(s, &start) {
                                    text_buffer.apply_tag(&*(self.digit), s, &start);
                                }
                            }
                            state = Status::Ready;
                        }
                        _ => {}
                    },
                    Status::Keyword => match c {
                        '(' | ')' | ' ' | '\n' => {
                            if let Some(s) = &vec {
                                if self.is_keyword(s, &start) {
                                    text_buffer.apply_tag(&*(self.keyword), s, &start);
                                }
                            }
                            state = Status::Ready;
                        }
                        _ => {}
                    },
                    Status::String => match c {
                        '"' => {
                            start.forward_char();
                            if let Some(s) = &vec {
                                text_buffer.apply_tag(&*(self.string), s, &start);
                            }
                            state = Status::Ready;
                            continue;
                        }
                        _ => {}
                    },
                }
            } else {
                break;
            }
            if false == start.forward_char() {
                break;
            }
        }
    }
    fn is_keyword(&self, s: &gtk::TextIter, r: &gtk::TextIter) -> bool {
        if let Some(w) = s.get_slice(r) {
            for word in vec![
                "define", "lambda", "if", "map", "filter", "reduce", "let", "set!", "and", "or",
                "not", "cond", "case", "begin", "else", "apply", "delay", "force", "quote",
                "for-each",
            ] {
                if w.as_str() == word {
                    return true;
                }
            }
        }
        return false;
    }
    fn is_number(&self, s: &gtk::TextIter, r: &gtk::TextIter) -> bool {
        if let Some(w) = s.get_slice(r) {
            for c in w.as_str().chars() {
                if false == c.is_digit(10) && c != '.' {
                    return false;
                }
            }
            return true;
        }
        return false;
    }
    pub fn do_highlight(&self, text_buffer: &gtk::TextBuffer) {
        // println!("{}", std::mem::size_of_val(&self.string));
        // println!("{}", std::mem::size_of::<gtk::TextBuffer>());
        // println!("{}", std::any::type_name::<gtk::TextBuffer>());
        let start = text_buffer.get_start_iter();
        let end = text_buffer.get_end_iter();

        text_buffer.remove_tag(&*(self.keyword), &start, &end);
        text_buffer.remove_tag(&*(self.digit), &start, &end);
        text_buffer.remove_tag(&*(self.string), &start, &end);

        self.core_highlight(text_buffer, start);
    }
}
//------------------------------------------------------------------------
// support search function
// ex) search_word_highlight(text_buffer, "search", "frame");
//------------------------------------------------------------------------
pub fn search_word_highlight(text_buffer: &gtk::TextBuffer, tag_name: &str, word: &str) {
    let table: gtk::TextTagTable = text_buffer.get_tag_table().unwrap();

    let search_tag = if let Some(tag) = table.lookup(tag_name) {
        tag
    } else {
        let search_tag = gtk::TextTag::new(Some(tag_name));
        search_tag.set_property_foreground(Some("#ee0000"));
        search_tag.set_property_background(Some("#eaeaea"));
        table.add(&search_tag);
        search_tag
    };

    let start = text_buffer.get_start_iter();
    let end = text_buffer.get_end_iter();
    text_buffer.remove_tag(&search_tag, &start, &end);

    if word.is_empty() {
        return;
    }
    search_word_iter(&search_tag, text_buffer, &start, &end, word);
    //------------------------------------------------------------------------
    // iter child function
    //------------------------------------------------------------------------
    fn search_word_iter(
        word_tag: &gtk::TextTag,
        text_buffer: &gtk::TextBuffer,
        start: &gtk::TextIter,
        end: &gtk::TextIter,
        word: &str,
    ) {
        if let Some(t) = start.forward_search(word, gtk::TextSearchFlags::all(), None) {
            let (match_start, match_end) = t;
            text_buffer.apply_tag(word_tag, &match_start, &match_end);
            search_word_iter(word_tag, text_buffer, &match_end, end, word);
        }
    }
}
//--------------------------------------------------------
// Load LISP programe(https://github.com/hidekuno/picture-language)
//--------------------------------------------------------
pub fn load_demo_program(dir: &str) -> std::io::Result<String> {
    fn get_program_name(vec: Vec<&str>) -> std::io::Result<Option<String>> {
        let mut program: Vec<String> = Vec::new();
        let mut path = PathBuf::new();

        path.push(match env::var("HOME") {
            Ok(v) => v,
            Err(_) => "/root".into(),
        });
        for dir in vec {
            path.push(dir);
        }
        if false == path.as_path().exists() {
            return Ok(None);
        }
        for entry in fs::read_dir(path)? {
            let dir = entry?;
            let path = dir.path();
            let f = path.to_str().unwrap();
            if f.ends_with(".scm") {
                program.push(format!("(load-file \"{}\")", f));
            }
        }
        program.sort();
        Ok(Some(program.join("\n")))
    }
    for v in vec![vec!["picture-language", dir], vec![dir]] {
        match get_program_name(v) {
            Ok(s) => match s {
                Some(s) => return Ok(s),
                None => continue,
            },
            Err(e) => return Err(e),
        }
    }
    Err(Error::new(ErrorKind::Other, "Not Installed Scheme Program"))
}
