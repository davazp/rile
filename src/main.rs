//! sted is a simple editor written in Rust.
//!

extern crate signal_hook;

mod buffer;
mod commands;
mod context;
mod key;
mod term;
mod window;

use buffer::Buffer;
use context::{Context, Cursor};
use key::Key;
use term::{get_window_size, with_raw_mode, Term};
use window::{adjust_scroll, refresh_screen, Window};

use nix;
use nix::libc;
use nix::unistd;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::{App, Arg};

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

const ARROW_UP: &'static [u8; 2] = b"[A";
const ARROW_DOWN: &'static [u8; 2] = b"[B";
const ARROW_RIGHT: &'static [u8; 2] = b"[C";
const ARROW_LEFT: &'static [u8; 2] = b"[D";

/// Read and return a key.
fn read_key() -> Key {
    let mut buf = [0u8];
    unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();
    let cmd = buf[0] as u32;
    if cmd == 0x1b {
        let mut seq: [u8; 2] = [0; 2];
        unistd::read(libc::STDIN_FILENO, &mut seq).unwrap();

        if seq[1] == 0 {
            Key::from_code(seq[0] as u32).alt()
        } else {
            match &seq {
                ARROW_UP => Key::parse_unchecked("C-p"),
                ARROW_DOWN => Key::parse_unchecked("C-n"),
                ARROW_RIGHT => Key::parse_unchecked("C-f"),
                ARROW_LEFT => Key::parse_unchecked("C-b"),
                _ => Key::from_code(cmd),
            }
        }
    } else {
        Key::from_code(cmd)
    }
}

/// Process user input.
fn process_user_input(term: &mut Term, win: &mut Window, context: &mut Context) {
    let k = read_key();
    context.to_refresh = true;
    if k == Key::parse_unchecked("C-a") {
        commands::move_beginning_of_line(context);
    } else if k == Key::parse_unchecked("C-e") {
        commands::move_end_of_line(context);
    } else if k == Key::parse_unchecked("C-f") {
        commands::forward_char(context);
    } else if k == Key::parse_unchecked("C-b") {
        commands::backward_char(context);
    } else if k == Key::parse_unchecked("C-p") {
        commands::previous_line(context);
    } else if k == Key::parse_unchecked("C-n") {
        commands::next_line(context);
    } else if k == Key::parse_unchecked("C-d") {
        commands::delete_char(context);
    } else if k == Key::parse_unchecked("DEL") {
        commands::delete_backward_char(context);
    } else if k == Key::parse_unchecked("C-k") {
        commands::kill_line(context);
    } else if k == Key::parse_unchecked("RET") || k == Key::parse_unchecked("C-j") {
        commands::newline(context);
    } else if k == Key::parse_unchecked("TAB") {
        commands::indent_line(context);
    } else if k == Key::parse_unchecked("C-x") {
        context.minibuffer.set("C-x ");
        refresh_screen(term, win, context);
        let k = read_key();
        if k == Key::parse_unchecked("C-c") {
            context.to_exit = true;
        } else if k == Key::parse_unchecked("C-s") {
            commands::save_buffer(context);
        }
    } else if k == Key::parse_unchecked("C-v") {
        commands::next_screen(context, win, term);
    } else if k == Key::parse_unchecked("M-v") {
        commands::previous_screen(context, win, term);
    } else {
        if let Some(ch) = k.as_char() {
            commands::insert_char(context, ch)
        } else {
            context.minibuffer.set(&format!("{:?}", k));
        }
    }
}

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
        to_exit: false,
        to_refresh: false,
        to_preserve_goal_column: false,
    };

    let mut window = Window {
        show_lines: false,
        scroll_line: 0,
    };

    // Detect when the terminal was resized
    let was_resize = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGWINCH, Arc::clone(&was_resize)).unwrap();

    let mut term = Term::new();

    term.enable_alternative_screen_buffer();

    refresh_screen(&mut term, &mut window, &context);

    with_raw_mode(|| loop {
        if was_resize.load(Ordering::Relaxed) {
            let (rows, columns) = get_window_size();
            term.rows = rows;
            term.columns = columns;
            adjust_scroll(&mut term, &mut window, &mut context);
            refresh_screen(&mut term, &mut window, &context);
            was_resize.store(false, Ordering::Relaxed);
        }

        context.to_preserve_goal_column = false;
        context.to_refresh = false;

        process_user_input(&mut term, &mut window, &mut context);

        if context.to_refresh {
            adjust_scroll(&mut term, &mut window, &mut context);
            refresh_screen(&mut term, &mut window, &context);
        }

        context.minibuffer.truncate();

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
