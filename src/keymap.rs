use std::collections::HashMap;

use crate::commands;
use crate::term::Term;
use crate::{Context, Key};

pub type CommandHandler = fn(&mut Context, term: &mut Term) -> commands::Result;

#[derive(Clone)]
pub enum Item {
    Command(CommandHandler),
    Keymap(Keymap),
}

#[derive(Clone)]
pub struct Keymap {
    inner: HashMap<Key, Item>,
}

impl Keymap {
    pub fn new() -> Keymap {
        Keymap {
            inner: HashMap::new(),
        }
    }

    pub fn define_key(&mut self, keyspec: &str, f: CommandHandler) {
        let key = Key::parse_unchecked(keyspec);
        self.inner.insert(key, Item::Command(f));
    }

    pub fn define_keymap(&mut self, keyspec: &str, keymap: Keymap) {
        let key = Key::parse_unchecked(keyspec);
        self.inner.insert(key, Item::Keymap(keymap));
    }

    pub fn lookup(&self, key: &Key) -> Option<&Item> {
        self.inner.get(key)
    }

    pub fn defaults() -> Keymap {
        let mut keymap = Keymap::new();
        let mut c_x = Keymap::new();

        keymap.define_key("C-a", commands::move_beginning_of_line);
        keymap.define_key("C-e", commands::move_end_of_line);
        keymap.define_key("C-f", commands::forward_char);
        keymap.define_key("C-b", commands::backward_char);
        keymap.define_key("C-p", commands::previous_line);
        keymap.define_key("C-n", commands::next_line);
        keymap.define_key("C-d", commands::delete_char);

        keymap.define_key("DEL", commands::delete_backward_char);
        keymap.define_key("C-k", commands::kill_line);
        keymap.define_key("RET", commands::newline);
        keymap.define_key("C-j", commands::newline);
        keymap.define_key("TAB", commands::indent_line);

        keymap.define_key("M-<", commands::beginning_of_buffer);
        keymap.define_key("M->", commands::end_of_buffer);

        keymap.define_key("C-v", commands::next_screen);
        keymap.define_key("M-v", commands::previous_screen);

        keymap.define_key("C-g", commands::keyboard_quit);
        keymap.define_key("C-s", commands::isearch_forward);

        c_x.define_key("C-s", commands::save_buffer);
        c_x.define_key("C-c", commands::kill_emacs);
        keymap.define_keymap("C-x", c_x);

        keymap
    }
}
