/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

pub const HISTORY_SIZE: usize = 10;
const HISTORY_COL_SIZE: usize = 32;

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
    pub fn search_word_highlight(&self, text_buffer: &gtk::TextBuffer, word: &String) {
        let start = text_buffer.get_start_iter();
        let end = text_buffer.get_end_iter();
        self.search_word_iter(&text_buffer, &start, &end, word.as_str());
    }
    fn search_word_iter(
        &self,
        text_buffer: &gtk::TextBuffer,
        start: &gtk::TextIter,
        end: &gtk::TextIter,
        word: &str,
    ) {
        if let Some(t) = start.forward_search(word, gtk::TextSearchFlags::all(), None) {
            let (mut match_start, match_end) = t;
            match_start.forward_chars(1);
            self.search_word_iter(&text_buffer, &match_end, end, word);
        }
    }
    fn create_highlight_index(&self, mut start: gtk::TextIter) -> Vec<gtk::TextIter> {
        enum Status {
            Ready,
            Number,
            Keyword,
            String,
        }
        let mut vec: Vec<gtk::TextIter> = Vec::new();
        let mut state = Status::Ready;

        loop {
            if let Some(c) = start.get_char() {
                match state {
                    Status::Ready => match c {
                        '(' | ')' | ' ' | '\n' => state = Status::Ready,
                        '"' => {
                            vec.push(start.clone());
                            state = Status::String;
                        }
                        _ => {
                            if true == c.is_digit(10) {
                                vec.push(start.clone());
                                state = Status::Number;
                            } else if true == c.is_lowercase() {
                                vec.push(start.clone());
                                state = Status::Keyword;
                            } else {
                                state = Status::Ready;
                            }
                        }
                    },
                    Status::Number => match c {
                        '(' | ')' | ' ' | '\n' => {
                            vec.push(start.clone());
                            state = Status::Ready;
                        }
                        _ => {
                            if false == c.is_digit(10) && c != '.' {
                                vec.pop();
                                state = Status::Ready;
                            }
                        }
                    },
                    Status::Keyword => match c {
                        '(' | ')' | ' ' | '\n' => {
                            vec.push(start.clone());
                            state = Status::Ready;
                        }
                        _ => {
                            if false == c.is_lowercase() && c != '-' {
                                vec.pop();
                                state = Status::Ready;
                            }
                        }
                    },
                    Status::String => match c {
                        '"' => {
                            vec.push(start.clone());
                            state = Status::Ready;
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
        vec
    }
    fn update_highlight(&self, text_buffer: &gtk::TextBuffer, vec: Vec<gtk::TextIter>) {
        if vec.len() == 0 {
            return;
        }
        let mut s = vec[0].clone();
        for (i, mut r) in vec.into_iter().enumerate() {
            if (i % 2) == 0 {
                s = r;
            } else {
                if let Some(w) = s.get_slice(&r) {
                    let mut b = w.as_str().bytes();
                    if let Some(b'"') = b.next() {
                        r.forward_chars(1);
                        text_buffer.apply_tag(&*(self.string), &s, &r);
                        continue;
                    }
                    let mut num = true;
                    for c in w.as_str().chars() {
                        if false == c.is_digit(10) && c != '.' {
                            num = false;
                            break;
                        }
                    }
                    if true == num {
                        text_buffer.apply_tag(&*(self.digit), &s, &r);
                        continue;
                    }
                    for word in vec![
                        "define", "lambda", "if", "map", "filter", "reduce", "let", "set!", "and",
                        "or", "not", "cond", "case", "begin", "else", "apply", "delay", "force",
                        "quote", "for-each",
                    ] {
                        if w.as_str() == word {
                            text_buffer.apply_tag(&*(self.keyword), &s, &r);
                            break;
                        }
                    }
                }
            }
        }
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

        let vec = self.create_highlight_index(start);
        self.update_highlight(&text_buffer, vec);
    }
}
