use crate::commands;
use crate::read;
use crate::term::Term;
use crate::window::adjust_scroll;
use crate::{Context, Key};

type Result = std::result::Result<(), ()>;

pub struct EventLoopState {
    /// If set (Some), the event loop is about to terminate with a
    /// specified Result.
    pub result: Option<Result>,
}

impl EventLoopState {
    pub fn new() -> EventLoopState {
        EventLoopState { result: None }
    }

    pub fn complete(&mut self, result: Result) {
        self.result = Some(result)
    }

    pub fn is_exit_successfully(&self) -> bool {
        match self.result {
            Some(Ok(_)) => true,
            _ => false,
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
pub fn process_user_input(term: &mut Term, context: &mut Context) -> bool {
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
            if keys == vec![Key::parse_unchecked("C-g")] {
                return false;
            }

            if let Some(ch) = is_self_insert(&keys) {
                commands::insert_char(context, ch);
            } else {
                minibuffer.set(format!("{} is undefined", Key::format_seq(&keys)));
            }
        }
    };

    true
}

pub fn event_loop(term: &mut Term, context: &mut Context) -> bool {
    // Save the context for a recursive event loop.
    let original_result = context.event_loop.result;

    let status = loop {
        context.event_loop.result = None;
        process_user_input(term, context);

        adjust_scroll(term, context);

        if let Some(result) = context.event_loop.result {
            break result;
        }
    };

    // Restore the saved context.
    context.event_loop.result = original_result;

    status.is_ok()
}
