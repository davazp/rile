//! sted is a simple editor written in Rust.
//!

extern crate signal_hook;

use nix;
use nix::libc;
use nix::sys::termios;
use nix::unistd;

use std::fs;

use std::cmp;
use std::env;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A buffer contains text that can be edited.
struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    #[allow(unused)]
    fn new() -> Buffer {
        Buffer { lines: Vec::new() }
    }

    fn from_string(str: &str) -> Buffer {
        Buffer {
            lines: str.lines().map(String::from).collect(),
        }
    }
}

/// A cursor into a buffer content
struct Cursor {
    line: usize,
    column: usize,
}

/// User Preferences
struct UserPreferences {
    show_lines: bool,
}

/// The state of the editor.
struct Context {
    rows: usize,
    columns: usize,
    truecolor: bool,

    /// The column that a following [`next-line`](fn.next_line.html) or
    /// [`previous_line`](fn.previous_line.html) should try to move
    /// to. This is automatically reset to `None` after each user
    /// command is processed, unless
    /// [`to_preserve_goal_column`](#structfield.to_preserve_goal_column)
    /// is set to true by the command.
    goal_column: Option<usize>,

    cursor: Cursor,
    current_buffer: Buffer,
    scroll_line: usize,

    preferences: UserPreferences,

    // Result of a command. They will take effect once a full command
    // has been processed.
    to_exit: bool,
    to_refresh: bool,

    /// If set by a command, [`goal_column`](#structfield.goal_column) won't be reset after it.
    to_preserve_goal_column: bool,
}

impl Context {
    fn get_current_line(&self) -> &str {
        &self.current_buffer.lines[self.cursor.line]
    }
}

// Terminal
//
//

/// Execute a function with the terminal in raw mode.
///
/// The argument `run` will be executed with the terminal in "raw
/// mode". In this mode, echo is disabled, most key presses will be
/// available as inputs through STDIN.
///
/// After `run` returns, the terminal will be restored to the previous
/// configuration.
fn with_raw_mode<F: FnOnce()>(run: F) -> nix::Result<()> {
    let mut termios = termios::tcgetattr(libc::STDIN_FILENO)?;
    let original_termios = termios.clone();

    termios.input_flags &= !termios::InputFlags::IXON;
    termios.input_flags &= !termios::InputFlags::ICRNL; // Fix C-m to be read as 13, not 10

    termios.output_flags &= !termios::OutputFlags::OPOST;

    termios.local_flags &= !termios::LocalFlags::ECHO;
    termios.local_flags &= !termios::LocalFlags::ICANON;
    termios.local_flags &= !termios::LocalFlags::ISIG; // Fix C-z and C-c
    termios.local_flags &= !termios::LocalFlags::IEXTEN; // Fix C-o on Mac OS X

    // Legacy flags
    //
    // The rest of flags should not have any effect on modern
    // terminals, but they are traditionally part of the raw mode.
    termios.input_flags &= !termios::InputFlags::BRKINT;
    termios.input_flags &= !termios::InputFlags::INPCK;
    termios.input_flags &= !termios::InputFlags::ISTRIP;
    termios.control_flags |= termios::ControlFlags::CS8;

    // Be okay with read() returning 0 bytes read, and add a time out
    // of 1 1/10 of a second (100 ms)
    termios.control_chars[termios::SpecialCharacterIndices::VMIN as usize] = 0;
    termios.control_chars[termios::SpecialCharacterIndices::VTIME as usize] = 1;

    termios::tcsetattr(libc::STDIN_FILENO, termios::SetArg::TCSAFLUSH, &termios)?;

    run();

    termios::tcsetattr(
        libc::STDIN_FILENO,
        termios::SetArg::TCSAFLUSH,
        &original_termios,
    )?;

    return Ok(());
}

struct Term {
    buffer: String,
}

/// Specify which part of the terminal to erase.
#[allow(unused)]
enum ErasePart {
    /// Remove from the cursor until the end of the line/screen.
    ToEnd = 0,
    /// Remove from the beginning of the line/screen up to the cursor.
    ToStart = 1,
    /// Remove the full line/screen.
    All = 2,
}

impl Term {
    fn new() -> Term {
        Term {
            buffer: String::new(),
        }
    }

    fn write(&mut self, str: &str) {
        self.buffer.push_str(str);
    }
    fn flush(&mut self) {
        unistd::write(libc::STDOUT_FILENO, self.buffer.as_bytes()).unwrap();
        self.buffer.clear();
    }

    /// Generate a Control Sequence Introducer (CSI) escape code.
    fn csi(&mut self, s: &str) {
        self.write(&format!("\x1b[{}", s));
    }

    /// Enable the alternative screen buffer.
    ///
    /// It will switch to a screen buffer with no scrolling. You can
    /// restore the previous screen buffer, including all the content
    /// and scroll level of the terminal back by calling
    /// [`disable_alternative_screen_buffer`](fn.disable_alternative_screen_buffer.html).
    fn enable_alternative_screen_buffer(&mut self) {
        self.csi("?1049h");
    }

    /// Disable the the alternative screen buffer.
    ///
    /// Switch back to the screen buffer when
    /// [`enable_alternative_screen_buffer`](fn.enable_alternative_screen_buffer.html)
    /// was invoked. Restoring the content of the screen.
    fn disable_alternative_screen_buffer(&mut self) {
        self.csi("?1049l");
    }

    /// Clear the screen.
    #[allow(unused)]
    fn clear_screen(&mut self) {
        self.csi("2J");
    }

    /// Set the cursor position to `row` and `column`.`
    ///
    /// Both `row` and `column` start at 1.
    ///
    fn set_cursor(&mut self, row: usize, column: usize) {
        let str = format!("{};{}H", row, column);
        self.csi(&str);
    }

    fn hide_cursor(&mut self) {
        self.csi("?25l")
    }

    fn show_cursor(&mut self) {
        self.csi("?25h");
    }

    fn erase_line(&mut self, part: ErasePart) {
        self.csi(&format!("{}K", part as usize));
    }

    fn save_cursor(&mut self) {
        self.csi("s");
    }
    fn restore_cursor(&mut self) {
        self.csi("u");
    }
}

/// Get the number of rows and columns of the terminal.
fn get_window_size() -> (usize, usize) {
    unsafe {
        let mut winsize: libc::winsize = mem::zeroed();
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize);
        (winsize.ws_row as usize, winsize.ws_col as usize)
    }
}

fn support_true_color() -> bool {
    env::var("COLORTERM") == Ok(String::from("truecolor"))
}

// Rendering
//
//

/// Adjust the scroll level so the cursor is on the screen.
fn adjust_scroll(context: &mut Context) {
    if context.cursor.line < context.scroll_line {
        context.scroll_line -= 1;
    }
    if context.cursor.line > context.scroll_line + context.rows - 2 - 1 {
        context.scroll_line += 1;
    }
}

fn render_modeline(term: &mut Term, context: &Context) {
    if context.truecolor {
        term.csi(&format!("38;5;0m"));
        term.csi(&format!("48;2;{};{};{}m", 235, 171, 52));
    } else {
        term.csi("7m");
    }
    // On MacOsX's terminal, when you erase a line it won't fill the
    // full line with the current attributes, unlike ITerm. So we use
    // `write_line` to pad the string with spaces.
    write_line(
        term,
        &format!("  {}   L{}", "main.rs", context.cursor.line + 1),
        context.columns,
    );
}

/// Refresh the screen.
///
/// Ensure the terminal reflects the latest state of the editor.
fn refresh_screen(term: &mut Term, context: &Context) {
    term.hide_cursor();
    term.set_cursor(1, 1);

    let window_lines = context.rows - 2;

    let offset = if context.preferences.show_lines {
        let last_linenum_width = format!("{}", context.scroll_line + window_lines).len();
        last_linenum_width + 1
    } else {
        0
    };

    let window_columns = context.columns - offset;

    let buffer = &context.current_buffer;

    // Main window
    for row in 0..window_lines {
        let linenum = row + context.scroll_line;

        if let Some(line) = buffer.lines.get(linenum) {
            if context.preferences.show_lines {
                term.csi("38;5;240m");
                term.write(&format!("{:width$} ", linenum + 1, width = offset - 1));
            }

            term.csi("m");
            term.write(&line[..cmp::min(line.len(), window_columns)]);
            term.erase_line(ErasePart::ToEnd);
        }

        term.write("\r\n");
    }
    term.csi("m");

    if false {
        term.save_cursor();
        let welcome = "Welcome to the sted editor";
        term.set_cursor(
            context.rows / 2,
            window_columns / 2 - welcome.len() / 2 + offset,
        );
        term.write(&welcome);
        term.restore_cursor();
    }

    render_modeline(term, context);

    term.csi("m");
    write_line(term, "", window_columns);

    term.set_cursor(
        context.cursor.line - context.scroll_line + 1,
        context.cursor.column + offset + 1,
    );

    term.show_cursor();
    term.flush()
}

fn write_line(term: &mut Term, str: &str, width: usize) {
    assert!(str.len() <= width);
    let padded = format!("{:width$}", str, width = width);
    term.write(&padded);
}

#[derive(PartialEq, Debug)]
struct Key(u32);

/// Return a key made of a character with ctrl pressed.
///
/// ## Example
///
/// ```
/// ctrl('q')
/// ```
///
fn ctrl(ch: char) -> Key {
    Key(0x1f & (ch as u32))
}

#[allow(unused)]
/// Return a key from a character.
fn key(ch: char) -> Key {
    Key(ch as u32)
}

const ARROW_UP: &'static [u8; 2] = b"[A";
const ARROW_DOWN: &'static [u8; 2] = b"[B";
const ARROW_RIGHT: &'static [u8; 2] = b"[C";
const ARROW_LEFT: &'static [u8; 2] = b"[D";

/// Read and return a key.
///
/// If no key is entered by the user, the function will timeout and it
/// will return None instead.
///
fn read_key() -> Option<Key> {
    let mut buf = [0u8];
    unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();
    let cmd = buf[0] as u32;
    if cmd == 0 {
        None
    } else if cmd == 0x1b {
        let mut seq: [u8; 2] = [0; 2];
        unistd::read(libc::STDIN_FILENO, &mut seq).unwrap();
        match &seq {
            ARROW_UP => Some(ctrl('p')),
            ARROW_DOWN => Some(ctrl('n')),
            ARROW_RIGHT => Some(ctrl('f')),
            ARROW_LEFT => Some(ctrl('b')),
            _ => Some(Key(cmd)),
        }
    } else {
        Some(Key(cmd))
    }
}

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

fn move_beginning_of_line(context: &mut Context) {
    let line = context.get_current_line();
    let indentation = get_line_indentation(line);
    context.cursor.column = if context.cursor.column <= indentation {
        0
    } else {
        indentation
    };
}

fn move_end_of_line(context: &mut Context) {
    let eol = context.get_current_line().len();
    context.cursor.column = eol;
}

fn forward_char(context: &mut Context) {
    let len = context.get_current_line().len();
    if context.cursor.column < len {
        context.cursor.column += 1;
    } else {
        context.cursor.column = 0;
        next_line(context);
    }
}

fn backward_char(context: &mut Context) {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
    } else {
        previous_line(context);
        move_end_of_line(context);
    }
}

fn get_or_set_gaol_column(context: &mut Context) -> usize {
    // We set `to_preserve_goal_column` to ensure the goal_column is
    // not lost for the next command.
    context.to_preserve_goal_column = true;
    *context.goal_column.get_or_insert(context.cursor.column)
}

fn next_line(context: &mut Context) {
    if context.cursor.line < context.current_buffer.lines.len() - 1 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line += 1;
        context.cursor.column = cmp::min(context.get_current_line().len(), goal_column);
        adjust_scroll(context);
    }
}

fn previous_line(context: &mut Context) {
    if context.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line -= 1;
        context.cursor.column = cmp::min(context.get_current_line().len(), goal_column);
        adjust_scroll(context);
    }
}

/// Process user input.
fn process_user_input(context: &mut Context) -> bool {
    if let Some(k) = read_key() {
        context.to_refresh = true;
        if k == ctrl('q') {
            context.to_exit = true;
        }
        if k == ctrl('a') {
            move_beginning_of_line(context);
        }
        if k == ctrl('e') {
            move_end_of_line(context);
        }
        if k == ctrl('f') {
            forward_char(context);
        }
        if k == ctrl('b') {
            backward_char(context);
        }
        if k == ctrl('p') {
            previous_line(context);
        }
        if k == ctrl('n') {
            next_line(context);
        }
        true
    } else {
        false
    }
}

/// The main entry point of the editor.
fn main() {
    let (rows, columns) = get_window_size();
    let mut context = Context {
        rows,
        columns,
        truecolor: support_true_color(),

        goal_column: None,
        cursor: Cursor { line: 0, column: 0 },

        preferences: UserPreferences { show_lines: true },

        current_buffer: Buffer::from_string(&fs::read_to_string("src/main.rs").unwrap()),
        scroll_line: 0,

        to_exit: false,
        to_refresh: false,
        to_preserve_goal_column: false,
    };

    // Detect when the terminal was resized
    let was_resize = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGWINCH, Arc::clone(&was_resize)).unwrap();

    let mut term = Term::new();

    term.enable_alternative_screen_buffer();

    refresh_screen(&mut term, &context);

    with_raw_mode(|| loop {
        if was_resize.load(Ordering::Relaxed) {
            let (rows, columns) = get_window_size();
            context.rows = rows;
            context.columns = columns;
            adjust_scroll(&mut context);
            refresh_screen(&mut term, &context);
            was_resize.store(false, Ordering::Relaxed);
        }

        context.to_preserve_goal_column = false;
        context.to_refresh = false;

        if !process_user_input(&mut context) {
            continue;
        }

        if context.to_exit {
            break;
        }

        if context.to_refresh {
            refresh_screen(&mut term, &context);
        }

        if !context.to_preserve_goal_column {
            context.goal_column = None;
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");

    term.disable_alternative_screen_buffer();
    term.show_cursor();
    term.flush();
}
