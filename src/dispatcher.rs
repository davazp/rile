use nix;
use nix::libc;
use nix::unistd;

use super::commands;
use super::context::Context;
use super::key::Key;
use super::term::Term;
use super::window::{refresh_screen, Window};

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
pub fn process_user_input(term: &mut Term, win: &mut Window, context: &mut Context) {
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
