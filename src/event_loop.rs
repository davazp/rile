use std::collections::VecDeque;

use crate::commands;
use crate::read;
use crate::term::{read_key_timeout, reconciliate_term_size, Term};
use crate::window::{adjust_scroll, refresh_screen};
use crate::{Context, Key};

pub enum EventLoopError {
    Quit,
}

pub type Result<T> = std::result::Result<T, EventLoopError>;

pub struct EventLoopState {
    /// A buffer of keys that should be read by read_key. If empty,
    /// this will be re-fill on demand from the keyboard input.
    pending_input: VecDeque<Key>,

    /// If set (Some), the event loop is about to terminate with a
    /// specified Result.
    pub result: Option<Result<()>>,
}

impl EventLoopState {
    pub fn new() -> EventLoopState {
        EventLoopState {
            result: None,
            pending_input: VecDeque::new(),
        }
    }

    pub fn unpeek_keys(&mut self, keys: Vec<Key>) {
        for k in keys.into_iter() {
            self.pending_input.push_back(k);
        }
    }

    pub fn complete(&mut self, result: Result<()>) {
        self.result = Some(result)
    }

    pub fn is_exit_successfully(&self) -> bool {
        match self.result {
            Some(Ok(_)) => true,
            _ => false,
        }
    }
}

pub fn read_key(term: &mut Term, context: &mut Context) -> Key {
    context
        .event_loop
        .pending_input
        .pop_front()
        .unwrap_or_else(|| {
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
        })
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
fn process_user_input(term: &mut Term, context: &mut Context) -> std::result::Result<(), Vec<Key>> {
    let cmd = read::read_key_binding(term, context);

    let minibuffer = &mut context.buffer_list.minibuffer;
    if !context.window_list.minibuffer_focused {
        minibuffer.truncate();
    }

    // Execute the command.
    match cmd {
        Ok(handler) => {
            let _ = handler(context, term);
            Ok(())
        }
        Err(keys) => {
            if let Some(ch) = is_self_insert(&keys) {
                commands::insert_char(context, ch);
                Ok(())
            } else {
                minibuffer.set(format!("{} is undefined", Key::format_seq(&keys)));
                Err(keys)
            }
        }
    }
}

pub fn event_loop<F>(
    term: &mut Term,
    context: &mut Context,
    callback: F,
    exit_on_undefined: bool,
) -> Result<()>
where
    F: Fn(&mut Term, &mut Context),
{
    // Save the context for a recursive event loop.
    let original_result = context.event_loop.result.take();

    let result = loop {
        context.goal_column.to_preserve = false;

        match process_user_input(term, context) {
            Ok(_) => {}
            Err(keys) => {
                if exit_on_undefined {
                    context.event_loop.unpeek_keys(keys);
                    context.event_loop.complete(Ok(()));
                }
            }
        }

        if !context.goal_column.to_preserve {
            context.goal_column.column = None;
        }

        adjust_scroll(term, context);

        if let Some(result) = context.event_loop.result.take() {
            break result;
        }

        callback(term, context);
    };

    //  the saved context.
    context.event_loop.result = original_result;

    result
}
