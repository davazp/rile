use crate::context::Context;
use crate::event_loop::process_user_input;
use crate::key::Key;
use crate::keymap::{CommandHandler, Item};
use crate::term::{read_key_timeout, reconciliate_term_size, Term};
use crate::window::{adjust_scroll, refresh_screen};

pub fn read_key(term: &mut Term, context: &Context) -> Key {
    refresh_screen(term, context);
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

pub fn read_key_binding(
    term: &mut Term,
    context: &mut Context,
) -> Result<CommandHandler, Vec<Key>> {
    let mut read = vec![];
    let mut keymap = &context.keymap;

    loop {
        if !read.is_empty() {
            let keys = Key::format_seq(&read) + "-";
            context.buffer_list.minibuffer.set(keys);
            refresh_screen(term, context);
        }

        let k = read_key(term, context);
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

pub fn read_string(term: &mut Term, context: &mut Context, prompt: &str) -> Result<String, ()> {
    context.buffer_list.minibuffer.set(prompt);
    context.buffer_list.minibuffer_focused = true;

    context.cursor.line = 0;
    context.cursor.column = prompt.len();

    while process_user_input(term, context) {}

    context.buffer_list.minibuffer_focused = false;
    context.buffer_list.minibuffer.truncate();

    Ok("test".to_string())
}
