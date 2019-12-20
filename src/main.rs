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

    termios.input_flags.remove(termios::InputFlags::IXON); // Fix C-s and C-w
    termios.input_flags.remove(termios::InputFlags::ICRNL); // Fix C-m to be read as 13, not 10

    termios.output_flags.remove(termios::OutputFlags::OPOST);

    termios.local_flags.remove(termios::LocalFlags::ECHO);
    termios.local_flags.remove(termios::LocalFlags::ICANON);
    termios.local_flags.remove(termios::LocalFlags::ISIG); // Fix C-z and C-c
    termios.local_flags.remove(termios::LocalFlags::IEXTEN); // Fix C-o on Mac OS X

    // Legacy flags
    //
    // The rest of flags should not have any effect on modern
    // terminals, but they are traditionally part of the raw mode.
    termios.input_flags.remove(termios::InputFlags::BRKINT);
    termios.input_flags.remove(termios::InputFlags::INPCK);
    termios.input_flags.remove(termios::InputFlags::ISTRIP);
    termios.control_flags.insert(termios::ControlFlags::CS8);

    termios::tcsetattr(libc::STDIN_FILENO, termios::SetArg::TCSAFLUSH, &termios)?;

    run();

    termios::tcsetattr(
        libc::STDIN_FILENO,
        termios::SetArg::TCSAFLUSH,
        &original_termios,
    )?;

    return Ok(());
}

fn main() {
    with_raw_mode(|| {
        let mut buf = [0u8];

        loop {
            let result = unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();

            match buf[0] as char {
                'q' => break,
                _ => (),
            };

            print!("read {} bytes: {:?}\r\n", result, buf);
        }
    })
    .expect("Could not initialize the terminal to run in raw mode.");
}
