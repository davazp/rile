use nix;
use nix::libc;
use nix::sys::termios;
use nix::unistd;

// Put the terminal into raw mode
//
// The returned `RawModeGuard` guard will restore the terminal to its
// original state when dropped.
//

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

fn csi(s: &str) {
    unistd::write(libc::STDOUT_FILENO, format!("\x1b[{}", s).as_bytes()).unwrap();
}

// Alternative screen allows us to enter in the editor and then
// restore back the content of the terminal and scroll level.

fn enable_alternative_screen_buffer() {
    csi("?1049h");
}

fn disable_alternative_screen_buffer() {
    csi("?1049l");
}

//
// Rendering
//

fn clear_screen() {
    csi("2J");
}

fn set_cursor(n: u32, m: u32) {
    let str = format!("{};{}H", n, m);
    csi(&str);
}

//
// Input processing
//

#[derive(PartialEq, Debug)]
struct Key(u32);

fn ctrl(ch: char) -> Key {
    Key(0x17 & (ch as u32))
}

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
