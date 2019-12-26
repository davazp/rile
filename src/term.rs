use nix::libc;
use nix::sys::termios;
use nix::unistd;
use std::env;
use std::mem;

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

    // Be okay with read() returning 0 bytes read, and add a time out
    // of 1 1/10 of a second (100 ms)
    // termios.control_chars[termios::SpecialCharacterIndices::VMIN as usize] = 0;
    // termios.control_chars[termios::SpecialCharacterIndices::VTIME as usize] = 1;

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
    pub fn flush(&mut self) {
        unistd::write(libc::STDOUT_FILENO, self.buffer.as_bytes()).unwrap();
        self.buffer.clear();
    }

    /// Generate a Control Sequence Introducer (CSI) escape code.
    pub fn csi(&mut self, s: &str) {
        self.write(&format!("\x1b[{}", s));
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
