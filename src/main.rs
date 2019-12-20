use nix::libc::STDIN_FILENO;
use nix::unistd;

fn main() {
    let mut buf = [0u8];

    loop {
        let result = unistd::read(STDIN_FILENO, &mut buf).unwrap();

        match buf[0] as char {
            'q' => break,
            _ => (),
        };

        println!("read {} bytes: {:?}", result, buf);
    }
}
