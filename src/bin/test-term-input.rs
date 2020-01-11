use rile::term::{self, with_raw_mode};
use rile::Key;

fn main() {
    println!("Reading and printing keys. Press 'q' to exit.\n");

    let _ = with_raw_mode(|| loop {
        if let Some(key) = term::read_key_timeout() {
            print!("{} ({})\r\n", key, key.to_code());

            if key == Key::parse("q").unwrap() {
                break;
            }
        }
    });
}
