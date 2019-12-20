use nix::libc::STDIN_FILENO;
use nix::unistd;

fn main() {
    let mut buf = [0];

    let result = unistd::read(STDIN_FILENO, &mut buf).unwrap();
    println!("read {} bytes: {:?}", result, buf);
}
