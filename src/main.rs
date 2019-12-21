//! sted is a simple editor written in Rust.
//!

extern crate signal_hook;

use nix;
use nix::libc;
use nix::sys::termios;
use nix::unistd;

use std::env;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The state of the editor.
struct Context {
    rows: usize,
    columns: usize,
    truecolor: bool,

    cursor_line: usize,
    cursor_column: usize,
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
    /// restore the previous screen buffer, including all the content and
    /// scroll level of the terminal back by calling
    /// `disable_alternative_screen_buffer`.
    fn enable_alternative_screen_buffer(&mut self) {
        self.csi("?1049h");
    }

    /// Disable the the alternative screen buffer.
    ///
    /// Switch back to the screen buffer when
    /// `enable_alternative_screen_buffer` was invoked. Restoring the
    /// content of the screen.
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

    fn erase_line(&mut self) {
        self.csi("2K");
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

/// Refresh the screen.
///
/// Ensure the terminal reflects the latest state of the editor.
fn refresh_screen(term: &mut Term, context: &Context) {
    term.hide_cursor();
    term.set_cursor(1, 1);

    let offset = 4;

    // Main window
    term.csi("38;5;240m");
    for row in 1..(context.rows - 1) {
        term.erase_line();
        term.write(&format!("{:width$} \r\n", row, width = offset - 1));
    }
    term.csi("m");

    term.save_cursor();
    let welcome = "Welcome to the sted editor";
    term.set_cursor(
        context.rows / 2,
        (context.columns - offset) / 2 - welcome.len() / 2 + offset,
    );
    term.write(&welcome);
    term.restore_cursor();

    // Modeline
    if context.truecolor {
        term.csi(&format!("38;5;0m"));
        term.csi(&format!("48;2;{};{};{}m", 235, 171, 52));
    } else {
        term.csi("7m");
    }
    term.erase_line();
    term.write("  main.rs");

    term.show_cursor();

    term.csi("m");

    term.set_cursor(context.cursor_line + 1, context.cursor_column + offset + 1);

    term.flush()
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
    Key(0x17 & (ch as u32))
}

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
    } else {
        Some(Key(cmd))
    }
}

/// The main entry point of the editor.
fn main() {
    let (rows, columns) = get_window_size();
    let mut context = Context {
        rows,
        columns,
        truecolor: support_true_color(),

        cursor_line: 0,
        cursor_column: 0,
    };

    // Detect when the terminal was resized
    let was_resize = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGWINCH, Arc::clone(&was_resize)).unwrap();

    let mut term = Term::new();

    term.enable_alternative_screen_buffer();

    with_raw_mode(|| {
        refresh_screen(&mut term, &context);
        loop {
            if was_resize.load(Ordering::Relaxed) {
                let (rows, columns) = get_window_size();
                context.rows = rows;
                context.columns = columns;
                refresh_screen(&mut term, &context);
                was_resize.store(false, Ordering::Relaxed);
            }

            if let Some(key) = read_key() {
                if key == ctrl('q') {
                    break;
                }

                term.set_cursor(1, 5);
                term.write(format!("{:?}", key).as_ref());
                term.flush();
            }
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");

    term.disable_alternative_screen_buffer();
    term.show_cursor();
    term.flush();
}
