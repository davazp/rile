use crate::commands;
use crate::context::Context;
use crate::key::Key;
use crate::keymap::{CommandHandler, Item};
use crate::term::{read_key, Term};
use crate::window::refresh_screen;

fn read_key_binding(term: &mut Term, context: &mut Context) -> Result<CommandHandler, Vec<Key>> {
    let mut read = vec![];
    let mut keymap = &context.keymap;

    loop {
        if !read.is_empty() {
            let keys = format!(
                "{}",
                read.iter()
                    .map(|k| format!("{}", k))
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            context.minibuffer.set(&keys);
            refresh_screen(term, context);
        }

        let k = read_key();
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

    context.minibuffer.truncate();

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
