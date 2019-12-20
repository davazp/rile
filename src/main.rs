use nix;
use nix::libc;
use nix::sys::termios;
use nix::unistd;

// Put the terminal into raw mode
//
// The returned `RawModeGuard` guard will restore the terminal to its
// original state when dropped.
//

struct RawModeGuard {
    original_termios: termios::Termios,
}

fn raw_mode() -> nix::Result<RawModeGuard> {
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

    return Ok(RawModeGuard { original_termios });
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        termios::tcsetattr(
            libc::STDIN_FILENO,
            termios::SetArg::TCSAFLUSH,
            &self.original_termios,
        )
        .expect("fail to reset terminal into cooked mode.")
    }
}

fn main() {
    let mut buf = [0u8];

    let raw_mode_guard = raw_mode().expect("Could not initialize the terminal into raw mode.");

    loop {
        let result = unistd::read(libc::STDIN_FILENO, &mut buf).unwrap();

        match buf[0] as char {
            'q' => break,
            _ => (),
        };

        print!("read {} bytes: {:?}\r\n", result, buf);
    }

    drop(raw_mode_guard)
}
