use crate::event_loop::{self, event_loop};
use crate::keymap::{CommandHandler, Item};
use crate::term::{read_key_timeout, reconciliate_term_size, Term};
use crate::window::{adjust_scroll, message, refresh_screen};
use crate::{Context, Key};

pub fn read_key(term: &mut Term, context: &mut Context) -> Key {
    refresh_screen(term, context).unwrap();
    loop {
        if let Some(key) = read_key_timeout() {
            return key;
        } else {
            if reconciliate_term_size(term, &context.was_resized) {
                adjust_scroll(term, context);
                refresh_screen(term, context).unwrap();
            }
        }
    }
}

pub fn read_key_binding(
    term: &mut Term,
    context: &mut Context,
) -> Result<CommandHandler, Vec<Key>> {
    let mut read = vec![];

    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref(window.buffer_ref);
    let mut keymap = buffer.keymap.clone();

    loop {
        if !read.is_empty() {
            let keys = Key::format_seq(&read) + "-";
            message(context, keys);
            refresh_screen(term, context).unwrap();
        }

        let k = read_key(term, context);
        let item = keymap.lookup(&k);

        read.push(k);

        match item {
            Some(Item::Command(cmd)) => break Ok(cmd),
            Some(Item::Keymap(km)) => {
                keymap = km;
            }
            None => break Err(read),
        }
    }
}

pub fn read_string<F>(
    term: &mut Term,
    context: &mut Context,
    prompt: &str,
    callback: F,
) -> event_loop::Result<String>
where
    F: Fn(&mut Term, &mut Context),
{
    context.buffer_list.minibuffer.set(prompt);
    context.window_list.minibuffer_focused = true;

    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);

    buffer.cursor.line = 0;
    buffer.cursor.column = prompt.len();

    let result =
        event_loop(term, context, callback).map(|_| context.buffer_list.minibuffer.to_string());

    context.buffer_list.minibuffer.truncate();
    context.window_list.minibuffer_focused = false;

    result
}
