//! sted is a simple editor written in Rust.
//!

extern crate signal_hook;

mod buffer;
mod commands;
mod context;
mod dispatcher;
mod key;
mod keymap;
mod term;
mod window;

use buffer::Buffer;
use context::{Context, Cursor};
use dispatcher::process_user_input;
use keymap::Keymap;
use term::{with_raw_mode, Term};
use window::{adjust_scroll, refresh_screen, Window};

use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::{App, Arg};

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

/// The main entry point of the editor.
fn main() {
    let matches = App::new(PKG_NAME)
        .version(
            format!(
                "{} (git: {})",
                PKG_VERSION,
                PKG_GIT_COMMIT.map(|c| &c[0..8]).unwrap_or("unknown")
            )
            .as_ref(),
        )
        .author(PKG_AUTHORS)
        .about(PKG_DESCRIPTION)
        .arg(Arg::with_name("FILE").help("Input file").index(1))
        .get_matches();

    let file_arg = matches.value_of("FILE");

    let mut context = Context {
        goal_column: None,
        cursor: Cursor { line: 0, column: 0 },

        minibuffer: Buffer::new(),
        current_buffer: if let Some(filename) = file_arg {
            Buffer::from_file(filename)
        } else {
            Buffer::from_string("")
        },

        keymap: Keymap::defaults(),

        window: Window::new(),

        was_resized: Arc::new(AtomicBool::new(false)),

        to_exit: false,
        to_preserve_goal_column: false,
    };

    signal_hook::flag::register(signal_hook::SIGWINCH, context.was_resized.clone()).unwrap();

    let mut term = Term::new();

    term.enable_alternative_screen_buffer();

    refresh_screen(&mut term, &context);

    with_raw_mode(|| loop {
        context.to_preserve_goal_column = false;

        process_user_input(&mut term, &mut context);

        adjust_scroll(&mut term, &mut context);
        refresh_screen(&mut term, &context);

        if !context.to_preserve_goal_column {
            context.goal_column = None;
        }

        if context.to_exit {
            break;
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");

    term.disable_alternative_screen_buffer();
    term.show_cursor();
    term.flush();
}
