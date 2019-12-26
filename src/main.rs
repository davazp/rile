//! sted is a simple editor written in Rust.
//!

extern crate signal_hook;

mod buffer;
mod key;
mod term;

use buffer::Buffer;
use key::Key;
use term::{get_window_size, with_raw_mode, ErasePart, Term};

use nix;
use nix::libc;
use nix::unistd;

use std::char;
use std::cmp;
use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::{App, Arg};

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

/// A cursor into a buffer content
struct Cursor {
    line: usize,
    column: usize,
}

/// The state of the editor.
struct Context {
    /// The column that a following [`next-line`](fn.next_line.html) or
    /// [`previous_line`](fn.previous_line.html) should try to move
    /// to. This is automatically reset to `None` after each user
    /// command is processed, unless
    /// [`to_preserve_goal_column`](#structfield.to_preserve_goal_column)
    /// is set to true by the command.
    goal_column: Option<usize>,

    cursor: Cursor,
    current_buffer: Buffer,

    minibuffer: Buffer,

    // Result of a command. They will take effect once a full command
    // has been processed.
    to_exit: bool,
    to_refresh: bool,

    /// If set by a command, [`goal_column`](#structfield.goal_column) won't be reset after it.
    to_preserve_goal_column: bool,
}

// Rendering
//
//

/// Adjust the scroll level so the cursor is on the screen.
///
/// If the cursor is after the screen, the screen will be scrolled so the
///
/// If the cursor is before the screen,
fn adjust_scroll(term: &Term, window: &mut Window, context: &mut Context) {
    if context.cursor.line < window.scroll_line {
        window.scroll_line = context.cursor.line;
    }
    if context.cursor.line > window.scroll_line + term.rows - 2 - 1 {
        window.scroll_line += 1;
    }
}

struct Window {
    scroll_line: usize,
    show_lines: bool,
}
impl Window {
    fn get_window_lines(&self, term: &Term) -> usize {
        term.rows - 2
    }

    fn get_pad_width(&self, term: &Term) -> usize {
        if self.show_lines {
            let last_linenum_width =
                format!("{}", self.scroll_line + self.get_window_lines(term)).len();
            last_linenum_width + 1
        } else {
            0
        }
    }

    fn render_cursor(&self, term: &mut Term, context: &Context) {
        term.set_cursor(
            context.cursor.line - self.scroll_line + 1,
            context.cursor.column + self.get_pad_width(term) + 1,
        );
    }

    fn render_window(&self, term: &mut Term, context: &Context) {
        let offset = self.get_pad_width(term);
        let window_columns = term.columns - offset;

        term.set_cursor(1, 1);

        let window_lines = term.rows - 2;

        let offset = self.get_pad_width(term);

        let buffer = &context.current_buffer;

        // Main window
        for row in 0..window_lines {
            let linenum = row + self.scroll_line;

            if let Some(line) = buffer.get_line(linenum) {
                if self.show_lines {
                    term.csi("38;5;240m");
                    term.write(&format!("{:width$} ", linenum + 1, width = offset - 1));
                }

                term.csi("m");
                term.write(&line[..cmp::min(line.len(), window_columns)]);
            }

            term.erase_line(ErasePart::ToEnd);
            term.write("\r\n");
        }
        term.csi("m");
    }

    fn render_modeline(&self, term: &mut Term, context: &Context) {
        term.csi("38;5;15m");
        term.csi("48;5;236m");
        // On MacOsX's terminal, when you erase a line it won't fill the
        // full line with the current attributes, unlike ITerm. So we use
        // `write_line` to pad the string with spaces.
        write_line(
            term,
            &format!(
                "  {}  {}% L{}",
                context
                    .current_buffer
                    .filename
                    .as_ref()
                    .unwrap_or(&"*scratch*".to_string()),
                100 * (context.cursor.line + 1) / context.current_buffer.lines_count(),
                context.cursor.line + 1
            ),
            term.columns,
        );
    }
}

fn render_minibuffer(term: &mut Term, context: &Context) {
    term.csi("m");
    write_line(
        term,
        &format!("{}", context.minibuffer.to_string()),
        term.columns,
    );
}

/// Refresh the screen.
///
/// Ensure the terminal reflects the latest state of the editor.
fn refresh_screen(term: &mut Term, win: &Window, context: &Context) {
    term.hide_cursor();

    win.render_window(term, context);
    win.render_modeline(term, context);
    render_minibuffer(term, context);

    win.render_cursor(term, context);

    term.show_cursor();
    term.flush()
}

fn write_line(term: &mut Term, str: &str, width: usize) {
    assert!(str.len() <= width);
    let padded = format!("{:width$}", str, width = width);
    term.write(&padded);
}

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

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

fn move_beginning_of_line(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_unchecked(context.cursor.line);
    let indentation = get_line_indentation(line);
    context.cursor.column = if context.cursor.column <= indentation {
        0
    } else {
        indentation
    };
}

fn move_end_of_line(context: &mut Context) {
    let eol = context
        .current_buffer
        .get_line_unchecked(context.cursor.line)
        .len();
    context.cursor.column = eol;
}

fn forward_char(context: &mut Context) {
    let len = context
        .current_buffer
        .get_line_unchecked(context.cursor.line)
        .len();
    if context.cursor.column < len {
        context.cursor.column += 1;
    } else {
        if next_line(context) {
            context.cursor.column = 0;
        };
    }
}

fn backward_char(context: &mut Context) {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
    } else {
        if previous_line(context) {
            move_end_of_line(context);
        };
    }
}

fn get_or_set_gaol_column(context: &mut Context) -> usize {
    // We set `to_preserve_goal_column` to ensure the goal_column is
    // not lost for the next command.
    context.to_preserve_goal_column = true;
    *context.goal_column.get_or_insert(context.cursor.column)
}

fn next_line(context: &mut Context) -> bool {
    if context.cursor.line < context.current_buffer.lines_count() - 1 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line += 1;
        context.cursor.column = cmp::min(
            context
                .current_buffer
                .get_line_unchecked(context.cursor.line)
                .len(),
            goal_column,
        );
        true
    } else {
        context.minibuffer.set("End of buffer");
        false
    }
}

fn previous_line(context: &mut Context) -> bool {
    if context.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line -= 1;
        context.cursor.column = cmp::min(
            context
                .current_buffer
                .get_line_unchecked(context.cursor.line)
                .len(),
            goal_column,
        );
        true
    } else {
        context.minibuffer.set("Beginning of buffer");
        false
    }
}

fn insert_char(context: &mut Context, ch: char) {
    let idx = context.cursor.column;
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    line.insert(idx, ch);
    context.cursor.column += 1;
}

fn delete_char(context: &mut Context) {
    forward_char(context);
    delete_backward_char(context);
}

fn delete_backward_char(context: &mut Context) {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
        context
            .current_buffer
            .remove_char_at(context.cursor.line, context.cursor.column);
    } else if context.cursor.line > 0 {
        let line = context.current_buffer.remove_line(context.cursor.line);
        let previous_line = context
            .current_buffer
            .get_line_mut_unchecked(context.cursor.line - 1);
        let previous_line_original_length = previous_line.len();
        previous_line.push_str(&line);

        context.cursor.line -= 1;
        context.cursor.column = previous_line_original_length;
    }
}

fn kill_line(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    if context.cursor.column == line.len() {
        if context.cursor.line < context.current_buffer.lines_count() - 1 {
            delete_char(context);
        }
    } else {
        line.drain(context.cursor.column..);
    }
}

fn newline(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    let newline = line.split_off(context.cursor.column);
    context
        .current_buffer
        .insert_at(context.cursor.line + 1, newline);

    context.cursor.line += 1;
    context.cursor.column = 0;
}

fn indent_line(context: &mut Context) {
    let line = &context
        .current_buffer
        .get_line_unchecked(context.cursor.line);
    let indent = get_line_indentation(line);
    if context.cursor.column < indent {
        context.cursor.column = indent;
    }
}

fn save_buffer(context: &mut Context) {
    let buffer = &context.current_buffer;
    let contents = buffer.to_string();
    if let Some(filename) = &buffer.filename {
        match fs::write(filename, contents) {
            Ok(_) => {
                context.minibuffer.set(&format!("Wrote {}", filename));
            }
            Err(_) => {
                context.minibuffer.set("Could not save file");
            }
        }
    } else {
        context.minibuffer.set("No file");
    }
}

const CONTEXT_LINES: usize = 2;

fn next_screen(context: &mut Context, window: &mut Window, term: &Term) {
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    let target = window.scroll_line + offset;
    if target < context.current_buffer.lines_count() {
        window.scroll_line = target;
        context.cursor.line = target;
    } else {
        context.minibuffer.set("End of buffer");
    }
}

fn previous_screen(context: &mut Context, window: &mut Window, term: &Term) {
    if window.scroll_line == 0 {
        context.minibuffer.set("Beginning of buffer");
        return;
    }
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    context.cursor.line = window.scroll_line + CONTEXT_LINES;
    window.scroll_line = if let Some(scroll_line) = window.scroll_line.checked_sub(offset) {
        scroll_line
    } else {
        0
    };
}

/// Process user input.
fn process_user_input(term: &mut Term, win: &mut Window, context: &mut Context) {
    let k = read_key();
    context.to_refresh = true;
    if k == Key::parse_unchecked("C-a") {
        move_beginning_of_line(context);
    } else if k == Key::parse_unchecked("C-e") {
        move_end_of_line(context);
    } else if k == Key::parse_unchecked("C-f") {
        forward_char(context);
    } else if k == Key::parse_unchecked("C-b") {
        backward_char(context);
    } else if k == Key::parse_unchecked("C-p") {
        previous_line(context);
    } else if k == Key::parse_unchecked("C-n") {
        next_line(context);
    } else if k == Key::parse_unchecked("C-d") {
        delete_char(context);
    } else if k == Key::parse_unchecked("DEL") {
        delete_backward_char(context);
    } else if k == Key::parse_unchecked("C-k") {
        kill_line(context);
    } else if k == Key::parse_unchecked("RET") || k == Key::parse_unchecked("C-j") {
        newline(context);
    } else if k == Key::parse_unchecked("TAB") {
        indent_line(context);
    } else if k == Key::parse_unchecked("C-x") {
        context.minibuffer.set("C-x ");
        refresh_screen(term, win, context);
        let k = read_key();
        if k == Key::parse_unchecked("C-c") {
            context.to_exit = true;
        } else if k == Key::parse_unchecked("C-s") {
            save_buffer(context);
        }
    } else if k == Key::parse_unchecked("C-v") {
        next_screen(context, win, term);
    } else if k == Key::parse_unchecked("M-v") {
        previous_screen(context, win, term);
    } else {
        if let Some(ch) = k.as_char() {
            insert_char(context, ch)
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
