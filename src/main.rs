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

fn clear_screen() {
    unistd::write(libc::STDOUT_FILENO, "\x1b[2J".as_bytes()).unwrap();
    // Reposition cursor
    unistd::write(libc::STDOUT_FILENO, "\x1b[2J".as_bytes()).unwrap();
    unistd::write(libc::STDOUT_FILENO, "\x1b[H".as_bytes()).unwrap();
}

fn ctrl(ch: char) -> u32 {
    0x17 & (ch as u32)
}

fn main() {
    clear_screen();

    with_raw_mode(|| {
        let mut buf = [0u8];

        loop {
            let read = unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();
            let cmd = buf[0] as u32;

            if cmd == ctrl('q') {
                break;
            }

            if read > 0 {
                clear_screen();
                print!("read {} bytes: {:?}\r\n", read, buf);
            }
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");
}
