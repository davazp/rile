use crate::commands;
use crate::context::Context;
use crate::key::Key;
use crate::read;
use crate::term::Term;

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
    let cmd = read::read_key_binding(term, context);
    let minibuffer = &mut context.buffer_list.minibuffer;

    if !context.buffer_list.minibuffer_focused {
        minibuffer.truncate();
    }

    // Execute the command.
    match cmd {
        Ok(handler) => {
            let _ = handler(context, term);
        }
        Err(keys) => {
            if let Some(ch) = is_self_insert(&keys) {
                commands::insert_char(context, ch);
            } else {
                minibuffer.set(format!("{} is undefined", Key::format_seq(&keys)));
            }
        }
    }
}
