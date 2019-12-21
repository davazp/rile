//! sted is a simple editor written in Rust.
//!

use nix;
use nix::libc;
use nix::sys::termios;
use nix::unistd;

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

/// Generate a Control Sequence Introducer (CSI) escape code.
fn csi(s: &str) {
    unistd::write(libc::STDOUT_FILENO, format!("\x1b[{}", s).as_bytes()).unwrap();
}

// Alternative screen allows us to enter in the editor and then
// restore back the content of the terminal and scroll level.

/// Enable the alternative screen buffer.
///
/// It will switch to a screen buffer with no scrolling. You can
/// restore the previous screen buffer, including all the content and
/// scroll level of the terminal back by calling
/// `disable_alternative_screen_buffer`.
fn enable_alternative_screen_buffer() {
    csi("?1049h");
}

/// Disable the the alternative screen buffer.
///
/// Switch back to the screen buffer when
/// `enable_alternative_screen_buffer` was invoked. Restoring the
/// content of the screen.
fn disable_alternative_screen_buffer() {
    csi("?1049l");
}

//
// Rendering
//

/// Clear the screen.
fn clear_screen() {
    csi("2J");
}

/// Set the cursor position to the row `n` and column `m`.
///
/// Both `row` and `column` start at 1.
///
fn set_cursor(row: u32, column: u32) {
    let str = format!("{};{}H", row, column);
    csi(&str);
}

//
// Input processing
//

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
    enable_alternative_screen_buffer();

    with_raw_mode(|| {
        clear_screen();
        loop {
            if let Some(key) = read_key() {
                if key == ctrl('q') {
                    break;
                }

                clear_screen();
                set_cursor(1, 1);
                unistd::write(libc::STDOUT_FILENO, format!("{:?}", key).as_bytes()).unwrap();
            }
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");

    disable_alternative_screen_buffer()
}
