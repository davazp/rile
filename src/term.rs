use nix::libc;
use nix::sys::termios;
use nix::unistd;
use std::env;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::Key;

/// Execute a function with the terminal in raw mode.
///
/// The argument `run` will be executed with the terminal in "raw
/// mode". In this mode, echo is disabled, most key presses will be
/// available as inputs through STDIN.
///
/// After `run` returns, the terminal will be restored to the previous
/// configuration.
pub fn with_raw_mode<F: FnOnce()>(run: F) -> nix::Result<()> {
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

    // Be okay with read() returning 0 bytes read
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

pub struct Term {
    buffer: String,
    // The size of the terminal
    pub rows: usize,
    pub columns: usize,
}

impl Term {
    pub fn new() -> Term {
        let (rows, columns) = get_window_size();
        Term {
            buffer: String::new(),
            rows,
            columns,
        }
    }

    pub fn write(&mut self, str: &str) {
        self.buffer.push_str(str);
    }

    /// Write the line `str` a line padded to `width`.
    pub fn write_line<T: AsRef<str>>(&mut self, str: T) {
        let str = str.as_ref();
        self.write(&str);
        self.erase_line(ErasePart::ToEnd);
        self.csi("E");
    }

    pub fn flush(&mut self) {
        let bytes = self.buffer.as_bytes();
        if cfg!(feature = "debug_slow_term") {
            for chunk in bytes.chunks(16) {
                unistd::write(libc::STDOUT_FILENO, chunk).unwrap();
                thread::sleep(Duration::from_micros(750));
            }
        } else {
            unistd::write(libc::STDOUT_FILENO, bytes).unwrap();
        }
        self.buffer.clear();
    }

    /// Generate a Control Sequence Introducer (CSI) escape code.
    pub fn csi(&mut self, s: &str) {
        self.write(&format!("\x1b[{}", s));
    }

    /// 8-bit
    pub fn fg(&mut self, n: u8) {
        self.csi(&format!("38;5;{}m", n));
    }
    pub fn bg(&mut self, n: u8) {
        self.csi(&format!("48;5;{}m", n));
    }

    /// True color
    pub fn rgb_fg(&mut self, r: u8, g: u8, b: u8) {
        self.csi(&format!("38;2;{};{};{}m", r, g, b));
    }

    pub fn rgb_bg(&mut self, r: u8, g: u8, b: u8) {
        self.csi(&format!("48;2;{};{};{}m", r, g, b));
    }

    pub fn reset_attr(&mut self) {
        self.csi("m")
    }

    /// Enable the alternative screen buffer.
    ///
    /// It will switch to a screen buffer with no scrolling. You can
    /// restore the previous screen buffer, including all the content
    /// and scroll level of the terminal back by calling
    /// [`disable_alternative_screen_buffer`](fn.disable_alternative_screen_buffer.html).
    pub fn enable_alternative_screen_buffer(&mut self) {
        self.csi("?1049h");
    }

    /// Disable the the alternative screen buffer.
    ///
    /// Switch back to the screen buffer when
    /// [`enable_alternative_screen_buffer`](fn.enable_alternative_screen_buffer.html)
    /// was invoked. Restoring the content of the screen.
    pub fn disable_alternative_screen_buffer(&mut self) {
        self.csi("?1049l");
    }

    /// Clear the screen.
    #[allow(unused)]
    pub fn clear_screen(&mut self) {
        self.csi("2J");
    }

    /// Set the cursor position to `row` and `column`.`
    ///
    /// Both `row` and `column` start at 1.
    ///
    pub fn set_cursor(&mut self, row: usize, column: usize) {
        let str = format!("{};{}H", row, column);
        self.csi(&str);
    }

    pub fn hide_cursor(&mut self) {
        self.csi("?25l")
    }

    pub fn show_cursor(&mut self) {
        self.csi("?25h");
    }

    pub fn erase_line(&mut self, part: ErasePart) {
        self.csi(&format!("{}K", part as usize));
    }

    pub fn erase_display(&mut self, part: ErasePart) {
        self.csi(&format!("{}J", part as usize));
    }

    #[allow(unused)]
    pub fn save_cursor(&mut self) {
        self.csi("s");
    }

    #[allow(unused)]
    pub fn restore_cursor(&mut self) {
        self.csi("u");
    }
}

/// Specify which part of the terminal to erase.
#[allow(unused)]
pub enum ErasePart {
    /// Remove from the cursor until the end of the line/screen.
    ToEnd = 0,
    /// Remove from the beginning of the line/screen up to the cursor.
    ToStart = 1,
    /// Remove the full line/screen.
    All = 2,
}

/// Get the number of rows and columns of the terminal.
pub fn get_window_size() -> (usize, usize) {
    unsafe {
        let mut winsize: libc::winsize = mem::zeroed();
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize);
        (winsize.ws_row as usize, winsize.ws_col as usize)
    }
}

#[allow(unused)]
fn support_true_color() -> bool {
    env::var("COLORTERM") == Ok(String::from("truecolor"))
}

/// Read and return a key.
pub fn read_key_timeout() -> Option<Key> {
    const ARROW_UP: &'static [u8; 2] = b"[A";
    const ARROW_DOWN: &'static [u8; 2] = b"[B";
    const ARROW_RIGHT: &'static [u8; 2] = b"[C";
    const ARROW_LEFT: &'static [u8; 2] = b"[D";

    let mut buf = [0u8];
    unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();
    let cmd = buf[0] as u32;
    if cmd == 0x1b {
        let mut seq: [u8; 2] = [0; 2];
        unistd::read(libc::STDIN_FILENO, &mut seq).unwrap();

        if seq[1] == 0 {
            Some(Key::from_code(seq[0] as u32).meta())
        } else {
            match &seq {
                ARROW_UP => Some(Key::parse_unchecked("C-p")),
                ARROW_DOWN => Some(Key::parse_unchecked("C-n")),
                ARROW_RIGHT => Some(Key::parse_unchecked("C-f")),
                ARROW_LEFT => Some(Key::parse_unchecked("C-b")),
                _ => None,
            }
        }
    } else if cmd > 0 {
        Some(Key::from_code(cmd))
    } else {
        None
    }
}

pub fn reconciliate_term_size(term: &mut Term, was_resized: &AtomicBool) -> bool {
    if was_resized.load(Ordering::Relaxed) {
        let (rows, columns) = get_window_size();
        term.rows = rows;
        term.columns = columns;
        was_resized.store(false, Ordering::Relaxed);
        true
    } else {
        false
    }
}

/// Discard all user inputs that have not being read yet.
pub fn discard_input_buffer() {
    let _ = termios::tcflush(libc::STDIN_FILENO, termios::FlushArg::TCIFLUSH);
}
