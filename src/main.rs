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
    termios.local_flags.remove(termios::LocalFlags::ECHO);
    termios::tcsetattr(libc::STDIN_FILENO, termios::SetArg::TCSAFLUSH, &termios)?;
    return Ok(RawModeGuard { original_termios });
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        println!("restoring terminal");
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

        println!("read {} bytes: {:?}", result, buf);
    }

    drop(raw_mode_guard)
}
