use crate::buffer::Buffer;
use crate::commands;
use crate::context::Context;
use crate::key::Key;
use crate::keymap::{CommandHandler, Item, Keymap};
use crate::term::{read_key, Term};

fn read_key_binding(
    minibuffer: &mut Buffer,
    keymap: &Keymap,
    mut read: Vec<Key>,
) -> Result<CommandHandler, Vec<Key>> {
    if !read.is_empty() {
        minibuffer.set(&format!(
            "{}",
            read.iter()
                .map(|k| format!("{}", k))
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }

    let k = read_key();
    let item = keymap.lookup(&k);

    read.push(k);

    match item {
        Some(Item::Command(cmd)) => {
            minibuffer.truncate();
            Ok(*cmd)
        }
        Some(Item::Keymap(km)) => read_key_binding(minibuffer, &km, read),
        None => Err(read),
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
pub fn process_user_input(term: &Term, context: &mut Context) {
    let cmd = read_key_binding(&mut context.minibuffer, &context.keymap, vec![]);
    // Execute the command.
    match cmd {
        Ok(handler) => {
            let _ = handler(context, term);
        }
        Err(keys) => {
            if let Some(ch) = is_self_insert(&keys) {
                commands::insert_char(context, ch);
            } else {
                context.minibuffer.set(&format!("{:?} is undefined", keys));
            }
        }
    }
}
