use crate::commands;
use crate::context::Context;
use crate::key::Key;
use crate::keymap::{CommandHandler, Item};
use crate::term::{read_key_timeout, reconciliate_term_size, Term};
use crate::window::{adjust_scroll, refresh_screen};

fn read_single_key(term: &mut Term, context: &Context) -> Key {
    loop {
        if let Some(key) = read_key_timeout() {
            return key;
        } else {
            if reconciliate_term_size(term, &context.was_resized) {
                adjust_scroll(term, context);
                refresh_screen(term, context);
            }
        }
    }
}

fn read_key_binding(term: &mut Term, context: &mut Context) -> Result<CommandHandler, Vec<Key>> {
    let mut read = vec![];
    let mut keymap = &context.keymap;

    loop {
        if !read.is_empty() {
            let keys = Key::format_seq(&read) + "-";
            context.buffer_list.minibuffer.set(&keys);
            refresh_screen(term, context);
        }

        let k = read_single_key(term, context);
        let item = keymap.lookup(&k);

        read.push(k);

        match item {
            Some(Item::Command(cmd)) => return Ok(*cmd),
            Some(Item::Keymap(km)) => {
                keymap = km;
            }
            None => break Err(read),
        }
    }
}

fn is_self_insert(keys: &Vec<Key>) -> Option<char> {
    if keys.len() != 1 {
        None
    } else if let Some(ch) = keys[0].as_char() {
        Some(ch)
    } else {
        None
    }
}

/// Process user input.
pub fn process_user_input(term: &mut Term, context: &mut Context) {
    let cmd = read_key_binding(term, context);

    let minibuffer = &mut context.buffer_list.minibuffer;

    minibuffer.truncate();

    // Execute the command.
    match cmd {
        Ok(handler) => {
            let _ = handler(context, term);
        }
        Err(keys) => {
            if let Some(ch) = is_self_insert(&keys) {
                commands::insert_char(context, ch);
            } else {
                minibuffer.set(&format!("{} is undefined", Key::format_seq(&keys)));
            }
        }
    }
}
